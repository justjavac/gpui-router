use crate::matcher::MatchedRoute;
use crate::{CurrentWindowId, Route, RouterRegistry, RouterState, normalize_pathname, normalize_shared_pathname};
use gpui::*;
use smallvec::SmallVec;

/// Renders a branch of [`Route`](crate::Route) that best matches the current path.
#[derive(IntoElement)]
pub struct Routes {
  basename: SharedString,
  routes: SmallVec<[Route; 1]>,
}

impl Default for Routes {
  fn default() -> Self {
    Self::new()
  }
}

impl Routes {
  /// Creates an empty route tree that matches under `/`.
  ///
  /// Child routes are added with [`Routes::child`] or [`Routes::children`], and
  /// the whole tree renders once it is used as a `Routes` element, normally as
  /// the root of a window's view tree or the `div` it renders inside.
  ///
  /// ```
  /// use gpui_router::prelude::*;
  ///
  /// let routes = Routes::new().child(Route::new().path("home").element(|_, _| "Home"));
  /// ```
  pub fn new() -> Self {
    Self {
      basename: SharedString::from("/"),
      routes: SmallVec::new(),
    }
  }

  /// Sets the base path for all child `Route`s. All child paths then resolve
  /// under it, so `Routes::new().basename("/app")` with a child `path("home")`
  /// matches `/app/home`. The basename is normalized like every route path.
  pub fn basename(mut self, basename: impl Into<SharedString>) -> Self {
    self.basename = normalize_pathname(basename.into());
    self
  }

  /// Adds one [`Route`] as a child of the tree, mirroring a React Router
  /// `<Routes>` wrapping a single `<Route>`.
  ///
  /// Callers usually chain this: `Routes::new().child(Route::new()...).child(...)`.
  pub fn child(mut self, child: Route) -> Self {
    self.routes.push(child);
    self
  }

  /// Adds several [`Route`]s as children, mirroring a React Router `<Routes>`
  /// wrapping a list of `<Route>`s. Equivalent to calling [`Routes::child`]
  /// once per route.
  pub fn children(mut self, children: impl IntoIterator<Item = Route>) -> Self {
    for child in children.into_iter() {
      self = self.child(child);
    }
    self
  }

  #[cfg(test)]
  pub fn routes(&self) -> &SmallVec<[Route; 1]> {
    &self.routes
  }

  #[cfg(test)]
  pub(crate) fn match_route(&self, pathname: &str) -> Option<MatchedRoute> {
    crate::matcher::match_path(&self.routes, self.basename.as_ref(), pathname)
  }

  /// Matches an already-normalized pathname without allocating.
  fn match_normalized(&self, pathname: &str) -> Option<MatchedRoute> {
    crate::matcher::match_normalized(&self.routes, self.basename.as_ref(), pathname)
  }

  /// Writes the match into the router state of the window being rendered,
  /// reusing the existing parameter map so a render pass does not allocate one
  /// per frame.
  pub(crate) fn apply_match(cx: &mut App, pathname: SharedString, matched: Option<MatchedRoute>) {
    let state = RouterState::require_mut(cx);
    state.location.pathname = pathname;

    state.matched_pattern = matched.as_ref().map(|matched| matched.pattern.clone());
    state.params.clear();
    if let Some(matched) = matched {
      state.params.extend(matched.params);
    }

    // The matched chain is recorded again while the router renders.
    state.matches.clear();
    state.current_route = None;
  }
}

/// Wraps the matched route element of a `Routes` tree, isolating the tree's
/// render frame while its subtree lays out.
///
/// `Routes` is a `RenderOnce` component, so its `render` runs while the parent
/// tree lays out: that is where the window is bound and its state is created.
/// The frame that replaces the window's match chain has to bracket the layout
/// of this tree's own subtree, which is what this element does, the same way
/// [`RouteScope`](crate::route) brackets a single route's subtree.
struct RoutesScope {
  window_id: WindowId,
  pathname: SharedString,
  matched: Option<MatchedRoute>,
  element: AnyElement,
}

impl Element for RoutesScope {
  type RequestLayoutState = ();
  type PrepaintState = Option<FocusHandle>;

  fn id(&self) -> Option<ElementId> {
    None
  }

  fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
    None
  }

  fn request_layout(
    &mut self,
    _id: Option<&GlobalElementId>,
    _inspector_id: Option<&InspectorElementId>,
    window: &mut Window,
    cx: &mut App,
  ) -> (LayoutId, Self::RequestLayoutState) {
    // Laying the subtree out is what renders the components inside it, so the
    // frame covers this call: it replaces the window's previous match chain,
    // lets this tree record its own, then restores the previous chain, so a
    // nested `Routes` tree cannot overwrite what an outer tree recorded.
    {
      let registry = cx.global_mut::<RouterRegistry>();
      registry.enter_tree_scope(self.window_id);
    }

    // Matching while the subtree lays out makes the components inside it see
    // this tree's location, the way the route scope does for one route.
    Routes::apply_match(cx, self.pathname.clone(), self.matched.clone());

    let id = self.element.request_layout(window, cx);

    {
      let registry = cx.global_mut::<RouterRegistry>();
      registry.leave_tree_scope(self.window_id);
    }

    (id, ())
  }

  fn prepaint(
    &mut self,
    _id: Option<&GlobalElementId>,
    _inspector_id: Option<&InspectorElementId>,
    _bounds: Bounds<Pixels>,
    _request_layout: &mut Self::RequestLayoutState,
    window: &mut Window,
    cx: &mut App,
  ) -> Self::PrepaintState {
    self.element.prepaint(window, cx)
  }

  fn paint(
    &mut self,
    _id: Option<&GlobalElementId>,
    _inspector_id: Option<&InspectorElementId>,
    _bounds: Bounds<Pixels>,
    _request_layout: &mut Self::RequestLayoutState,
    _prepaint: &mut Self::PrepaintState,
    window: &mut Window,
    cx: &mut App,
  ) {
    self.element.paint(window, cx);
  }
}

impl IntoElement for RoutesScope {
  type Element = Self;

  fn into_element(self) -> Self::Element {
    self
  }
}

impl RenderOnce for Routes {
  fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
    let window_id = window.window_handle().window_id();
    cx.set_global(CurrentWindowId(window_id));

    {
      let registry = cx.global_mut::<RouterRegistry>();
      registry.ensure_window(window_id, &self.basename);
    }

    let pathname = normalize_shared_pathname(&RouterState::require(cx).location.pathname);
    let matched = self.match_normalized(pathname.as_ref());
    let index = matched.as_ref().map(|matched| matched.index);

    let element = if let Some(index) = index
      && let Some(route) = self.routes.into_iter().nth(index)
    {
      route.basename(self.basename).into_any_element()
    } else {
      Empty {}.into_any_element()
    };

    RoutesScope {
      window_id,
      pathname,
      matched,
      element,
    }
    .into_any_element()
  }
}

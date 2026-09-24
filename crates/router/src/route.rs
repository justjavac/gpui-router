use crate::matcher;
use crate::outlet::with_outlet_scope;
use crate::{Layout, RouterState, normalize_pathname};
use gpui::*;
use smallvec::SmallVec;
use std::fmt::{Debug, Display};

type RouteElementFactory = Box<dyn Fn(&mut Window, &mut App) -> AnyElement>;

/// Creates a new [`Route`](crate::Route) element.
pub fn route() -> impl IntoElement {
  Route::new()
}

/// Configures an element to render when a pattern matches the current path.
/// It must be rendered within a [`Routes`](crate::Routes) element.
#[derive(IntoElement)]
pub struct Route {
  basename: SharedString,
  pub(crate) path: Option<SharedString>,
  pub(crate) element: Option<RouteElementFactory>,
  pub(crate) routes: SmallVec<[Box<Route>; 1]>,
  pub(crate) layout: Option<Box<dyn Layout>>,
}

impl Default for Route {
  fn default() -> Self {
    Self {
      basename: SharedString::default(),
      path: None,
      element: None,
      routes: SmallVec::new(),
      layout: None,
    }
  }
}

impl Debug for Route {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Route")
      .field("basename", &self.basename)
      .field("path", &self.path)
      .field("layout", &self.layout.is_some())
      .field("element", &self.element.is_some())
      .field("routes", &self.routes.len())
      .finish()
  }
}

impl Display for Route {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Route")
  }
}

impl Route {
  pub fn new() -> Self {
    Self::default()
  }

  pub(crate) fn basename(mut self, basename: impl Into<SharedString>) -> Self {
    self.basename = basename.into();
    self
  }

  /// The path to match against the current location.
  pub fn path(mut self, path: impl Into<SharedString>) -> Self {
    self.path = Some(path.into());
    self
  }

  /// The element to render when the route matches.
  /// Accepts a closure that returns an IntoElement, which will be called lazily when the route matches.
  ///
  /// The matched child of a route with children renders into the first
  /// [`Outlet`](crate::Outlet) created by this element, so an outlet has to be
  /// built inside the element closure; children render only through it. Use
  /// [`Route::layout`] instead when the surrounding chrome needs its own type.
  /// Panics in debug builds if a layout is already set.
  ///
  /// # Examples
  /// ```
  /// Route::new().path("home").element(|| HomeView::render())
  /// Route::new().path("about").element(|| div().child("About"))
  /// ```
  pub fn element<F, E>(mut self, element_fn: F) -> Self
  where
    F: Fn(&mut Window, &mut App) -> E + 'static,
    E: IntoElement,
  {
    if cfg!(debug_assertions) && self.layout.is_some() {
      panic!("Route element and layout cannot be set at the same time");
    }

    self.element = Some(Box::new(move |window, cx| element_fn(window, cx).into_any_element()));
    self
  }

  /// The layout to use when the route matches.
  /// Panics if an element is already set.
  pub fn layout(mut self, layout: impl Layout + 'static) -> Self {
    if cfg!(debug_assertions) && self.element.is_some() {
      panic!("Route element and layout cannot be set at the same time");
    }

    self.layout = Some(Box::new(layout));
    self
  }

  /// Sets the route as an index route.
  /// Panics if a path is already set.
  pub fn index(self) -> Self {
    if cfg!(debug_assertions) && self.path.is_some() {
      panic!("Route index and path cannot be set at the same time");
    }
    self.path("")
  }

  /// Adds a `Route` as a child to the `Route`.
  ///
  /// The child renders into this route's outlet: the first
  /// [`Outlet`](crate::Outlet) built inside [`Route::element`], or the layout
  /// set with [`Route::layout`]. A route with neither renders the matched child
  /// directly, like a pathless layout route in React Router.
  pub fn child(mut self, child: Route) -> Self {
    self.routes.push(Box::new(child));
    self
  }

  /// Adds multiple `Route`s as children to the `Route`.
  pub fn children(mut self, children: impl IntoIterator<Item = Route>) -> Self {
    for child in children.into_iter() {
      self = self.child(child);
    }
    self
  }

  pub(crate) fn full_path(&self, basename: &str) -> SharedString {
    let basename = basename.trim_end_matches('/');
    let path = match self.path {
      Some(ref path) => format!("{}/{}", basename, path),
      None => basename.to_string(),
    };
    normalize_pathname(path)
  }

  /// Renders and removes the child that matches the current pathname.
  fn take_matched_child(
    routes: &mut SmallVec<[Box<Route>; 1]>,
    basename: &str,
    window: &mut Window,
    cx: &mut App,
  ) -> Option<AnyElement> {
    // The location pathname is normalized, so matching borrows it as is.
    let pathname = RouterState::require(cx).location.pathname.clone();
    let matched = matcher::match_normalized(routes, basename, pathname.as_ref())?;
    let route = routes.remove(matched.index);

    // Fully qualified because newer GPUI releases add a `View::render` for every
    // type, which makes the method call ambiguous.
    Some(RenderOnce::render(route.basename(basename.to_owned()), window, cx).into_any_element())
  }
}

impl RenderOnce for Route {
  fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
    let basename = self.full_path(self.basename.as_ref());
    RouterState::require_mut(cx).record_match(&basename);
    let mut routes = std::mem::take(&mut self.routes);

    if let Some(element_fn) = self.element {
      let child = Route::take_matched_child(&mut routes, basename.as_ref(), window, cx);
      return with_outlet_scope(child, || element_fn(window, cx));
    }

    if let Some(mut layout) = self.layout {
      if let Some(child) = Route::take_matched_child(&mut routes, basename.as_ref(), window, cx) {
        layout.outlet(child);
      }
      return layout.render_layout(window, cx).into_any_element();
    }

    // A route with children but no chrome of its own renders the matched child
    // directly, which is React Router's pathless layout route.
    if let Some(child) = Route::take_matched_child(&mut routes, basename.as_ref(), window, cx) {
      return child;
    }

    Empty {}.into_any_element()
  }
}

#[cfg(test)]
mod tests {
  use super::Route;

  #[test]
  fn test_element_route_keeps_children() {
    let route = Route::new()
      .element(|_, _| "home")
      .child(Route::new().index().element(|_, _| "index"));

    assert_eq!(route.routes.len(), 1);
  }

  #[test]
  fn test_children_are_allowed_without_element() {
    let route = Route::new()
      .child(Route::new().index().element(|_, _| "index"))
      .child(Route::new().path("about").element(|_, _| "about"));

    assert_eq!(route.routes.len(), 2);
  }
}

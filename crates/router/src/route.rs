use crate::{Layout, RouterState};
use gpui::*;
use matchit::Router as MatchitRouter;
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
  path: Option<SharedString>,
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
  /// Panics if a layout is already set.
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

  pub(crate) fn build_route_map(&self, basename: &str) -> MatchitRouter<()> {
    let basename = basename.trim_end_matches('/');
    let mut router_map = MatchitRouter::new();

    let path = match self.path {
      Some(ref path) => format!("{}/{}", basename, path),
      None => basename.to_string(),
    };

    let path = if path != "/" { path.trim_end_matches('/') } else { &path };

    if self.element.is_some() {
      router_map.insert(path, ()).unwrap();
      return router_map;
    }

    // Recursively build the route map
    for route in self.routes.iter() {
      router_map.merge(route.build_route_map(path)).unwrap();
    }

    router_map
  }

  pub(crate) fn in_pattern(&self, basename: &str, path: &str) -> bool {
    self.build_route_map(basename).at(path).is_ok()
  }
}

impl RenderOnce for Route {
  fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
    if let Some(element_fn) = self.element {
      return element_fn(window, cx);
    }

    if let Some(mut layout) = self.layout {
      let pathname = cx.global::<RouterState>().location.pathname.clone();
      let basename = self.basename.trim_end_matches('/');
      let basename = match self.path {
        Some(ref path) => format!("{}/{}", basename, path),
        None => basename.to_string(),
      };
      let routes = std::mem::take(&mut self.routes);
      let route = routes.into_iter().find(|route| route.in_pattern(&basename, &pathname));
      if let Some(route) = route {
        layout.outlet(route.basename(basename).render(window, cx).into_any_element());
      }
      return layout.render_layout(window, cx).into_any_element();
    }
    Empty {}.into_any_element()
  }
}

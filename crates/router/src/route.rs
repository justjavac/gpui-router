use crate::{Layout, RouterState};
use gpui::*;
use matchit::Router as MatchitRouter;
use smallvec::SmallVec;
use std::fmt::{Debug, Display};

pub fn route() -> impl IntoElement {
  Route::new()
}

/// Configures an element to render when a pattern matches the current path.
/// It must be rendered within a [`Routes`](crate::Routes) element.
#[derive(IntoElement, Default)]
pub struct Route {
  basename: SharedString,
  index: bool,
  path: Option<SharedString>,
  pub(crate) element: Option<AnyElement>,
  pub(crate) routes: SmallVec<[Box<Route>; 1]>,
  pub(crate) layout: Option<Box<dyn Layout>>,
}

impl Debug for Route {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Route")
      .field("basename", &self.basename)
      .field("index", &self.index)
      .field("path", &self.path)
      .field("routes", &self.routes)
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

  /// The path to match against the current location.
  pub fn path(mut self, path: impl Into<SharedString>) -> Self {
    if self.index {
      panic!("Route path and index cannot be set at the same time");
    }

    self.path = Some(path.into());
    self
  }

  pub(crate) fn basename(mut self, basename: impl Into<SharedString>) -> Self {
    self.basename = basename.into();
    self
  }

  pub fn element<E: IntoElement>(mut self, element: E) -> Self {
    if cfg!(debug_assertions) && self.layout.is_some() {
      panic!("Route element and layout cannot be set at the same time");
    }

    self.element = Some(element.into_any_element());
    self
  }

  pub fn layout(mut self, layout: impl Layout + 'static) -> Self {
    if cfg!(debug_assertions) && self.element.is_some() {
      panic!("Route element and layout cannot be set at the same time");
    }

    self.layout = Some(Box::new(layout));
    self
  }

  pub fn index(mut self) -> Self {
    if self.path.is_some() {
      panic!("Route path and index cannot be set at the same time");
    }
    self.index = true;
    self
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
    let mut router_map = MatchitRouter::new();

    if let Some(path) = &self.path {
      let path = format!("{}/{}", basename, path);
      router_map.insert(path.clone(), ()).unwrap();
    } else if self.index {
      let path = format!("{}/", basename);
      router_map.insert(path.clone(), ()).unwrap();
    }

    // Recursively build the route map
    for route in self.routes.iter() {
      router_map.merge(route.build_route_map(basename)).unwrap();
    }

    router_map
  }

  pub(crate) fn in_pattern(&self, path: &str) -> bool {
    self.build_route_map("").at(path).is_ok()
  }
}

impl RenderOnce for Route {
  fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
    if let Some(element) = self.element {
      return element;
    }

    let pathname = cx.global::<RouterState>().location.pathname.clone();
    let routes = std::mem::take(&mut self.routes);
    let route = routes.into_iter().find(|route| route.in_pattern(&pathname));
    if let Some(mut layout) = self.layout {
      if let Some(route) = route {
        layout.outlet(route.basename(self.basename).render(window, cx).into_any_element());
      }
      return layout.render_layout(window, cx).into_any_element();
    }
    Empty {}.into_any_element()
  }
}

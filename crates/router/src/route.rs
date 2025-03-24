use crate::RouterState;
use gpui::*;
use matchit::Router as MatchitRouter;
use smallvec::SmallVec;
use std::fmt::{Debug, Display};

pub fn route() -> impl IntoElement {
  Route::new()
}

/// Configures an element to render when a pattern matches the current path.
/// It must be rendered within a [`Routes`](crate::Routes) element.
#[derive(IntoElement)]
pub struct Route {
  basename: SharedString,
  index: bool,
  path: Option<SharedString>,
  pub(crate) element: AnyElement,
  pub(crate) routes: SmallVec<[Box<Route>; 1]>,
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

impl Default for Route {
  fn default() -> Self {
    Self::new()
  }
}

impl Route {
  pub fn new() -> Self {
    Self {
      basename: SharedString::from(""),
      index: false,
      path: None,
      element: Empty {}.into_any_element(),
      routes: SmallVec::new(),
    }
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
    self.element = element.into_any_element();
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

  fn router_maps(&self, basename: &str) -> MatchitRouter<String> {
    let mut router_map = MatchitRouter::new();

    if let Some(path) = &self.path {
      let path = format!("{}/{}", basename, path);
      router_map.insert(path.clone(), path.clone()).unwrap();
    } else if self.index {
      let path = format!("{}/", basename);
      router_map.insert(path.clone(), path.clone()).unwrap();
    }

    // Recursively build the route map
    for route in self.routes.iter() {
      router_map.merge(route.router_maps(basename)).unwrap();
    }

    router_map
  }

  pub(crate) fn in_pattern(&self, path: &str) -> bool {
    self.router_maps("").at(path).is_ok()
  }
}

impl RenderOnce for Route {
  fn render(mut self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let pathname = cx.global::<RouterState>().pathname.clone();
    let route = self.routes.into_iter().find(|route| route.in_pattern(&pathname));
    if let Some(route) = route {
      println!("Route: {:?}", route);
      let element = self.element.downcast_mut::<Div>().unwrap();
      let element = std::mem::replace(element, div());
      return element.child(route.basename(self.basename)).into_any_element();
    }
    println!("Route self: {} {:?}", self.basename, self.path);
    self.element
  }
}

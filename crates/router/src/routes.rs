use crate::Route;
use crate::RouterState;
use gpui::prelude::*;
use gpui::{App, Empty, SharedString, Window};
use matchit::Router as MatchitRouter;
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
  pub fn new() -> Self {
    Self {
      basename: SharedString::from(""),
      routes: SmallVec::new(),
    }
  }

  pub fn basename(mut self, basename: impl Into<SharedString>) -> Self {
    let s: SharedString = basename.into();
    self.basename = SharedString::from(s.trim_end_matches('/').to_owned());
    self
  }

  /// Adds a `Route` as a child to the `Routes`.
  pub fn child(mut self, child: Route) -> Self {
    self.routes.push(child);
    self
  }

  /// Adds multiple `Route`s as children to the `Routes`.
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
}

impl RenderOnce for Routes {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    if cfg!(debug_assertions) && !cx.has_global::<RouterState>() {
      panic!("RouterState not initialized");
    }

    let mut route_map = MatchitRouter::new();
    for route in self.routes.iter() {
      route_map.merge(route.build_route_map(&self.basename)).unwrap();
    }

    let pathname = cx.global::<RouterState>().location.pathname.clone();
    let matched = route_map.at(&pathname);

    if let Ok(matched) = matched {
      for (key, value) in matched.params.iter() {
        cx.global_mut::<RouterState>()
          .params
          .insert(key.to_owned().into(), value.to_owned().into());
      }
      let route = self.routes.into_iter().find(|route| route.in_pattern(&pathname));
      if let Some(route) = route {
        return route.basename(self.basename).into_any_element();
      }
    }

    Empty {}.into_any_element()
  }
}

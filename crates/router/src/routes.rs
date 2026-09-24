use crate::matcher::MatchedRoute;
use crate::{Route, RouterState, normalize_pathname};
use gpui::prelude::*;
use gpui::{App, Empty, SharedString, Window};
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
      basename: SharedString::from("/"),
      routes: SmallVec::new(),
    }
  }

  /// Sets the base path for all child `Route`s.
  pub fn basename(mut self, basename: impl Into<SharedString>) -> Self {
    self.basename = normalize_pathname(basename.into());
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

  pub(crate) fn match_route(&self, pathname: &str) -> Option<MatchedRoute> {
    crate::matcher::match_path(&self.routes, self.basename.as_ref(), pathname)
  }

  pub(crate) fn apply_match(cx: &mut App, pathname: SharedString, matched: Option<&MatchedRoute>) {
    let state = cx.global_mut::<RouterState>();
    state.location.pathname = pathname.clone();

    if let Some(matched) = matched {
      state.params = matched.params.clone();
    } else {
      state.params.clear();
    }

    state.path_match = None;
  }
}

impl RenderOnce for Routes {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    if cfg!(debug_assertions) && !cx.has_global::<RouterState>() {
      panic!("RouterState not initialized");
    }

    let pathname = normalize_pathname(cx.global::<RouterState>().location.pathname.as_ref());
    let matched = self.match_route(pathname.as_ref());
    Self::apply_match(cx, pathname, matched.as_ref());

    if let Some(matched) = matched
      && let Some(route) = self.routes.into_iter().nth(matched.index)
    {
      return route.basename(self.basename).into_any_element();
    }

    Empty {}.into_any_element()
  }
}

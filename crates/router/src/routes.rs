use crate::matcher::MatchedRoute;
use crate::{Route, RouterState, normalize_pathname, normalize_shared_pathname};
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

  #[cfg(test)]
  pub(crate) fn match_route(&self, pathname: &str) -> Option<MatchedRoute> {
    crate::matcher::match_path(&self.routes, self.basename.as_ref(), pathname)
  }

  /// Matches an already-normalized pathname without allocating.
  fn match_normalized(&self, pathname: &str) -> Option<MatchedRoute> {
    crate::matcher::match_normalized(&self.routes, self.basename.as_ref(), pathname)
  }

  /// Writes the match into the global router state, reusing the existing
  /// parameter map so a render pass does not allocate one per frame.
  pub(crate) fn apply_match(cx: &mut App, pathname: SharedString, matched: Option<MatchedRoute>) {
    let state = cx.global_mut::<RouterState>();
    state.location.pathname = pathname;

    state.params.clear();
    if let Some(matched) = matched {
      state.params.extend(matched.params);
    }

    state.path_match = None;
  }
}

impl RenderOnce for Routes {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    if cfg!(debug_assertions) && !cx.has_global::<RouterState>() {
      panic!("RouterState not initialized");
    }

    let pathname = normalize_shared_pathname(&cx.global::<RouterState>().location.pathname);
    let matched = self.match_normalized(pathname.as_ref());
    let index = matched.as_ref().map(|matched| matched.index);
    Self::apply_match(cx, pathname, matched);

    if let Some(index) = index
      && let Some(route) = self.routes.into_iter().nth(index)
    {
      return route.basename(self.basename).into_any_element();
    }

    Empty {}.into_any_element()
  }
}

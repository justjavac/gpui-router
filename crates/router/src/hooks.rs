use crate::{Location, RouterState};
use gpui::{App, SharedString};
use std::sync::LazyLock;

static EMPTY_PARAMS: LazyLock<matchit::Params<'static, 'static>> = LazyLock::new(|| matchit::Params::new());

/// Returns a function that lets you navigate programmatically in response to user interactions or effects.
pub fn use_navigate(cx: &mut App) -> impl FnMut(SharedString) + '_ {
  move |path: SharedString| {
    cx.global_mut::<RouterState>().location.pathname = path;
  }
}

/// Returns the current [Location](crate::Location).
/// This can be useful if you'd like to perform some side effect whenever it changes.
pub fn use_location(cx: &App) -> &Location {
  &cx.global::<RouterState>().location
}

pub fn use_params(cx: &App) -> &matchit::Params<'static, 'static> {
  if let Some(path_match) = cx.global::<RouterState>().path_match.as_ref() {
    &path_match.params
  } else {
    &EMPTY_PARAMS
  }
}

#[cfg(test)]
pub mod tests {
  use super::use_navigate;
  use crate::RouterState;
  use gpui::TestAppContext;

  #[gpui::test]
  async fn test_use_navigate(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);
      assert_eq!(cx.global::<RouterState>().location.pathname, "/");

      {
        let mut navigate = use_navigate(cx);
        navigate("/about".into());
      }
      assert_eq!(cx.global::<RouterState>().location.pathname, "/about");

      {
        let mut navigate = use_navigate(cx);
        navigate("/dashboard".into());
      }
      assert_eq!(cx.global::<RouterState>().location.pathname, "/dashboard");

      {
        let mut navigate = use_navigate(cx);
        navigate("/".into());
      }
      assert_eq!(cx.global::<RouterState>().location.pathname, "/about");
      assert_eq!(cx.global::<RouterState>().location.pathname, "/");

      {
        let mut navigate = use_navigate(cx);
        navigate("/nothing-here".into());
      }
      assert_eq!(cx.global::<RouterState>().location.pathname, "/about");
      assert_eq!(cx.global::<RouterState>().location.pathname, "/nothing-here");
    });
  }
}

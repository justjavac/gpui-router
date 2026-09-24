use crate::{Location, RouterState};
use gpui::{App, SharedString};
use hashbrown::HashMap;

/// Returns a function that lets you navigate programmatically in response to user interactions or effects.
pub fn use_navigate(cx: &mut App) -> impl FnMut(SharedString) + '_ {
  move |path: SharedString| {
    cx.global_mut::<RouterState>().with_path(path);
  }
}

/// Returns the current [Location](crate::Location).
/// This can be useful if you'd like to perform some side effect whenever it changes.
pub fn use_location(cx: &App) -> &Location {
  &cx.global::<RouterState>().location
}

/// Returns the route pattern that matched the current location, if any.
/// For example, `/users/{id}` while the pathname is `/users/42`.
pub fn use_pattern(cx: &App) -> Option<&SharedString> {
  cx.global::<RouterState>().matched_pattern.as_ref()
}

/// Returns the current route parameters as a map of key-value pairs.
/// This is useful for accessing dynamic segments in the route path.
/// For example, if you have a route defined as `/user/{id}`,
/// you can access the `id` parameter using this hook.
pub fn use_params(cx: &App) -> &HashMap<SharedString, SharedString> {
  &cx.global::<RouterState>().params
}

#[cfg(all(test, any(feature = "gpui", feature = "test-support")))]
pub mod tests {
  use super::{use_navigate, use_pattern};
  use crate::{Route, RouterState, Routes, normalize_pathname};
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
      assert_eq!(cx.global::<RouterState>().location.pathname, "/");

      {
        let mut navigate = use_navigate(cx);
        navigate("/nothing-here".into());
      }
      assert_eq!(cx.global::<RouterState>().location.pathname, "/nothing-here");

      {
        let mut navigate = use_navigate(cx);
        navigate("settings/".into());
      }
      assert_eq!(cx.global::<RouterState>().location.pathname, "/settings");

      {
        let mut navigate = use_navigate(cx);
        navigate("".into());
      }
      assert_eq!(cx.global::<RouterState>().location.pathname, "/");
    });
  }

  #[gpui::test]
  async fn test_use_pattern(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      let routes = Routes::new()
        .basename("/")
        .child(Route::new().path("users/{id}").element(|_, _| "user"));

      assert_eq!(use_pattern(cx), None);

      let matched = routes.match_route("/users/42").unwrap();
      Routes::apply_match(cx, normalize_pathname("/users/42"), Some(matched));
      assert_eq!(use_pattern(cx).map(|pattern| pattern.as_ref()), Some("/users/{id}"));
      assert_eq!(
        cx.global::<RouterState>().params.get("id").map(|value| value.as_ref()),
        Some("42")
      );

      Routes::apply_match(cx, normalize_pathname("/missing"), None);
      assert_eq!(use_pattern(cx), None);
      assert!(cx.global::<RouterState>().params.is_empty());
    });
  }
}

use crate::{Location, RouterState};
use gpui::{App, SharedString};
use hashbrown::HashMap;

/// Navigates programmatically, mirroring React Router's `navigate`.
///
/// | React Router | `Navigator` |
/// | --- | --- |
/// | `navigate(to)` | [`Navigator::push`] |
/// | `navigate(to, { replace: true })` | [`Navigator::replace`] |
/// | `navigate(-1)` | [`Navigator::back`] |
/// | `navigate(1)` | [`Navigator::forward`] |
///
/// Mutating the router state does not repaint by itself; the built-in links
/// refresh their window after navigating, and application code should do the
/// same (`window.refresh()`).
pub struct Navigator<'a> {
  state: &'a mut RouterState,
}

impl Navigator<'_> {
  /// Navigates to `to`, like React Router's `navigate(to)`.
  pub fn push(&mut self, to: impl Into<SharedString>) {
    let to = crate::normalize_shared_pathname(&to.into());
    self.state.push_location(Location { pathname: to });
  }

  /// Navigates to `to` without adding a history entry, like
  /// `navigate(to, { replace: true })`.
  pub fn replace(&mut self, to: impl Into<SharedString>) {
    let to = crate::normalize_shared_pathname(&to.into());
    self.state.replace_location(Location { pathname: to });
  }

  /// Moves to the previous history entry, like `navigate(-1)`. Does nothing at
  /// the oldest entry.
  pub fn back(&mut self) {
    let _ = self.state.go_back();
  }

  /// Moves to the next history entry, like `navigate(1)`. Does nothing when the
  /// application has not gone back.
  pub fn forward(&mut self) {
    let _ = self.state.go_forward();
  }
}

/// Returns a [`Navigator`] for programmatic navigation.
pub fn use_navigate(cx: &mut App) -> Navigator<'_> {
  Navigator {
    state: RouterState::require_mut(cx),
  }
}

/// Returns the current [Location](crate::Location).
/// This can be useful if you'd like to perform some side effect whenever it changes.
pub fn use_location(cx: &App) -> &Location {
  &RouterState::require(cx).location
}

/// Returns the route pattern that matched the current location, if any.
/// For example, `/users/{id}` while the pathname is `/users/42`.
pub fn use_pattern(cx: &App) -> Option<&SharedString> {
  RouterState::require(cx).matched_pattern.as_ref()
}

/// Returns the current route parameters as a map of key-value pairs.
/// This is useful for accessing dynamic segments in the route path.
/// For example, if you have a route defined as `/user/{id}`,
/// you can access the `id` parameter using this hook.
pub fn use_params(cx: &App) -> &HashMap<SharedString, SharedString> {
  &RouterState::require(cx).params
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
        navigate.push("/about");
      }
      assert_eq!(cx.global::<RouterState>().location.pathname, "/about");

      {
        let mut navigate = use_navigate(cx);
        navigate.push("/dashboard");
      }
      assert_eq!(cx.global::<RouterState>().location.pathname, "/dashboard");

      {
        let mut navigate = use_navigate(cx);
        navigate.push("/");
      }
      assert_eq!(cx.global::<RouterState>().location.pathname, "/");

      {
        let mut navigate = use_navigate(cx);
        navigate.push("/nothing-here");
      }
      assert_eq!(cx.global::<RouterState>().location.pathname, "/nothing-here");

      {
        let mut navigate = use_navigate(cx);
        navigate.push("settings/");
      }
      assert_eq!(cx.global::<RouterState>().location.pathname, "/settings");

      {
        let mut navigate = use_navigate(cx);
        navigate.push("");
      }
      assert_eq!(cx.global::<RouterState>().location.pathname, "/");
    });
  }

  #[gpui::test]
  async fn test_navigator_history(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      let location = |cx: &gpui::App| cx.global::<RouterState>().location.pathname.clone();

      {
        let mut nav = use_navigate(cx);
        nav.push("/about");
        nav.push("/dashboard");
      }
      assert_eq!(location(cx), "/dashboard");

      {
        let mut nav = use_navigate(cx);
        nav.back();
      }
      assert_eq!(location(cx), "/about");

      {
        let mut nav = use_navigate(cx);
        nav.back();
        nav.back();
      }
      assert_eq!(location(cx), "/", "back stops at the oldest entry");

      {
        let mut nav = use_navigate(cx);
        nav.forward();
      }
      assert_eq!(location(cx), "/about");

      {
        let mut nav = use_navigate(cx);
        nav.push("/settings");
        nav.back();
      }
      assert_eq!(location(cx), "/about", "pushing drops the forward entries");

      {
        let mut nav = use_navigate(cx);
        nav.forward();
      }
      assert_eq!(location(cx), "/settings");

      {
        let mut nav = use_navigate(cx);
        nav.replace("/login");
      }
      assert_eq!(location(cx), "/login");

      {
        let mut nav = use_navigate(cx);
        nav.back();
      }
      assert_eq!(location(cx), "/about", "replace keeps the history length");
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

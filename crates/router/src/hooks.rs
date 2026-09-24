use crate::matcher::matches_pattern;
use crate::{Location, Match, RouterState, SearchParams};
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
    let location = Location::parse(to.into());
    self.state.push_location(location);
  }

  /// Navigates to `to` without adding a history entry, like
  /// `navigate(to, { replace: true })`.
  pub fn replace(&mut self, to: impl Into<SharedString>) {
    let location = Location::parse(to.into());
    self.state.replace_location(location);
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

/// Returns the routes that matched the current location, from the root to the
/// leaf. This is React Router's `useMatches`, which breadcrumbs are usually
/// built from.
pub fn use_matches(cx: &App) -> &[Match] {
  &RouterState::require(cx).matches
}

/// Returns the query string of the current location, parsed. This is the read
/// side of React Router's `useSearchParams`.
pub fn use_search_params(cx: &App) -> &SearchParams {
  &RouterState::require(cx).search_params
}

/// Updates the query string of the current location, which is the setter that
/// React Router's `useSearchParams` returns next to the parameters.
///
/// ```ignore
/// let next = use_search_params(cx).clone().set("page", "2");
/// use_set_search_params(cx).push(next);     // setSearchParams(next)
/// use_set_search_params(cx).replace(next);  // setSearchParams(next, { replace: true })
/// ```
pub struct SearchParamsSetter<'a> {
  state: &'a mut RouterState,
}

impl SearchParamsSetter<'_> {
  /// Navigates to the same location with `params` as its query string, adding a
  /// history entry.
  pub fn push(&mut self, params: SearchParams) {
    let mut location = self.state.location.clone();
    location.search = params.to_search();
    self.state.push_location(location);
  }

  /// Replaces the current history entry with the same location and `params`.
  pub fn replace(&mut self, params: SearchParams) {
    let mut location = self.state.location.clone();
    location.search = params.to_search();
    self.state.replace_location(location);
  }
}

/// Returns a setter for the query string of the current location.
pub fn use_set_search_params(cx: &mut App) -> SearchParamsSetter<'_> {
  SearchParamsSetter {
    state: RouterState::require_mut(cx),
  }
}

/// Matches `pattern` against the current location, like React Router's
/// `useMatch`. Patterns are absolute for now; relative patterns arrive with the
/// relative path work.
pub fn use_match(cx: &App, pattern: &str) -> Option<Match> {
  let state = RouterState::require(cx);

  matches_pattern(pattern, state.location.pathname.as_ref()).then(|| Match {
    pattern: SharedString::from(pattern.to_owned()),
    pathname: state.location.pathname.clone(),
  })
}

#[cfg(all(test, any(feature = "gpui", feature = "test-support")))]
pub mod tests {
  use super::{use_match, use_matches, use_navigate, use_pattern, use_search_params, use_set_search_params};
  use crate::{Route, RouterState, Routes, normalize_pathname};
  use gpui::{SharedString, TestAppContext};

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

  #[gpui::test]
  async fn test_use_match(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      {
        let mut nav = use_navigate(cx);
        nav.push("/users/42");
      }

      let matched = use_match(cx, "users/:id").expect("the pattern matches");
      assert_eq!(matched.pattern, "users/:id");
      assert_eq!(matched.pathname, "/users/42");

      assert!(use_match(cx, "/users/{id}").is_some());
      assert!(use_match(cx, "/groups/:id").is_none());
    });
  }

  #[gpui::test]
  async fn test_use_matches_returns_the_recorded_chain(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      {
        let state = RouterState::require_mut(cx);
        state.record_match(&SharedString::from("/"));
        state.record_match(&SharedString::from("/settings"));
        state.record_match(&SharedString::from("/settings/:section"));
      }

      let patterns: Vec<&str> = use_matches(cx).iter().map(|m| m.pattern.as_ref()).collect();
      assert_eq!(patterns, ["/", "/settings", "/settings/:section"]);
    });
  }

  #[gpui::test]
  async fn test_use_search_params(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      assert!(use_search_params(cx).is_empty());

      {
        let mut nav = use_navigate(cx);
        nav.push("/search?q=rust&tag=gpui#results");
      }

      assert_eq!(super::use_location(cx).pathname, "/search");
      assert_eq!(super::use_location(cx).search, "?q=rust&tag=gpui");
      assert_eq!(super::use_location(cx).hash, "#results");

      let params = use_search_params(cx);
      assert_eq!(params.get("q"), Some("rust"));
      assert_eq!(params.get("tag"), Some("gpui"));
      assert!(params.has("q"));
      assert!(params.get_all("missing").next().is_none());
    });
  }

  #[gpui::test]
  async fn test_use_set_search_params(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      {
        let mut nav = use_navigate(cx);
        nav.push("/search?q=rust");
      }

      let next = use_search_params(cx).clone().set("page", "2");
      {
        let mut set = use_set_search_params(cx);
        set.push(next);
      }
      assert_eq!(super::use_location(cx).search, "?q=rust&page=2");
      assert_eq!(use_search_params(cx).get("page"), Some("2"));

      let history_length = RouterState::require(cx).history.len();
      let next = use_search_params(cx).clone().delete("page");
      {
        let mut set = use_set_search_params(cx);
        set.replace(next);
      }
      assert_eq!(super::use_location(cx).search, "?q=rust");
      assert_eq!(
        RouterState::require(cx).history.len(),
        history_length,
        "replace keeps the history length"
      );
    });
  }
}

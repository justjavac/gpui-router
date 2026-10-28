use crate::matcher::matches_pattern;
use crate::state::resolve_target;
use crate::{Location, Match, NavigationType, Relative, RouterRegistry, RouterState, SearchParams};
use gpui::{App, SharedString, Window};
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
/// Every navigation refreshes the window it was created from, so the routed
/// UI repaints without the caller scheduling a frame.
pub struct Navigator<'a> {
  state: &'a mut RouterState,
  window: &'a mut Window,
  relative: Relative,
  location_state: Option<std::collections::BTreeMap<SharedString, SharedString>>,
}

impl Navigator<'_> {
  /// Resolves the following targets against the current pathname instead of the
  /// current route, like React Router's `relative: "path"`. The default is
  /// [`Relative::Route`].
  pub fn relative(&mut self, relative: Relative) -> &mut Self {
    self.relative = relative;
    self
  }

  /// Attaches data to the following navigations, like React Router's
  /// `navigate(to, { state })`. It is stored on the location and restored by
  /// `back` and `forward`.
  pub fn state<I, K, V>(&mut self, state: I) -> &mut Self
  where
    I: IntoIterator<Item = (K, V)>,
    K: Into<SharedString>,
    V: Into<SharedString>,
  {
    self.location_state = Some(
      state
        .into_iter()
        .map(|(key, value)| (key.into(), value.into()))
        .collect(),
    );
    self
  }

  /// Navigates to `to`, like React Router's `navigate(to)`.
  pub fn push(&mut self, to: impl Into<SharedString>) {
    let target = resolve_target(self.state, to.into().as_ref(), self.relative);
    let mut location = Location::parse(target);
    location.state = self.location_state.clone();
    self.state.push_location(location);
    self.window.refresh();
  }

  /// Navigates to `to` without adding a history entry, like
  /// `navigate(to, { replace: true })`.
  pub fn replace(&mut self, to: impl Into<SharedString>) {
    let target = resolve_target(self.state, to.into().as_ref(), self.relative);
    let mut location = Location::parse(target);
    location.state = self.location_state.clone();
    self.state.replace_location(location);
    self.window.refresh();
  }

  /// Moves to the previous history entry, like `navigate(-1)`. Does nothing at
  /// the oldest entry.
  pub fn back(&mut self) {
    if self.state.go_back() {
      self.window.refresh();
    }
  }

  /// Moves to the next history entry, like `navigate(1)`. Does nothing when the
  /// application has not gone back.
  pub fn forward(&mut self) {
    if self.state.go_forward() {
      self.window.refresh();
    }
  }
}

/// Returns a [`Navigator`] for programmatic navigation.
///
/// Pass the window the navigation belongs to, so the navigator can refresh it
/// after changing the location. In an event handler that window is the one the
/// click happened in; from an async task, update the window first, for example
/// with `window_handle.update(cx, |_, window, cx| use_navigate(window, cx)...)`.
pub fn use_navigate<'a>(window: &'a mut Window, cx: &'a mut App) -> Navigator<'a> {
  let window_id = window.window_handle().window_id();
  let state = cx
    .global_mut::<RouterRegistry>()
    .windows
    .get_mut(&window_id)
    .expect("use_navigate for a window that never rendered a `Routes` tree");

  Navigator {
    state,
    window,
    relative: Relative::Route,
    location_state: None,
  }
}

/// Returns the current [Location](crate::Location), for reads while
/// rendering. This can be useful if you'd like to perform some side effect
/// whenever it changes.
///
/// The location is a snapshot that `Routes` refreshes while it renders: an
/// event handler that navigates still sees the previous location until the
/// next frame renders.
pub fn use_location(cx: &App) -> &Location {
  &RouterState::require(cx).location
}

/// Returns how the current location was reached, mirroring React Router's
/// `useNavigationType`: the initial location, `back` and `forward` are
/// [`NavigationType::Pop`], [`Navigator::push`] is
/// [`NavigationType::Push`] and [`Navigator::replace`] is
/// [`NavigationType::Replace`].
pub fn use_navigation_type(cx: &App) -> NavigationType {
  RouterState::require(cx).navigation_type
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
/// Every update refreshes the window it was created from, so the routed UI
/// repaints without the caller scheduling a frame.
///
/// ```ignore
/// let next = use_search_params(cx).clone().set("page", "2");
/// use_set_search_params(window, cx).push(next);     // setSearchParams(next)
/// use_set_search_params(window, cx).replace(next);  // setSearchParams(next, { replace: true })
/// ```
pub struct SearchParamsSetter<'a> {
  state: &'a mut RouterState,
  window: &'a mut Window,
}

impl SearchParamsSetter<'_> {
  /// Navigates to the same location with `params` as its query string, adding a
  /// history entry.
  pub fn push(&mut self, params: SearchParams) {
    let mut location = self.state.location.clone();
    location.search = params.to_search();
    self.state.push_location(location);
    self.window.refresh();
  }

  /// Replaces the current history entry with the same location and `params`.
  pub fn replace(&mut self, params: SearchParams) {
    let mut location = self.state.location.clone();
    location.search = params.to_search();
    self.state.replace_location(location);
    self.window.refresh();
  }
}

/// Returns a setter for the query string of the current location.
///
/// Pass the window the navigation belongs to, so the setter can refresh it after
/// changing the query string. In an event handler that window is the one the
/// interaction happened in; from an async task, update the window first.
pub fn use_set_search_params<'a>(window: &'a mut Window, cx: &'a mut App) -> SearchParamsSetter<'a> {
  let window_id = window.window_handle().window_id();
  let state = cx
    .global_mut::<RouterRegistry>()
    .windows
    .get_mut(&window_id)
    .expect("use_set_search_params for a window that never rendered a `Routes` tree");

  SearchParamsSetter { state, window }
}

/// Matches `pattern` against the current location, like React Router's
/// `useMatch`. Patterns are absolute for now; relative patterns arrive with the
/// relative path work.
pub fn use_match(cx: &App, pattern: &str) -> Option<Match> {
  let state = RouterState::require(cx);
  let pattern = resolve_target(state, pattern, Relative::Route);

  matches_pattern(pattern.as_ref(), state.location.pathname.as_ref()).then(|| Match {
    pattern: pattern.clone(),
    pathname: state.location.pathname.clone(),
  })
}

#[cfg(all(test, any(feature = "gpui", feature = "test-support")))]
pub mod tests {
  use super::{
    use_match, use_matches, use_navigate, use_navigation_type, use_pattern, use_search_params, use_set_search_params,
  };
  use crate::{NavigationType, Route, RouterState, Routes, normalize_pathname};
  use gpui::{Context, IntoElement, Render, SharedString, TestAppContext, VisualTestContext, Window};

  /// A window that renders an empty `Routes` tree, which is all the Navigator
  /// tests need: it gives them a window whose router state exists.
  struct TestRoutes;

  impl Render for TestRoutes {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
      Routes::new()
    }
  }

  /// Opens a routed window and draws it once, so its router state exists and
  /// the rendering window is bound for the test body's hooks.
  fn open_window(cx: &mut TestAppContext) -> &mut VisualTestContext {
    cx.update(crate::init);
    let (_, visual) = cx.add_window_view(|_, _| TestRoutes);
    visual.update(|window, cx| {
      let _ = window.draw(cx);
    });
    visual
  }

  #[gpui::test]
  async fn test_use_navigate(cx: &mut TestAppContext) {
    let visual = open_window(cx);
    visual.update(|window, cx| {
      assert_eq!(RouterState::global(cx).location.pathname, "/");

      {
        let mut navigate = use_navigate(window, cx);
        navigate.push("/about");
      }
      assert_eq!(RouterState::global(cx).location.pathname, "/about");

      {
        let mut navigate = use_navigate(window, cx);
        navigate.push("/dashboard");
      }
      assert_eq!(RouterState::global(cx).location.pathname, "/dashboard");

      {
        let mut navigate = use_navigate(window, cx);
        navigate.push("/");
      }
      assert_eq!(RouterState::global(cx).location.pathname, "/");

      {
        let mut navigate = use_navigate(window, cx);
        navigate.push("/nothing-here");
      }
      assert_eq!(RouterState::global(cx).location.pathname, "/nothing-here");

      {
        let mut navigate = use_navigate(window, cx);
        navigate.push("settings/");
      }
      assert_eq!(RouterState::global(cx).location.pathname, "/settings");

      {
        let mut navigate = use_navigate(window, cx);
        navigate.push("");
      }
      assert_eq!(RouterState::global(cx).location.pathname, "/");
    });
  }

  #[gpui::test]
  async fn test_navigator_history(cx: &mut TestAppContext) {
    let visual = open_window(cx);
    visual.update(|window, cx| {
      let location = |cx: &gpui::App| RouterState::global(cx).location.pathname.clone();

      {
        let mut nav = use_navigate(window, cx);
        nav.push("/about");
        nav.push("/dashboard");
      }
      assert_eq!(location(cx), "/dashboard");

      {
        let mut nav = use_navigate(window, cx);
        nav.back();
      }
      assert_eq!(location(cx), "/about");

      {
        let mut nav = use_navigate(window, cx);
        nav.back();
        nav.back();
      }
      assert_eq!(location(cx), "/", "back stops at the oldest entry");

      {
        let mut nav = use_navigate(window, cx);
        nav.forward();
      }
      assert_eq!(location(cx), "/about");

      {
        let mut nav = use_navigate(window, cx);
        nav.push("/settings");
        nav.back();
      }
      assert_eq!(location(cx), "/about", "pushing drops the forward entries");

      {
        let mut nav = use_navigate(window, cx);
        nav.forward();
      }
      assert_eq!(location(cx), "/settings");

      {
        let mut nav = use_navigate(window, cx);
        nav.replace("/login");
      }
      assert_eq!(location(cx), "/login");

      {
        let mut nav = use_navigate(window, cx);
        nav.back();
      }
      assert_eq!(location(cx), "/about", "replace keeps the history length");
    });
  }

  #[gpui::test]
  async fn test_use_navigation_type(cx: &mut TestAppContext) {
    let visual = open_window(cx);
    visual.update(|window, cx| {
      assert_eq!(
        use_navigation_type(cx),
        NavigationType::Pop,
        "the initial location is a pop"
      );

      use_navigate(window, cx).push("/about");
      assert_eq!(use_navigation_type(cx), NavigationType::Push);

      use_navigate(window, cx).replace("/dashboard");
      assert_eq!(use_navigation_type(cx), NavigationType::Replace);

      use_navigate(window, cx).back();
      assert_eq!(use_navigation_type(cx), NavigationType::Pop);

      use_navigate(window, cx).forward();
      assert_eq!(use_navigation_type(cx), NavigationType::Pop);
    });
  }

  #[gpui::test]
  async fn test_use_pattern(cx: &mut TestAppContext) {
    let visual = open_window(cx);
    visual.update(|_window, cx| {
      let routes = Routes::new()
        .basename("/")
        .child(Route::new().path("users/{id}").element(|_, _| "user"));

      assert_eq!(use_pattern(cx), None);

      let matched = routes.match_route("/users/42").unwrap();
      Routes::apply_match(cx, normalize_pathname("/users/42"), Some(matched));
      assert_eq!(use_pattern(cx).map(|pattern| pattern.as_ref()), Some("/users/{id}"));
      assert_eq!(
        RouterState::global(cx).params.get("id").map(|value| value.as_ref()),
        Some("42")
      );

      Routes::apply_match(cx, normalize_pathname("/missing"), None);
      assert_eq!(use_pattern(cx), None);
      assert!(RouterState::global(cx).params.is_empty());
    });
  }

  #[gpui::test]
  async fn test_use_match(cx: &mut TestAppContext) {
    let visual = open_window(cx);
    visual.update(|window, cx| {
      {
        let mut nav = use_navigate(window, cx);
        nav.push("/users/42");
      }

      let matched = use_match(cx, "users/:id").expect("the pattern matches");
      // The pattern is reported resolved, like React Router's match.
      assert_eq!(matched.pattern, "/users/:id");
      assert_eq!(matched.pathname, "/users/42");

      assert!(use_match(cx, "/users/{id}").is_some());
      assert!(use_match(cx, "/groups/:id").is_none());
    });
  }

  #[gpui::test]
  async fn test_use_matches_returns_the_recorded_chain(cx: &mut TestAppContext) {
    let visual = open_window(cx);
    visual.update(|window, cx| {
      let _ = window.draw(cx);
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
    let visual = open_window(cx);
    visual.update(|window, cx| {
      assert!(use_search_params(cx).is_empty());

      {
        let mut nav = use_navigate(window, cx);
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
    let visual = open_window(cx);
    visual.update(|window, cx| {
      {
        let mut nav = use_navigate(window, cx);
        nav.push("/search?q=rust");
      }

      let next = use_search_params(cx).clone().set("page", "2");
      {
        let mut set = use_set_search_params(window, cx);
        set.push(next);
      }
      assert_eq!(super::use_location(cx).search, "?q=rust&page=2");
      assert_eq!(use_search_params(cx).get("page"), Some("2"));

      let history_length = RouterState::require(cx).history.len();
      let next = use_search_params(cx).clone().delete("page");
      {
        let mut set = use_set_search_params(window, cx);
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

  #[gpui::test]
  async fn test_navigator_resolves_relative_targets(cx: &mut TestAppContext) {
    let visual = open_window(cx);
    visual.update(|window, cx| {
      let routes = Routes::new().basename("/").child(
        Route::new()
          .path("settings")
          .element(|_, _| "settings")
          .children(vec![Route::new().path("profile").element(|_, _| "profile")]),
      );
      let matched = routes.match_route("/settings/profile").unwrap();
      Routes::apply_match(cx, normalize_pathname("/settings/profile"), Some(matched));

      // A navigator created while rendering resolves against that route.
      {
        let state = RouterState::require_mut(cx);
        state.matches.clear();
        state.matches.push(crate::Match {
          pattern: SharedString::from("/settings"),
          pathname: SharedString::from("/settings"),
        });
        state.matches.push(crate::Match {
          pattern: SharedString::from("/settings/profile"),
          pathname: SharedString::from("/settings/profile"),
        });
        state.current_route = Some(1);
      }

      // Route-relative (the default): a child target resolves against the route.
      {
        let mut nav = use_navigate(window, cx);
        nav.push("billing");
      }
      assert_eq!(super::use_location(cx).pathname, "/settings/profile/billing");

      // `..` climbs one route, not one path segment: the deepest match is
      // `/settings/profile`, so its parent route `/settings` is the target.
      {
        let mut nav = use_navigate(window, cx);
        nav.push("..");
      }
      assert_eq!(super::use_location(cx).pathname, "/settings");

      // Path-relative: `..` climbs one path segment instead.
      {
        let mut nav = use_navigate(window, cx);
        nav.relative(crate::Relative::Path).push("../billing");
      }
      assert_eq!(super::use_location(cx).pathname, "/billing");
    });
  }

  #[gpui::test]
  async fn test_navigator_location_state(cx: &mut TestAppContext) {
    let visual = open_window(cx);
    visual.update(|window, cx| {
      let return_to = |cx: &gpui::App| {
        super::use_location(cx)
          .state
          .as_ref()
          .and_then(|state| state.get("returnTo"))
          .map(|value| value.to_string())
      };

      {
        let mut nav = use_navigate(window, cx);
        nav.state([("returnTo", "/settings")]).push("/login");
      }
      assert_eq!(return_to(cx).as_deref(), Some("/settings"));

      // A navigation without state carries none.
      {
        let mut nav = use_navigate(window, cx);
        nav.push("/about");
      }
      assert_eq!(return_to(cx), None);

      // Going back restores the state of that entry.
      {
        let mut nav = use_navigate(window, cx);
        nav.back();
      }
      assert_eq!(return_to(cx).as_deref(), Some("/settings"));
    });
  }
}

use crate::SearchParams;
use gpui::{App, Global, SharedString};
use hashbrown::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// How a relative navigation target is resolved, mirroring React Router's
/// `relative` option.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Relative {
  /// Resolve against the route that renders the link, which is React Router's
  /// default `relative="route"`. A leading `..` climbs one route.
  #[default]
  Route,
  /// Resolve against the current pathname, like `relative="path"`. A leading
  /// `..` climbs one path segment.
  Path,
}

thread_local! {
  /// The routes whose elements are being laid out, outermost first, so that a
  /// relative target inside a route's element resolves against that route.
  ///
  /// GPUI renders the `RenderOnce` components of an element tree while it lays
  /// the tree out, after the route's element closure returned, so this is a
  /// stack that an element wrapper pushes around the whole subtree instead of a
  /// scope around a single call.
  static RENDER_ROUTES: std::cell::RefCell<Vec<usize>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Keeps `route` on top of the render stack until it is dropped.
pub(crate) struct RenderRouteGuard {
  _private: (),
}

impl Drop for RenderRouteGuard {
  fn drop(&mut self) {
    RENDER_ROUTES.with(|stack| {
      stack.borrow_mut().pop();
    });
  }
}

/// Pushes `route` on top of the render stack.
///
/// The guard has to be held while the whole element subtree is laid out, which
/// is what [`RouteScope`](crate::route) does; a scope around a single call only
/// covers elements that are built eagerly.
pub(crate) fn push_render_route(route: usize) -> RenderRouteGuard {
  RENDER_ROUTES.with(|stack| stack.borrow_mut().push(route));

  RenderRouteGuard { _private: () }
}

/// The route whose element is currently being laid out, if any.
fn current_render_route() -> Option<usize> {
  RENDER_ROUTES.with(|stack| stack.borrow().last().copied())
}

/// Runs `build` with `route` on top of the render stack.
pub(crate) fn with_render_route<R>(route: usize, build: impl FnOnce() -> R) -> R {
  let _scope = push_render_route(route);

  build()
}

/// Resolves a navigation target the way React Router resolves `to`: absolute
/// targets pass through, relative ones resolve against the route that is
/// rendering, or against the deepest match when nothing is rendering, which is
/// what an event handler sees.
pub(crate) fn resolve_target(state: &RouterState, to: &str, relative: Relative) -> SharedString {
  let (path, search, hash) = split_target(to);

  if path.starts_with('/') {
    return SharedString::from(format!("{}{search}{hash}", normalize_pathname(path)));
  }

  let (path, from) = match relative {
    Relative::Path => (path.to_string(), state.location.pathname.to_string()),
    Relative::Route => {
      let mut route = current_render_route().or(state.current_route);
      let mut segments: Vec<&str> = path.split('/').collect();

      while segments.first() == Some(&"..") {
        segments.remove(0);
        route = match route {
          Some(index) if index > 0 => Some(index - 1),
          _ => None,
        };
      }

      let from = route
        .and_then(|index| state.matches.get(index))
        .map(|matched| matched.pathname.to_string())
        .unwrap_or_else(|| "/".to_string());

      (segments.join("/"), from)
    }
  };

  let pathname = normalize_pathname(resolve_from(&path, &from));
  SharedString::from(format!("{pathname}{search}{hash}"))
}

/// Appends a relative path to `from`, popping one segment per `..`, following
/// React Router's `resolvePath`.
fn resolve_from(path: &str, from: &str) -> String {
  let mut segments: Vec<&str> = from.trim_end_matches('/').split('/').collect();

  for segment in path.split('/') {
    match segment {
      "" | "." => {}
      ".." => {
        if segments.len() > 1 {
          segments.pop();
        }
      }
      segment => segments.push(segment),
    }
  }

  if segments.len() > 1 {
    segments.join("/")
  } else {
    "/".to_string()
  }
}

pub(crate) fn normalize_pathname(pathname: impl AsRef<str>) -> SharedString {
  let pathname = pathname.as_ref().trim();
  let mut normalized = if pathname.is_empty() {
    "/".to_string()
  } else if pathname.starts_with('/') {
    pathname.to_string()
  } else {
    format!("/{pathname}")
  };

  while normalized.len() > 1 && normalized.ends_with('/') {
    normalized.pop();
  }

  normalized.into()
}

/// The message every entry point uses when the application forgot to call
/// [`init`](crate::init).
const NOT_INITIALIZED: &str =
  "the router is not initialized: call `gpui_router::init(cx)` once while starting the application";

/// How many locations the history keeps. Older entries are dropped, so `back`
/// walks the most recent ones.
const MAX_HISTORY: usize = 100;

/// Source of the unique keys history entries carry, like React Router's
/// `location.key`.
static NEXT_LOCATION_KEY: AtomicU64 = AtomicU64::new(1);

fn next_location_key() -> SharedString {
  SharedString::from(NEXT_LOCATION_KEY.fetch_add(1, Ordering::Relaxed).to_string())
}

/// Normalizes a shared pathname without allocating when it is already in the
/// form the router stores.
pub(crate) fn normalize_shared_pathname(pathname: &SharedString) -> SharedString {
  let value = pathname.as_ref();

  if is_normalized_pathname(value) {
    pathname.clone()
  } else {
    normalize_pathname(value)
  }
}

/// A normalized pathname starts with a single `/`, has no trailing `/` (except
/// the root) and carries no surrounding whitespace.
fn is_normalized_pathname(pathname: &str) -> bool {
  pathname.starts_with('/')
    && pathname.trim() == pathname
    && (pathname.len() == 1 || (!pathname.ends_with('/') && !pathname.ends_with(' ')))
}

/// A Location represents a URL-like location in the router.
#[derive(PartialEq, Eq, Ord, PartialOrd, Clone, Debug)]
pub struct Location {
  /// A URL pathname, beginning with a `/`.
  pub pathname: SharedString,
  /// The query string, including the `?`, or empty. Routes only match
  /// [`Location::pathname`].
  pub search: SharedString,
  /// The fragment, including the `#`, or empty.
  pub hash: SharedString,
  /// A key that is unique per history entry, like React Router's
  /// `location.key`. The initial location uses `"default"`.
  pub key: SharedString,
  /// Data carried by the navigation that produced this location, like React
  /// Router's `location.state`. A sorted map keeps `Location` orderable.
  pub state: Option<std::collections::BTreeMap<SharedString, SharedString>>,
}

impl Location {
  /// Parses a navigation target the way React Router parses `to`: everything
  /// before `?` or `#` is the pathname that routes match.
  pub(crate) fn parse(to: impl AsRef<str>) -> Self {
    let (pathname, search, hash) = split_target(to.as_ref());

    Self {
      pathname: normalize_pathname(pathname),
      search,
      hash,
      key: SharedString::from("default"),
      state: None,
    }
  }
}

/// A route that matched the current location, from the root to the leaf.
///
/// [`use_matches`](crate::use_matches) returns the chain an application needs
/// for breadcrumbs, mirroring React Router's `useMatches`.
#[derive(PartialEq, Eq, Ord, PartialOrd, Clone, Debug)]
pub struct Match {
  /// The route pattern, for example `/users/:id`.
  pub pattern: SharedString,
  /// The pathname that pattern matched, for example `/users/42`.
  pub pathname: SharedString,
}

/// Builds the concrete pathname a pattern matched by substituting the dynamic
/// segments with their parameter values.
pub(crate) fn concrete_pathname(pattern: &str, params: &HashMap<SharedString, SharedString>) -> SharedString {
  let mut pathname = String::with_capacity(pattern.len());

  for (index, segment) in pattern.split('/').enumerate() {
    if index > 0 {
      pathname.push('/');
    }

    let name = segment
      .strip_prefix("{*")
      .and_then(|rest| rest.strip_suffix('}'))
      .or_else(|| segment.strip_prefix(':').map(|rest| rest.trim_end_matches('}')))
      .or_else(|| segment.strip_prefix('{').and_then(|rest| rest.strip_suffix('}')));

    match name.and_then(|name| params.get(name)) {
      Some(value) => pathname.push_str(value),
      None => pathname.push_str(segment),
    }
  }

  SharedString::from(if pathname.is_empty() { "/".to_string() } else { pathname })
}

impl Default for Location {
  /// Creates a default Location with pathname `/`.
  fn default() -> Self {
    Self {
      pathname: normalize_pathname("/"),
      search: SharedString::default(),
      hash: SharedString::default(),
      key: SharedString::from("default"),
      state: None,
    }
  }
}

/// Splits a target into its pathname, query string and fragment. The query and
/// the fragment keep their leading `?` and `#`.
pub(crate) fn split_target(to: &str) -> (&str, SharedString, SharedString) {
  let (rest, hash) = match to.split_once('#') {
    Some((rest, hash)) => (rest, SharedString::from(format!("#{hash}"))),
    None => (to, SharedString::default()),
  };
  let (pathname, search) = match rest.split_once('?') {
    Some((pathname, search)) => (pathname, SharedString::from(format!("?{search}"))),
    None => (rest, SharedString::default()),
  };

  (pathname, search, hash)
}

/// The global state of the router: the current location, the route pattern that
/// matched it, and the dynamic parameters of that match.
///
/// This state is stored globally within the GPUI application context, so an
/// application renders one `Routes` tree per window.
#[derive(PartialEq, Clone)]
pub struct RouterState {
  /// The current location in the router.
  pub location: Location,
  /// The route pattern that matched the current location, if any.
  /// For example `/users/{id}` for the pathname `/users/42`.
  pub matched_pattern: Option<SharedString>,
  /// The dynamic parameters for the current location.
  pub params: HashMap<SharedString, SharedString>,
  /// The query string of the current location, parsed. Kept in sync with
  /// [`Location::search`] whenever the location changes.
  pub search_params: SearchParams,
  /// The routes that matched the current location, from the root to the leaf.
  /// Filled while the router renders, like React Router's `useMatches`.
  pub matches: Vec<Match>,
  /// Index into [`RouterState::matches`] of the deepest route that rendered,
  /// which relative navigation from an event handler resolves against.
  pub current_route: Option<usize>,
  /// Locations visited before and after the current one, oldest first.
  pub history: Vec<Location>,
  /// Index of [`RouterState::location`] inside
  /// [`RouterState::history`].
  pub history_index: usize,
}

impl Global for RouterState {}

impl RouterState {
  /// Initializes the RouterState within the GPUI application context.
  /// This function sets up the initial state of the router.
  pub fn init(cx: &mut App) {
    let location = Location::default();
    let state = Self {
      location: location.clone(),
      matched_pattern: None,
      params: HashMap::new(),
      search_params: SearchParams::default(),
      matches: Vec::new(),
      current_route: None,
      history: vec![location],
      history_index: 0,
    };
    cx.set_global::<RouterState>(state);
  }

  /// Sets the current pathname in the router state, replacing the current
  /// history entry. Prefer [`use_navigate`](crate::use_navigate), which also
  /// records history.
  pub fn with_path(&mut self, pathname: SharedString) -> &mut Self {
    let location = Location::parse(pathname);
    self.replace_location(location);
    self
  }

  /// Navigates to a new location, keeping the previous one in the history.
  pub(crate) fn push_location(&mut self, mut location: Location) {
    location.key = next_location_key();
    self.history.truncate(self.history_index + 1);
    self.history.push(location.clone());

    if self.history.len() > MAX_HISTORY {
      let oldest = self.history.len() - MAX_HISTORY;
      self.history.drain(..oldest);
    }

    self.history_index = self.history.len() - 1;
    self.set_location(location);
  }

  /// Records a route of the chain that is being rendered, which is what
  /// [`use_matches`](crate::use_matches) returns.
  pub(crate) fn record_match(&mut self, pattern: &SharedString) -> usize {
    let pathname = concrete_pathname(pattern.as_ref(), &self.params);
    self.matches.push(Match {
      pattern: pattern.clone(),
      pathname,
    });

    let index = self.matches.len() - 1;
    self.current_route = Some(index);
    index
  }

  /// Navigates to a location, replacing the current history entry.
  pub(crate) fn replace_location(&mut self, mut location: Location) {
    location.key = next_location_key();

    match self.history.get_mut(self.history_index) {
      Some(current) => *current = location.clone(),
      None => {
        self.history.push(location.clone());
        self.history_index = self.history.len() - 1;
      }
    }

    self.set_location(location);
  }

  /// Makes `location` current and keeps the parsed query string in sync.
  fn set_location(&mut self, location: Location) {
    self.search_params = SearchParams::parse(location.search.as_ref());
    self.location = location;
  }

  /// Moves to the previous history entry, if there is one.
  pub(crate) fn go_back(&mut self) -> bool {
    if self.history_index == 0 {
      return false;
    }

    self.history_index -= 1;
    let location = self.history[self.history_index].clone();
    self.set_location(location);
    true
  }

  /// Moves to the next history entry, if the application went back earlier.
  pub(crate) fn go_forward(&mut self) -> bool {
    if self.history_index + 1 >= self.history.len() {
      return false;
    }

    self.history_index += 1;
    let location = self.history[self.history_index].clone();
    self.set_location(location);
    true
  }

  /// Retrieves an immutable reference to the global RouterState from the GPUI application context.
  pub fn global(cx: &App) -> &Self {
    cx.global::<Self>()
  }

  /// Retrieves a mutable reference to the global RouterState from the GPUI application context.
  pub fn global_mut(cx: &mut App) -> &mut Self {
    cx.global_mut::<Self>()
  }

  /// Returns the router state, panicking with the fix when an application never
  /// called [`init`](crate::init).
  pub(crate) fn require(cx: &App) -> &Self {
    cx.try_global::<Self>().unwrap_or_else(|| panic!("{NOT_INITIALIZED}"))
  }

  /// Mutable counterpart of [`RouterState::require`].
  pub(crate) fn require_mut(cx: &mut App) -> &mut Self {
    if !cx.has_global::<Self>() {
      panic!("{NOT_INITIALIZED}");
    }

    cx.global_mut::<Self>()
  }
}

#[cfg(test)]
mod tests {
  use super::{Location, MAX_HISTORY, Relative, RouterState, normalize_pathname, resolve_target, with_render_route};
  use gpui::SharedString;

  impl RouterState {
    /// A state with the default location and empty everything else.
    fn default_for_test() -> Self {
      Self {
        location: Location::default(),
        matched_pattern: None,
        params: Default::default(),
        search_params: Default::default(),
        matches: Vec::new(),
        current_route: None,
        history: vec![Location::default()],
        history_index: 0,
      }
    }
  }

  #[test]
  fn test_normalize_pathname_handles_empty_relative_and_trailing_slashes() {
    assert_eq!(normalize_pathname(""), "/");
    assert_eq!(normalize_pathname("about"), "/about");
    assert_eq!(normalize_pathname("/about/"), "/about");
    assert_eq!(normalize_pathname("  /about/team/  "), "/about/team");
  }

  #[test]
  fn test_normalize_pathname_preserves_root() {
    assert_eq!(normalize_pathname("/"), "/");
    assert_eq!(normalize_pathname("////"), "/");
  }

  #[test]
  fn test_normalize_shared_pathname_keeps_normalized_values() {
    use super::normalize_shared_pathname;
    use gpui::SharedString;

    assert_eq!(
      normalize_shared_pathname(&SharedString::from("/about")).as_ref(),
      "/about"
    );
    assert_eq!(
      normalize_shared_pathname(&SharedString::from("about/")).as_ref(),
      "/about"
    );
    assert_eq!(
      normalize_shared_pathname(&SharedString::from("  /about  ")).as_ref(),
      "/about"
    );
    assert_eq!(normalize_shared_pathname(&SharedString::from("/")).as_ref(), "/");
    assert_eq!(normalize_shared_pathname(&SharedString::from("////")).as_ref(), "/");
  }

  #[test]
  fn test_router_state_with_path_normalizes_pathname() {
    let mut state = RouterState {
      location: Location::default(),
      matched_pattern: None,
      params: Default::default(),
      search_params: Default::default(),
      matches: Vec::new(),
      current_route: None,
      history: vec![Location::default()],
      history_index: 0,
    };

    state.with_path("dashboard/".into());
    assert_eq!(state.location.pathname, "/dashboard");

    state.with_path("".into());
    assert_eq!(state.location.pathname, "/");

    assert_eq!(
      state
        .history
        .get(state.history_index)
        .map(|location| &location.pathname),
      Some(&state.location.pathname),
      "with_path replaces the current history entry"
    );
  }

  #[test]
  fn test_history_keeps_the_most_recent_entries() {
    let mut state = RouterState {
      location: Location::default(),
      matched_pattern: None,
      params: Default::default(),
      search_params: Default::default(),
      matches: Vec::new(),
      current_route: None,
      history: vec![Location::default()],
      history_index: 0,
    };

    for page in 0..MAX_HISTORY + 4 {
      state.push_location(Location::parse(format!("/page{page}")));
    }

    assert_eq!(state.history.len(), MAX_HISTORY);
    assert_eq!(state.history_index, MAX_HISTORY - 1);
    assert_eq!(state.location.pathname, SharedString::from("/page103"));

    while state.go_back() {}

    assert_eq!(state.history_index, 0);
    assert_eq!(
      state.location.pathname,
      SharedString::from("/page4"),
      "the oldest entries are dropped once the cap is reached"
    );
  }

  #[test]
  fn test_location_parse_splits_the_target() {
    let location = Location::parse("/users/42?tab=posts&page=2#top");
    assert_eq!(location.pathname, "/users/42");
    assert_eq!(location.search, "?tab=posts&page=2");
    assert_eq!(location.hash, "#top");
    assert_eq!(location.key, "default");

    let location = Location::parse("settings/");
    assert_eq!(location.pathname, "/settings");
    assert_eq!(location.search, "");
    assert_eq!(location.hash, "");

    let location = Location::parse("/docs#section");
    assert_eq!(location.pathname, "/docs");
    assert_eq!(location.search, "");
    assert_eq!(location.hash, "#section");
  }

  #[test]
  fn test_push_location_assigns_unique_keys_and_parses_search() {
    let mut state = RouterState {
      location: Location::default(),
      matched_pattern: None,
      params: Default::default(),
      search_params: Default::default(),
      matches: Vec::new(),
      current_route: None,
      history: vec![Location::default()],
      history_index: 0,
    };

    state.push_location(Location::parse("/search?q=rust"));
    let first_key = state.location.key.clone();
    assert_eq!(state.search_params.get("q"), Some("rust"));

    state.push_location(Location::parse("/search?q=gpui"));
    assert_ne!(state.location.key, first_key, "every entry gets its own key");
    assert_ne!(state.location.key, "default");
    assert_eq!(state.search_params.get("q"), Some("gpui"));

    state.go_back();
    assert_eq!(state.search_params.get("q"), Some("rust"));
    assert_eq!(state.location.key, first_key);
  }

  #[test]
  fn test_resolve_target_handles_absolute_targets() {
    let state = RouterState::default_for_test();

    assert_eq!(resolve_target(&state, "/about", Relative::Route), "/about");
    assert_eq!(
      resolve_target(&state, "/search?q=1#top", Relative::Path),
      "/search?q=1#top"
    );
    assert_eq!(resolve_target(&state, "about/", Relative::Route), "/about");
  }

  #[test]
  fn test_resolve_target_route_relative() {
    let mut state = RouterState::default_for_test();
    {
      let base = &mut state;
      base.record_match(&SharedString::from("/"));
      base.record_match(&SharedString::from("/settings"));
    }
    state.location.pathname = SharedString::from("/settings/profile");

    // The deepest match is the base when nothing is rendering.
    assert_eq!(resolve_target(&state, "profile", Relative::Route), "/settings/profile");
    assert_eq!(resolve_target(&state, "", Relative::Route), "/settings");

    // `..` climbs a route, not a path segment.
    assert_eq!(
      resolve_target(&state, "../permissions", Relative::Route),
      "/permissions"
    );
    assert_eq!(resolve_target(&state, "..", Relative::Route), "/");
    assert_eq!(
      resolve_target(&state, "../../above", Relative::Route),
      "/above",
      "climbing above the root stays at the root"
    );

    // While rendering, the route being rendered wins.
    assert_eq!(
      with_render_route(0, || resolve_target(&state, "profile", Relative::Route)),
      "/profile"
    );
  }

  #[test]
  fn test_resolve_target_path_relative() {
    let mut state = RouterState::default_for_test();
    state.location.pathname = SharedString::from("/settings/profile");

    assert_eq!(
      resolve_target(&state, "../billing", Relative::Path),
      "/settings/billing"
    );
    assert_eq!(
      resolve_target(&state, "billing", Relative::Path),
      "/settings/profile/billing"
    );
    assert_eq!(resolve_target(&state, "../../", Relative::Path), "/");
  }
}

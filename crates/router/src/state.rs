use gpui::{App, Global, SharedString};
use hashbrown::HashMap;

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
    }
  }
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
  /// The routes that matched the current location, from the root to the leaf.
  /// Filled while the router renders, like React Router's `useMatches`.
  pub matches: Vec<Match>,
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
      matches: Vec::new(),
      history: vec![location],
      history_index: 0,
    };
    cx.set_global::<RouterState>(state);
  }

  /// Sets the current pathname in the router state, replacing the current
  /// history entry. Prefer [`use_navigate`](crate::use_navigate), which also
  /// records history.
  pub fn with_path(&mut self, pathname: SharedString) -> &mut Self {
    self.replace_location(Location {
      pathname: normalize_pathname(pathname),
    });
    self
  }

  /// Navigates to a new location, keeping the previous one in the history.
  pub(crate) fn push_location(&mut self, location: Location) {
    self.history.truncate(self.history_index + 1);
    self.history.push(location.clone());

    if self.history.len() > MAX_HISTORY {
      let oldest = self.history.len() - MAX_HISTORY;
      self.history.drain(..oldest);
    }

    self.history_index = self.history.len() - 1;
    self.location = location;
  }

  /// Records a route of the chain that is being rendered, which is what
  /// [`use_matches`](crate::use_matches) returns.
  pub(crate) fn record_match(&mut self, pattern: &SharedString) {
    let pathname = concrete_pathname(pattern.as_ref(), &self.params);
    self.matches.push(Match {
      pattern: pattern.clone(),
      pathname,
    });
  }

  /// Navigates to a location, replacing the current history entry.
  pub(crate) fn replace_location(&mut self, location: Location) {
    match self.history.get_mut(self.history_index) {
      Some(current) => *current = location.clone(),
      None => {
        self.history.push(location.clone());
        self.history_index = self.history.len() - 1;
      }
    }

    self.location = location;
  }

  /// Moves to the previous history entry, if there is one.
  pub(crate) fn go_back(&mut self) -> bool {
    if self.history_index == 0 {
      return false;
    }

    self.history_index -= 1;
    self.location = self.history[self.history_index].clone();
    true
  }

  /// Moves to the next history entry, if the application went back earlier.
  pub(crate) fn go_forward(&mut self) -> bool {
    if self.history_index + 1 >= self.history.len() {
      return false;
    }

    self.history_index += 1;
    self.location = self.history[self.history_index].clone();
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
  use super::{Location, MAX_HISTORY, RouterState, normalize_pathname};
  use gpui::SharedString;

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
      matches: Vec::new(),
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
      matches: Vec::new(),
      history: vec![Location::default()],
      history_index: 0,
    };

    for page in 0..MAX_HISTORY + 4 {
      state.push_location(Location {
        pathname: normalize_pathname(format!("/page{page}")),
      });
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
}

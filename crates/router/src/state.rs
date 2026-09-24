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
}

impl Global for RouterState {}

impl RouterState {
  /// Initializes the RouterState within the GPUI application context.
  /// This function sets up the initial state of the router.
  pub fn init(cx: &mut App) {
    let state = Self {
      location: Location::default(),
      matched_pattern: None,
      params: HashMap::new(),
    };
    cx.set_global::<RouterState>(state);
  }

  /// Sets the current pathname in the router state.
  pub fn with_path(&mut self, pathname: SharedString) -> &mut Self {
    self.location.pathname = normalize_pathname(pathname);
    self
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
  use super::{Location, RouterState, normalize_pathname};

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
    };

    state.with_path("dashboard/".into());
    assert_eq!(state.location.pathname, "/dashboard");

    state.with_path("".into());
    assert_eq!(state.location.pathname, "/");
  }
}

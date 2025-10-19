use gpui::{App, Global, SharedString};
use hashbrown::HashMap;
use matchit::Params;

/// A Location represents a URL-like location in the router.
/// It contains a pathname and an optional state object.
#[derive(PartialEq, Eq, Ord, PartialOrd, Clone, Debug)]
pub struct Location {
  /// A URL pathname, beginning with a `/`.
  pub pathname: SharedString,
  /// A value of arbitrary data associated with this location.
  pub state: Params<'static, 'static>,
}

impl Default for Location {
  /// Creates a default Location with pathname `/` and empty state.
  fn default() -> Self {
    Self {
      pathname: "/".into(),
      state: Params::default(),
    }
  }
}

/// A PathMatch contains info about how a PathPattern matched on a URL-like pathname.
#[derive(PartialEq, Eq, Ord, PartialOrd, Clone, Debug)]
pub struct PathMatch {
  /// The portion of the URL-like pathname that was matched.
  pub pathname: SharedString,
  /// The portion of the URL-like pathname that was matched before child routes.
  pub pathname_base: SharedString,
  /// The route pattern that was matched.
  pub pattern: SharedString,
  /// The names and values of dynamic parameters in the URL-like.
  /// For example, if the route pattern is `/users/{id}`, and the URL pathname is `/users/123`,
  /// then the `params` would be `{"id": "123"}`.
  pub params: Params<'static, 'static>,
}

/// The global state of the router, including the current location, path match, and parameters.
/// This state is stored globally within the GPUI application context.
#[derive(PartialEq, Clone)]
pub struct RouterState {
  /// The current location in the router.
  pub location: Location,
  /// The path match information for the current location.
  pub path_match: Option<PathMatch>,
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
      path_match: None,
      params: HashMap::new(),
    };
    cx.set_global::<RouterState>(state);
  }

  /// Sets the current pathname in the router state.
  pub fn with_path(&mut self, pathname: SharedString) -> &mut Self {
    self.location.pathname = pathname;
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
}

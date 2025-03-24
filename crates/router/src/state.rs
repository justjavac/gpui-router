use gpui::{App, Global, SharedString};
use hashbrown::HashMap;
use matchit::Params;

#[derive(PartialEq, Eq, Ord, PartialOrd, Clone, Debug)]
pub struct Location {
  /// A URL pathname, beginning with a `/`.
  pub pathname: SharedString,
  /// A value of arbitrary data associated with this location.
  pub state: Params<'static, 'static>,
}

impl Default for Location {
  fn default() -> Self {
    Self {
      pathname: "/".into(),
      state: Params::default(),
    }
  }
}

// A PathMatch contains info about how a PathPattern matched on a URL-like pathname.
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

#[derive(PartialEq, Clone)]
pub struct RouterState {
  pub location: Location,
  pub path_match: Option<PathMatch>,
  pub params: HashMap<SharedString, SharedString>,
}

impl Global for RouterState {}

impl RouterState {
  pub fn init(cx: &mut App) {
    let state = Self {
      location: Location::default(),
      path_match: None,
      params: HashMap::new(),
    };
    cx.set_global::<RouterState>(state);
  }

  pub fn with_path(&mut self, pathname: SharedString) -> &mut Self {
    self.location.pathname = pathname;
    self
  }

  pub fn global(cx: &App) -> &Self {
    cx.global::<Self>()
  }

  pub fn global_mut(cx: &mut App) -> &mut Self {
    cx.global_mut::<Self>()
  }
}

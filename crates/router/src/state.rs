use gpui::{App, Global, SharedString};
use matchit::Params;

#[derive(PartialEq, Eq, Ord, PartialOrd, Clone)]
pub struct Location {
  /// A URL pathname, beginning with a `/`.
  pub pathname: SharedString,
  /// A value of arbitrary data associated with this location.
  pub state: Params<'static, 'static>,
}

#[derive(PartialEq, Eq, Ord, PartialOrd, Clone)]
pub struct Match {
  /// The portion of the URL pathname that was matched.
  pub pathname: SharedString,
  /// The portion of the URL pathname that was matched before child routes.
  pub pathname_base: SharedString,
  /// The route pattern that was matched.
  pub pattern: SharedString,
  /// The names and values of dynamic parameters in the URL.
  /// For example, if the route pattern is `/users/{id}`, and the URL pathname is `/users/123`,
  /// then the `params` would be `{"id": "123"}`.
  pub params: Params<'static, 'static>,
}

impl Default for Location {
  fn default() -> Self {
    Self {
      pathname: "/".into(),
      state: Params::default(),
    }
  }
}

#[derive(PartialEq, Eq, Ord, PartialOrd, Clone)]
pub struct RouterState {
  pub pathname: SharedString,
  pub location: Location,
}

impl Global for RouterState {}

impl RouterState {
  pub fn init(cx: &mut App) {
    let state = Self {
      pathname: "/".into(),
      location: Location::default(),
    };
    cx.set_global::<RouterState>(state);
  }

  pub fn global(cx: &App) -> &Self {
    cx.global::<Self>()
  }

  pub fn global_mut(cx: &mut App) -> &mut Self {
    cx.global_mut::<Self>()
  }
}

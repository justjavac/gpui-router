use crate::{Location, RouterState, use_navigate};
use gpui::*;

/// Navigates to another location while it renders, like React Router's
/// `<Navigate>`.
///
/// ```ignore
/// fn guard(logged_in: bool) -> impl IntoElement {
///   if logged_in { home() } else { Redirect::to("/login").replace(true) }
/// }
/// ```
///
/// A redirect navigates when the current location is not already its target, so
/// a re-render cannot navigate twice.
#[derive(IntoElement)]
pub struct Redirect {
  to: SharedString,
  replace: bool,
  state: Option<std::collections::BTreeMap<SharedString, SharedString>>,
}

impl Redirect {
  /// Creates a redirect to `to`, which may carry a query string and a fragment.
  pub fn to(to: impl Into<SharedString>) -> Self {
    Self {
      to: to.into(),
      replace: false,
      state: None,
    }
  }

  /// When `true`, the redirect replaces the current history entry instead of
  /// adding one. This is `<Navigate replace />`.
  pub fn replace(mut self, replace: bool) -> Self {
    self.replace = replace;
    self
  }

  /// Attaches data to the navigation, like React Router's `state` prop on
  /// `<Navigate>`.
  pub fn state<I, K, V>(mut self, state: I) -> Self
  where
    I: IntoIterator<Item = (K, V)>,
    K: Into<SharedString>,
    V: Into<SharedString>,
  {
    self.state = Some(
      state
        .into_iter()
        .map(|(key, value)| (key.into(), value.into()))
        .collect(),
    );
    self
  }
}

impl RenderOnce for Redirect {
  fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
    let target = Location::parse(self.to.as_ref());
    let already_there = {
      let current = &RouterState::require(cx).location;
      current.pathname == target.pathname && current.search == target.search && current.hash == target.hash
    };

    if !already_there {
      let mut navigate = use_navigate(cx);
      if let Some(state) = self.state {
        navigate.state(state);
      }
      if self.replace {
        navigate.replace(self.to);
      } else {
        navigate.push(self.to);
      }
      // The location changed while this frame renders, so ask for another one.
      window.refresh();
    }

    Empty {}.into_any_element()
  }
}

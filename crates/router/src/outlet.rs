use gpui::*;

/// An outlet is a placeholder in the UI where routed components will be rendered.
pub fn outlet() -> impl IntoElement {
  Outlet::new()
}

/// A placeholder element for routed components.
/// When no component is routed to this outlet, it renders as an empty element.
#[derive(IntoElement)]
pub struct Outlet {
  pub(crate) element: AnyElement,
}

impl Default for Outlet {
  fn default() -> Self {
    Outlet {
      element: Empty {}.into_any_element(),
    }
  }
}

impl Outlet {
  pub fn new() -> Self {
    Default::default()
  }
}

impl From<AnyElement> for Outlet {
  fn from(element: AnyElement) -> Outlet {
    Outlet { element }
  }
}

impl RenderOnce for Outlet {
  fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    self.element
  }
}

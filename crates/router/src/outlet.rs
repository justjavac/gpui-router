use gpui::*;

pub fn outlet() -> impl IntoElement {
  Outlet::new()
}

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

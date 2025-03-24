use gpui::*;

pub fn outlet() -> impl IntoElement {
  Outlet::new()
}

#[derive(IntoElement, Default)]
pub struct Outlet {}

impl Outlet {
  pub fn new() -> Self {
    Default::default()
  }
}

impl RenderOnce for Outlet {
  fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    div()
  }
}

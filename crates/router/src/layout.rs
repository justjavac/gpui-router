use gpui::{AnyElement, App, Window};

pub trait Layout {
  fn outlet(&mut self, element: AnyElement);
  fn render_layout(self: Box<Self>, window: &mut Window, cx: &mut App) -> AnyElement;
}

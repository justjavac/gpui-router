use gpui::{AnyElement, App, Window};

/// A layout that can wrap around routed elements.
/// Used by [`Router`](crate::Router) to render layouts around matched routes.
pub trait Layout {
  /// Sets the outlet element that the layout should render its children into.
  fn outlet(&mut self, element: AnyElement);
  /// Renders the layout with the given outlet element.
  fn render_layout(self: Box<Self>, window: &mut Window, cx: &mut App) -> AnyElement;
}

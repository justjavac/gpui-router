use gpui::*;
use smallvec::SmallVec;

pub fn router() -> impl IntoElement {
  Router::new()
}

#[derive(IntoElement, Default)]
pub struct Router {
  children: SmallVec<[AnyElement; 1]>,
}

impl Router {
  pub fn new() -> Self {
    Default::default()
  }
}

impl ParentElement for Router {
  fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
    self.children.extend(elements);
  }
}

impl RenderOnce for Router {
  fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    div().children(self.children)
  }
}

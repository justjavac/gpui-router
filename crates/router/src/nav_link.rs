use crate::use_navigate;
use gpui::*;
use smallvec::SmallVec;

pub fn nav_link() -> impl IntoElement {
  NavLink::new().active(|style| style)
}

#[derive(IntoElement)]
pub struct NavLink {
  base: Div,
  children: SmallVec<[AnyElement; 1]>,
  to: SharedString,
  // is_active: bool,
}

impl Default for NavLink {
  fn default() -> Self {
    Self {
      base: div(),
      children: Default::default(),
      to: Default::default(),
    }
  }
}

impl Styled for NavLink {
  fn style(&mut self) -> &mut StyleRefinement {
    self.base.style()
  }
}

impl ParentElement for NavLink {
  fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
    self.children.extend(elements);
  }
}

impl InteractiveElement for NavLink {
  fn interactivity(&mut self) -> &mut gpui::Interactivity {
      self.base.interactivity()
  }
}

impl NavLink {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn to(mut self, to: impl Into<SharedString>) -> Self {
    self.to = to.into();
    self
  }

  pub fn active(self, _f: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self {
    unimplemented!()
  }
}

impl RenderOnce for NavLink {
  fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    self
      .base
      .id(ElementId::from(self.to.clone()))
      .on_click(move |_, window, cx| {
        let mut navigate = use_navigate(cx);
        navigate(self.to.clone());
        window.refresh();
      })
      .children(self.children)
  }
}

use crate::use_navigate;
use gpui::*;

pub fn nav_link() -> impl IntoElement {
  NavLink::new().active(|style| style)
}

#[derive(IntoElement, Default)]
pub struct NavLink {
  child: Option<AnyElement>,
  to: SharedString,
  // is_active: bool,
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

  pub fn child(mut self, child: impl IntoElement) -> Self {
    self.child = Some(child.into_any_element());
    self
  }
}

impl RenderOnce for NavLink {
  fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    if let Some(child) = self.child {
      div()
        .id(ElementId::from(self.to.clone()))
        .child(child)
        .on_click(move |_, window, cx| {
          let mut navigate = use_navigate(cx);
          navigate(self.to.clone());
          window.refresh();
        })
        .into_any_element()
    } else {
      Empty {}.into_any_element()
    }
  }
}

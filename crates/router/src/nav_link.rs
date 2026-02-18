use crate::{RouterState, use_navigate};
use gpui::*;
use smallvec::SmallVec;

/// A navigation link that changes the route when clicked.
pub fn nav_link() -> impl IntoElement {
  NavLink::new().active(|style| style)
}

/// A navigation link that changes the route when clicked.
#[derive(IntoElement)]
pub struct NavLink {
  base: Div,
  children: SmallVec<[AnyElement; 1]>,
  to: SharedString,
  active_style: Option<Box<StyleRefinement>>,
  end: bool,
}

impl Default for NavLink {
  fn default() -> Self {
    Self {
      base: div(),
      children: Default::default(),
      to: Default::default(),
      active_style: None,
      end: false,
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

  /// Sets the destination route for the navigation link.
  pub fn to(mut self, to: impl Into<SharedString>) -> Self {
    self.to = to.into();
    self
  }

  /// Sets the style for the active state of the navigation link.
  pub fn active(mut self, f: impl FnOnce(StyleRefinement) -> StyleRefinement) -> Self {
    debug_assert!(self.active_style.is_none(), "active style already set");
    self.active_style = Some(Box::new(f(StyleRefinement::default())));
    self
  }

  /// When `true`, the active style will only be applied when the pathname
  /// matches the `to` path exactly. By default this is `false`, meaning the
  /// link is also considered active when the current pathname is a child of
  /// the `to` path (prefix matching).
  ///
  /// This is equivalent to React Router's `end` prop on `NavLink`.
  pub fn end(mut self, end: bool) -> Self {
    self.end = end;
    self
  }
}

impl RenderOnce for NavLink {
  fn render(mut self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let pathname = &cx.global::<RouterState>().location.pathname;
    let is_active = if self.to == "/" || self.end {
      pathname.as_ref() == self.to.as_ref()
    } else {
      pathname.as_ref() == self.to.as_ref()
        || pathname
          .strip_prefix(self.to.as_ref())
          .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    };

    if is_active {
      if let Some(active_style) = self.active_style.as_ref() {
        self.base.style().refine(active_style);
      }
    }

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

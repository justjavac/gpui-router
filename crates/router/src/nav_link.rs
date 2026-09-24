#[cfg(test)]
use crate::normalize_pathname;
use crate::{RouterState, normalize_shared_pathname, use_navigate};
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
  element_id: Option<ElementId>,
  active_style: Option<Box<StyleRefinement>>,
  end: bool,
}

impl Default for NavLink {
  fn default() -> Self {
    Self {
      base: div(),
      children: Default::default(),
      to: Default::default(),
      element_id: None,
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

  /// Sets the id of the clickable element.
  ///
  /// It defaults to the target path, so several links to the same path share
  /// that id. Use this when an application needs a stable id per link, for
  /// example to annotate the link for tooling or accessibility.
  pub fn element_id(mut self, id: impl Into<ElementId>) -> Self {
    self.element_id = Some(id.into());
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

#[cfg(test)]
fn is_active_path(pathname: &str, to: &str, end: bool) -> bool {
  is_active(
    normalize_pathname(pathname).as_ref(),
    normalize_pathname(to).as_ref(),
    end,
  )
}

/// Both pathnames are already normalized.
fn is_active(pathname: &str, to: &str, end: bool) -> bool {
  if to == "/" || end {
    pathname == to
  } else {
    pathname == to
      || pathname
        .strip_prefix(to)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
  }
}

impl RenderOnce for NavLink {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    self.link_element(cx)
  }
}

impl NavLink {
  /// Applies the active style, the click handler and the element id, and
  /// returns the element an application renders.
  fn link_element(mut self, cx: &App) -> Stateful<Div> {
    let to = normalize_shared_pathname(&self.to);
    let is_active = if cx.has_global::<RouterState>() {
      is_active(
        cx.global::<RouterState>().location.pathname.as_ref(),
        to.as_ref(),
        self.end,
      )
    } else {
      debug_assert!(
        false,
        "NavLink rendered without initialized RouterState; \
         ensure the router is initialized (e.g., via crate::init()) before rendering NavLink."
      );
      false
    };

    if is_active && let Some(active_style) = self.active_style.as_ref() {
      self.base.style().refine(active_style);
    }

    let element_id = self.element_id.take().unwrap_or_else(|| ElementId::from(to.clone()));

    self
      .base
      .id(element_id)
      .on_click(move |_, window, cx| {
        let mut navigate = use_navigate(cx);
        navigate(to.clone());
        window.refresh();
      })
      .children(self.children)
  }
}

#[cfg(test)]
mod tests {
  use super::{NavLink, is_active_path};
  use gpui::{Element, ElementId};

  #[test]
  fn test_root_nav_link_is_only_active_on_exact_root() {
    assert!(is_active_path("/", "/", false));
    assert!(is_active_path("", "", false));
    assert!(!is_active_path("/settings", "/", false));
  }

  #[test]
  fn test_nav_link_matches_descendants_by_default() {
    assert!(is_active_path("/settings/profile", "/settings", false));
    assert!(is_active_path("/settings/profile/", "settings", false));
  }

  #[test]
  fn test_nav_link_end_requires_exact_match() {
    assert!(is_active_path("/settings", "/settings", true));
    assert!(!is_active_path("/settings/profile", "/settings", true));
  }

  #[test]
  fn test_nav_link_respects_segment_boundaries() {
    assert!(!is_active_path("/users", "/user", false));
    assert!(!is_active_path("/settings-and-more", "/settings", false));
  }

  #[gpui::test]
  async fn test_nav_link_element_id(cx: &mut gpui::TestAppContext) {
    cx.update(crate::init);

    cx.update(|cx| {
      let default_id = NavLink::new().to("/about").link_element(cx);
      assert_eq!(Element::id(&default_id), Some(ElementId::from("/about")));

      let explicit = NavLink::new().to("/about").element_id("footer-about").link_element(cx);
      assert_eq!(Element::id(&explicit), Some(ElementId::from("footer-about")));
    });
  }
}

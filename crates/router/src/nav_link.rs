use crate::state::split_target;
use crate::{RouterState, normalize_pathname, use_navigate};
use gpui::*;
use smallvec::SmallVec;

/// A link that navigates to another route when clicked.
pub fn link() -> impl IntoElement {
  Link::new()
}

/// A link that changes the route when clicked and styles itself while active.
pub fn nav_link() -> impl IntoElement {
  NavLink::new().active(|style| style)
}

/// A link that navigates to another route when clicked.
///
/// This is React Router's `Link`: no active state, just navigation.
#[derive(IntoElement)]
pub struct Link {
  base: Div,
  children: SmallVec<[AnyElement; 1]>,
  to: SharedString,
  element_id: Option<ElementId>,
}

impl Default for Link {
  fn default() -> Self {
    Self {
      base: div(),
      children: Default::default(),
      to: Default::default(),
      element_id: None,
    }
  }
}

impl Styled for Link {
  fn style(&mut self) -> &mut StyleRefinement {
    self.base.style()
  }
}

impl ParentElement for Link {
  fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
    self.children.extend(elements);
  }
}

impl InteractiveElement for Link {
  fn interactivity(&mut self) -> &mut gpui::Interactivity {
    self.base.interactivity()
  }
}

impl Link {
  pub fn new() -> Self {
    Default::default()
  }

  /// Sets the destination route for the link.
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

  /// Applies the click handler and the element id, and returns the element an
  /// application renders.
  fn link_element(mut self, cx: &App) -> Stateful<Div> {
    // A link in an application that never called `init` would otherwise fail
    // when the user clicks it; fail while rendering instead.
    let _ = RouterState::require(cx);

    // The target keeps its query string and fragment: the navigator parses
    // them, and routes only match the pathname.
    let to = self.to.clone();
    let element_id = self.element_id.take().unwrap_or_else(|| ElementId::from(to.clone()));

    self
      .base
      .id(element_id)
      .on_click(move |_, window, cx| {
        let mut navigate = use_navigate(cx);
        navigate.push(to.clone());
        window.refresh();
      })
      .children(self.children)
  }
}

impl RenderOnce for Link {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    self.link_element(cx)
  }
}

/// A link that knows whether it points at the current location.
///
/// This is React Router's `NavLink`: a `Link` plus `active`, `end` and
/// `case_sensitive`.
#[derive(Default, IntoElement)]
pub struct NavLink {
  link: Link,
  active_style: Option<Box<StyleRefinement>>,
  end: bool,
  /// React Router compares the active path case-insensitively by default.
  case_sensitive: bool,
}

impl Styled for NavLink {
  fn style(&mut self) -> &mut StyleRefinement {
    self.link.style()
  }
}

impl ParentElement for NavLink {
  fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
    self.link.extend(elements);
  }
}

impl InteractiveElement for NavLink {
  fn interactivity(&mut self) -> &mut gpui::Interactivity {
    self.link.interactivity()
  }
}

impl NavLink {
  pub fn new() -> Self {
    Default::default()
  }

  /// Sets the destination route for the navigation link.
  pub fn to(mut self, to: impl Into<SharedString>) -> Self {
    self.link = self.link.to(to);
    self
  }

  /// Sets the id of the clickable element.
  ///
  /// It defaults to the target path, so several links to the same path share
  /// that id. Use this when an application needs a stable id per link, for
  /// example to annotate the link for tooling or accessibility.
  pub fn element_id(mut self, id: impl Into<ElementId>) -> Self {
    self.link = self.link.element_id(id);
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

  /// When `true`, the active state only matches pathnames with the same case.
  /// The default is `false`, which is React Router's `caseSensitive` default.
  pub fn case_sensitive(mut self, case_sensitive: bool) -> Self {
    self.case_sensitive = case_sensitive;
    self
  }

  /// Applies the active style, then renders the underlying link.
  fn link_element(mut self, cx: &App) -> Stateful<Div> {
    let (pathname, _, _) = split_target(self.link.to.as_ref());
    let to = normalize_pathname(pathname);
    let is_active = is_active(
      RouterState::require(cx).location.pathname.as_ref(),
      to.as_ref(),
      self.end,
      self.case_sensitive,
    );

    if is_active && let Some(active_style) = self.active_style.as_ref() {
      self.link.base.style().refine(active_style);
    }

    self.link.link_element(cx)
  }
}

impl RenderOnce for NavLink {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    self.link_element(cx)
  }
}

#[cfg(test)]
fn is_active_path(pathname: &str, to: &str, end: bool, case_sensitive: bool) -> bool {
  is_active(
    normalize_pathname(pathname).as_ref(),
    normalize_pathname(to).as_ref(),
    end,
    case_sensitive,
  )
}

/// Both pathnames are already normalized.
fn is_active(pathname: &str, to: &str, end: bool, case_sensitive: bool) -> bool {
  if to == "/" || end {
    same_path(pathname, to, case_sensitive)
  } else {
    same_path(pathname, to, case_sensitive) || is_child_path(pathname, to, case_sensitive)
  }
}

fn same_path(pathname: &str, to: &str, case_sensitive: bool) -> bool {
  if case_sensitive {
    pathname == to
  } else {
    pathname.eq_ignore_ascii_case(to)
  }
}

/// Whether `pathname` sits below `to`, on a segment boundary. Comparing byte
/// lengths is safe because the prefixes here are ASCII whenever they match.
fn is_child_path(pathname: &str, to: &str, case_sensitive: bool) -> bool {
  pathname.len() > to.len()
    && pathname.is_char_boundary(to.len())
    && same_path(&pathname[..to.len()], to, case_sensitive)
    && pathname.as_bytes()[to.len()] == b'/'
}

#[cfg(test)]
mod tests {
  use super::{Link, NavLink, is_active_path};
  use gpui::{Element, ElementId};

  #[test]
  fn test_root_nav_link_is_only_active_on_exact_root() {
    assert!(is_active_path("/", "/", false, false));
    assert!(is_active_path("", "", false, false));
    assert!(!is_active_path("/settings", "/", false, false));
  }

  #[test]
  fn test_nav_link_matches_descendants_by_default() {
    assert!(is_active_path("/settings/profile", "/settings", false, false));
    assert!(is_active_path("/settings/profile/", "settings", false, false));
  }

  #[test]
  fn test_nav_link_end_requires_exact_match() {
    assert!(is_active_path("/settings", "/settings", true, false));
    assert!(!is_active_path("/settings/profile", "/settings", true, false));
  }

  #[test]
  fn test_nav_link_respects_segment_boundaries() {
    assert!(!is_active_path("/users", "/user", false, false));
    assert!(!is_active_path("/settings-and-more", "/settings", false, false));
  }

  #[test]
  fn test_nav_link_active_is_case_insensitive_unless_requested() {
    assert!(is_active_path("/About", "/about", false, false));
    assert!(is_active_path("/About/Team", "/about", false, false));
    assert!(!is_active_path("/About", "/about", false, true));
    assert!(!is_active_path("/About/Team", "/about", false, true));
  }

  #[test]
  fn test_nav_link_active_handles_multibyte_prefixes() {
    // The byte length of `to` is not a character boundary in `pathname`.
    assert!(!is_active_path("/日本語", "/ab", false, false));
  }

  #[gpui::test]
  async fn test_link_element_id(cx: &mut gpui::TestAppContext) {
    cx.update(crate::init);

    cx.update(|cx| {
      let default_id = Link::new().to("/about").link_element(cx);
      assert_eq!(Element::id(&default_id), Some(ElementId::from("/about")));

      let explicit = Link::new().to("/about").element_id("footer-about").link_element(cx);
      assert_eq!(Element::id(&explicit), Some(ElementId::from("footer-about")));

      let nav = NavLink::new().to("/about").element_id("nav-about").link_element(cx);
      assert_eq!(Element::id(&nav), Some(ElementId::from("nav-about")));
    });
  }
}

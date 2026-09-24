use gpui::*;
use std::cell::RefCell;

thread_local! {
  /// The matched child of the route whose element is currently being built.
  ///
  /// [`Route::render`](crate::Route) fills this slot while it calls a route's
  /// element closure, so an [`Outlet`] created inside that closure renders the
  /// matched child. Nested routes save and restore the slot, which keeps every
  /// level pointing at its own child.
  static SCOPED_OUTLET: RefCell<Option<AnyElement>> = const { RefCell::new(None) };
}

/// Makes `element` available to the first [`Outlet`] built inside `build`.
pub(crate) fn with_outlet_scope<R>(element: Option<AnyElement>, build: impl FnOnce() -> R) -> R {
  struct Restore(Option<AnyElement>);

  impl Drop for Restore {
    fn drop(&mut self) {
      let previous = self.0.take();
      SCOPED_OUTLET.with(|slot| *slot.borrow_mut() = previous);
    }
  }

  let previous = SCOPED_OUTLET.with(|slot| std::mem::replace(&mut *slot.borrow_mut(), element));
  let _restore = Restore(previous);
  build()
}

/// Takes the child that the current route is rendering into its outlet, if any.
fn take_scoped_outlet() -> Option<AnyElement> {
  SCOPED_OUTLET.with(|slot| slot.borrow_mut().take())
}

/// An outlet is a placeholder in the UI where routed components will be rendered.
pub fn outlet() -> impl IntoElement {
  Outlet::new()
}

/// A placeholder element for routed components.
///
/// An outlet renders the child route that matches the current location. It
/// picks that child up while the surrounding route's element is created, so it
/// has to be constructed inside that element closure:
///
/// ```ignore
/// Route::new()
///   .path("/")
///   .element(|_, _| div().child(Outlet::new()))
///   .children(vec![Route::new().index().element(|_, _| home())]);
/// ```
///
/// Routes that use [`Route::layout`](crate::Route::layout) fill the outlet
/// through [`Layout::outlet`](crate::Layout::outlet) instead. When no child
/// matches, an outlet renders as an empty element.
///
/// Only the first outlet created by an element receives the matched child, and
/// a route whose element never creates one keeps matching its children while
/// rendering no child content.
#[derive(IntoElement)]
pub struct Outlet {
  pub(crate) element: AnyElement,
}

impl Default for Outlet {
  fn default() -> Self {
    Outlet {
      element: take_scoped_outlet().unwrap_or_else(|| Empty {}.into_any_element()),
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

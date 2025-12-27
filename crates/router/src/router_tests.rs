#[cfg(test)]
pub mod tests {
  use crate::{Route, RouterState, Routes};
  use gpui::prelude::*;
  use gpui::{TestAppContext, VisualTestContext, Window};

  struct Basic {}

  impl Render for Basic {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
      Routes::new()
        .basename("/")
        .child(Route::new().index().element(|_, _| "home"))
        .child(Route::new().path("about").element(|_, _| "about"))
        .child(Route::new().path("dashboard").element(|_, _| "dashboard"))
        .child(Route::new().path("{*not_match}").element(|_, _| "not_match"))
    }
  }

  #[gpui::test]
  async fn test_router(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);
      assert_eq!(cx.global::<RouterState>().location.pathname, "/");
    });
    let window = cx.add_window(|_, _cx| Basic {});

    {
      let mut cx = VisualTestContext::from_window(window.into(), cx);
      assert!(!cx.simulate_close());
    }

    let view = cx.new(|_cx| {
      Routes::new()
        .basename("/")
        .child(Route::new().index().element(|_, _| "home"))
        .child(Route::new().path("about").element(|_, _| "about"))
        .child(Route::new().path("dashboard").element(|_, _| "dashboard"))
        .child(Route::new().path("{*not_match}").element(|_, _| "not_match"))
    });
    view.update(cx, |this, cx| {
      assert_eq!(cx.global::<RouterState>().location.pathname, "/");
      assert_eq!(this.routes().len(), 4);
    })
  }

  #[gpui::test]
  async fn test_lazy_element_evaluation(cx: &mut TestAppContext) {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    cx.update(|cx| {
      crate::init(cx);
    });

    // Counter to track how many times each element function is called
    let home_counter = Arc::new(AtomicU32::new(0));
    let about_counter = Arc::new(AtomicU32::new(0));

    let home_counter_clone = home_counter.clone();
    let about_counter_clone = about_counter.clone();

    // Create routes with elements that increment counters when called
    let _view = cx.new(|_cx| {
      Routes::new()
        .basename("/")
        .child(Route::new().index().element(move |_, _| {
          home_counter_clone.fetch_add(1, Ordering::SeqCst);
          "home"
        }))
        .child(Route::new().path("about").element(move |_, _| {
          about_counter_clone.fetch_add(1, Ordering::SeqCst);
          "about"
        }))
    });

    // At this point, neither element function should have been called yet
    // because we only created the Routes structure, not rendered it
    assert_eq!(
      home_counter.load(Ordering::SeqCst),
      0,
      "Home element should not be evaluated during route configuration"
    );
    assert_eq!(
      about_counter.load(Ordering::SeqCst),
      0,
      "About element should not be evaluated during route configuration"
    );
  }
}

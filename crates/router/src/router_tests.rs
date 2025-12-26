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
}

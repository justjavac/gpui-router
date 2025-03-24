use gpui::prelude::*;
use gpui::{App, Application, Context, Window, WindowOptions, div, rgb, white};
use gpui_router::{NavLink, Outlet, init as router_init};
use gpui_router::{Route, Routes};

struct SubRouter {}

impl Render for SubRouter {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .flex()
      .flex_col()
      .gap_2()
      .size_full()
      .p_2()
      .bg(rgb(0x2e7d32))
      .text_color(white())
      .child(div().text_xl().child("Sub Router Example"))
      .child(
        Routes::new().child(
          Route::new()
            .path("/")
            .element(layout())
            .child(Route::new().index().element(home()))
            .child(Route::new().path("about").element(about()))
            .child(Route::new().path("dashboard").element(dashboard()))
            .child(Route::new().path("{*not_match}").element(not_match())),
        ),
      )
  }
}

fn layout() -> impl IntoElement {
  div()
    .child(
      div()
        .flex()
        .gap_4()
        .text_lg()
        .child(NavLink::new().to("/").child(div().child("Home")))
        .child(NavLink::new().to("/about").child(div().child("About")))
        .child(NavLink::new().to("/dashboard").child(div().child("Dashboard")))
        .child(NavLink::new().to("/nothing-here").child(div().child("Not Match"))),
    )
    .child(Outlet::new())
}

fn home() -> impl IntoElement {
  div().child("Home")
}

fn about() -> impl IntoElement {
  div().child("About")
}

fn dashboard() -> impl IntoElement {
  div().child("Dashboard")
}

fn not_match() -> impl IntoElement {
  div()
    .child(div().child("Nothing to see here!"))
    .child(NavLink::new().to("/").child(div().child("Go to the home page")))
}

fn main() {
  Application::new().run(|cx: &mut App| {
    router_init(cx);

    cx.on_window_closed(|cx| {
      if cx.windows().is_empty() {
        cx.quit();
      }
    })
    .detach();

    cx.activate(true);
    cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_cx| SubRouter {}))
      .unwrap();
  });
}

use crate::init as router_init;
use gpui::*;
use gpui_router::*;

struct Basic {}

impl Render for Basic {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .flex()
      .flex_col()
      .gap_2()
      .size_full()
      .p_2()
      .bg(rgb(0x2e7d32))
      .text_color(white())
      .child(div().text_xl().child("Basic Example"))
      .child(nav())
      .child(
        Routes::new()
          .basename("/")
          .child(Route::new().index().element(|_, _| home()))
          .child(Route::new().path("about").element(|_, _| about()))
          .child(Route::new().path("dashboard").element(|_, _| dashboard()))
          .child(Route::new().path("{*not_match}").element(|_, _| not_match())),
      )
  }
}

fn nav() -> impl IntoElement {
  div()
    .flex()
    .gap_4()
    .text_lg()
    .child(NavLink::new().to("/").child(div().child("Home")))
    .child(NavLink::new().to("/about").child(div().child("About")))
    .child(NavLink::new().to("/dashboard").child(div().child("Dashboard")))
    .child(NavLink::new().to("/nothing-here").child(div().child("Not Match")))
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
  div().id("not_match").child(div().child("Nothing to see here!")).child(
    NavLink::new()
      .to("/")
      .child(div().text_decoration_1().child("Go to the home page")),
  )
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
    cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_cx| Basic {}))
      .unwrap();
  });
}

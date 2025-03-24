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
      .child(div().text_xl().child("Basic Example With Params"))
      .child(nav())
      .child(
        Routes::new()
          .basename("/")
          .child(Route::new().index().element(home()))
          .child(Route::new().path("user").element(user_list()))
          .child(Route::new().path("user/{id}").element(user()))
          .child(Route::new().path("{*not_match}").element(not_match())),
      )
  }
}

fn nav() -> impl IntoElement {
  div()
    .flex()
    .gap_4()
    .text_lg()
    .child(NavLink::new().to("/").child(div().child("Home")))
    .child(NavLink::new().to("/user").child(div().child("Users")))
    .child(NavLink::new().to("/nothing-here").child(div().child("Not Match")))
}

fn home() -> impl IntoElement {
  div().child("Home")
}

fn user_list() -> impl IntoElement {
  div()
    .flex()
    .flex_col()
    .gap_2()
    .child(NavLink::new().to("/user/1").child(div().child("User1")))
    .child(NavLink::new().to("/user/2").child(div().child("User2")))
    .child(NavLink::new().to("/user/3").child(div().child("User3")))
}

fn user() -> impl IntoElement {
  div().child("User")
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

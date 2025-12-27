use gpui::prelude::*;
use gpui::{App, Application, Context, Window, WindowOptions, div, rgb, white};
use gpui_router::{IntoLayout, NavLink, Outlet, Route, Routes, init as router_init, use_params};

struct NestedRouter {}

impl Render for NestedRouter {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .flex()
      .flex_col()
      .gap_2()
      .size_full()
      .p_2()
      .bg(rgb(0x2e7d32))
      .text_color(white())
      .child(div().text_xl().child("Nested Router Example"))
      .child(
        Routes::new().child(
          Route::new()
            .layout(Nav::new())
            .child(Route::new().index().element(|_, _| home()))
            .child(Route::new().path("about").element(|_, _| about()))
            .child(
              Route::new()
                .path("user")
                .layout(UserLayout::new())
                .child(Route::new().index().element(|_, _| user_list()))
                .child(Route::new().path("{id}").element(|_, _| User {})),
            )
            .child(Route::new().path("{*not_match}").element(|_, _| not_match())),
        ),
      )
  }
}

#[derive(IntoElement, IntoLayout)]
pub struct Nav {
  outlet: Outlet,
}

impl Nav {
  pub fn new() -> Self {
    Self { outlet: Outlet::new() }
  }
}

impl RenderOnce for Nav {
  fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    div()
      .child(
        div()
          .flex()
          .gap_4()
          .text_lg()
          .child(NavLink::new().to("/").child(div().child("Home")))
          .child(NavLink::new().to("/about").child(div().child("About")))
          .child(NavLink::new().to("/user").child(div().child("User")))
          .child(NavLink::new().to("/user/1").child(div().child("User1")))
          .child(NavLink::new().to("/nothing-here").child(div().child("Not Match"))),
      )
      .child(self.outlet)
  }
}

#[derive(IntoElement, IntoLayout)]
pub struct UserLayout {
  outlet: Outlet,
}

impl UserLayout {
  pub fn new() -> Self {
    Self { outlet: Outlet::new() }
  }
}

impl RenderOnce for UserLayout {
  fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    div()
      .child(div().flex().gap_4().text_lg().child("User Layout"))
      .child(self.outlet)
  }
}

fn home() -> impl IntoElement {
  div().child("Home")
}

fn about() -> impl IntoElement {
  div().child("About")
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

#[derive(IntoElement)]
pub struct User {}

impl RenderOnce for User {
  fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
    let params = use_params(cx);
    div().child(format!("User: {}", params.get("id").unwrap()))
  }
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
    cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_cx| NestedRouter {}))
      .unwrap();
  });
}

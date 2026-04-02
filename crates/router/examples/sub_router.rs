use gpui::prelude::*;
use gpui::{App, Application, Context, RenderOnce, Window, WindowOptions, div, rgb, white};
use gpui_router::{NavLink, Route, Routes, init as router_init};

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
      .child(div().child(
        "The top-level router hands off every /settings route to a nested SettingsRouter, \
           which then routes inside the /settings subtree with its own basename.",
      ))
      .child(root_nav())
      .child(
        Routes::new()
          .basename("/")
          .child(Route::new().index().element(|_, _| home()))
          .child(Route::new().path("about").element(|_, _| about()))
          .child(Route::new().path("dashboard").element(|_, _| dashboard()))
          .child(Route::new().path("settings").element(|_, _| SettingsRouter {}))
          .child(Route::new().path("settings/{*rest}").element(|_, _| SettingsRouter {}))
          .child(Route::new().path("{*not_match}").element(|_, _| not_match())),
      )
  }
}

#[derive(IntoElement)]
struct SettingsRouter {}

impl RenderOnce for SettingsRouter {
  fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    div()
      .mt_4()
      .flex()
      .flex_col()
      .gap_2()
      .rounded_md()
      .bg(rgb(0x1b5e20))
      .p_3()
      .child(div().text_lg().child("SettingsRouter"))
      .child(
        div().child("This nested router only handles the /settings subtree via Routes::new().basename(\"/settings\")."),
      )
      .child(settings_nav())
      .child(
        Routes::new()
          .basename("/settings")
          .child(Route::new().index().element(|_, _| settings_home()))
          .child(Route::new().path("profile").element(|_, _| settings_profile()))
          .child(Route::new().path("security").element(|_, _| settings_security()))
          .child(Route::new().path("billing").element(|_, _| settings_billing()))
          .child(Route::new().path("{*rest}").element(|_, _| settings_not_match())),
      )
  }
}

fn root_nav() -> impl IntoElement {
  div()
    .flex()
    .gap_4()
    .text_lg()
    .child(NavLink::new().to("/").child(div().child("Home")))
    .child(NavLink::new().to("/about").child(div().child("About")))
    .child(NavLink::new().to("/dashboard").child(div().child("Dashboard")))
    .child(NavLink::new().to("/settings").child(div().child("Settings")))
    .child(
      NavLink::new()
        .to("/settings/security")
        .child(div().child("Settings Security")),
    )
    .child(NavLink::new().to("/nothing-here").child(div().child("Not Match")))
}

fn settings_nav() -> impl IntoElement {
  div()
    .flex()
    .gap_4()
    .child(NavLink::new().to("/settings").end(true).child(div().child("Overview")))
    .child(NavLink::new().to("/settings/profile").child(div().child("Profile")))
    .child(NavLink::new().to("/settings/security").child(div().child("Security")))
    .child(NavLink::new().to("/settings/billing").child(div().child("Billing")))
    .child(
      NavLink::new()
        .to("/settings/unknown-page")
        .child(div().child("Local 404")),
    )
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

fn settings_home() -> impl IntoElement {
  div().child("Settings overview handled by the nested router.")
}

fn settings_profile() -> impl IntoElement {
  div().child("Profile settings page")
}

fn settings_security() -> impl IntoElement {
  div().child("Security settings page")
}

fn settings_billing() -> impl IntoElement {
  div().child("Billing settings page")
}

fn settings_not_match() -> impl IntoElement {
  div()
    .child(div().child("This route was not found inside SettingsRouter."))
    .child(
      NavLink::new()
        .to("/settings")
        .child(div().child("Back to settings overview")),
    )
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

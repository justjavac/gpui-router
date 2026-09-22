//! gpui-kit compatibility harness for `gpui-router`.
//!
//! The crate is unpublished: it exists so the `gpui-pre` backend, the
//! `IntoLayout` derive, and the router outlet are exercised the way a
//! gpui-kit application uses them.

use gpui_kit::component::{
  Root,
  badge::Badge,
  button::{Button, ButtonVariants as _},
};
use gpui_kit::prelude::*;
use gpui_kit::{
  App, Context, Entity, IntoElement, ParentElement, Render, RenderOnce, SharedString, TestSupportExt, Window, div,
};
use gpui_router::{IntoLayout, NavLink, Outlet, Route, Routes};

/// The routed application view.
pub struct DemoApp;

impl Render for DemoApp {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div().size_full().child(
      Routes::new().child(
        Route::new()
          .layout(Shell::new())
          .child(Route::new().index().element(|_, _| home()))
          .child(Route::new().path("about").element(|_, _| about()))
          .child(Route::new().path("{*not_match}").element(|_, _| not_match())),
      ),
    )
  }
}

/// Chrome around the routed outlet.
///
/// `#[derive(IntoLayout)]` has to expand to `gpui_router` paths: this crate has
/// no dependency on a crate named `gpui`.
#[derive(Default, IntoElement, IntoLayout)]
pub struct Shell {
  outlet: Outlet,
}

impl Shell {
  pub fn new() -> Self {
    Self { outlet: Outlet::new() }
  }
}

impl RenderOnce for Shell {
  fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    div()
      .id("shell")
      .child(
        div()
          .id("nav-about")
          .test_support()
          .child(NavLink::new().to("/about").child(div().child("About"))),
      )
      .child(self.outlet)
  }
}

/// Mounts the routed view inside the gpui-kit root view.
pub fn root(view: Entity<DemoApp>, window: &mut Window, cx: &mut Context<Root>) -> Root {
  Root::new(view, window, cx)
}

fn home() -> impl IntoElement {
  div()
    .id("page")
    .test_support()
    .aria_label(SharedString::from("home"))
    .child(Button::new("cta").primary().label("Kit button"))
    .child(Badge::new().count(3))
}

fn about() -> impl IntoElement {
  div()
    .id("page")
    .test_support()
    .aria_label(SharedString::from("about"))
    .child(Badge::new().dot())
}

fn not_match() -> impl IntoElement {
  div()
    .id("page")
    .test_support()
    .aria_label(SharedString::from("not-match"))
    .child(NavLink::new().to("/").child(div().child("Go home")))
}

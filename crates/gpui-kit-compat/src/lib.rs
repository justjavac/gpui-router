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
  AnyView, App, Context, IntoElement, ParentElement, Render, RenderOnce, SharedString, TestSupportExt, Window, div,
};
use gpui_router::{IntoLayout, Link, NavLink, Outlet, Redirect, Route, Routes};

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
pub fn root(view: impl Into<AnyView>, window: &mut Window, cx: &mut Context<Root>) -> Root {
  Root::new(view, window, cx)
}

/// Nested routes written the way the README shows them: the parent element
/// renders the matched child through `Outlet`.
pub struct OutletApp;

impl Render for OutletApp {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div().size_full().child(
      Routes::new().child(Route::new().path("/").element(|_, cx| layout(cx)).children(vec![
        Route::new().index().element(|_, _| home()),
        Route::new().path("about").element(|_, _| about_layout()).children(vec![
          Route::new().index().element(|_, _| about()),
          Route::new().path("team").element(|_, _| team()),
        ]),
        Route::new()
          .path("settings")
          .layout(SettingsShell::new())
          .child(Route::new().path("profile").element(|_, _| settings_profile())),
        Route::new().path("{*not_match}").element(|_, _| not_match()),
      ])),
    )
  }
}

fn layout(cx: &App) -> impl IntoElement + use<> {
  div()
    .id("shell")
    .child(
      div()
        .child(
          div()
            .id("nav-about")
            .test_support()
            .child(NavLink::new().to("/about").child(div().child("About"))),
        )
        .child(
          div()
            .id("nav-team")
            .test_support()
            .child(NavLink::new().to("/about/team").child(div().child("Team"))),
        )
        .child(
          div()
            .id("nav-settings")
            .test_support()
            .child(NavLink::new().to("/settings/profile").child(div().child("Settings"))),
        )
        .child(
          div()
            .id("link-team")
            .test_support()
            .child(Link::new().to("/about/team").child(div().child("Team link"))),
        )
        .child(div().id("breadcrumbs").test_support().aria_label(breadcrumbs(cx)))
        .child(
          div().id("link-query").test_support().child(
            Link::new()
              .to("/settings/profile?tab=billing")
              .child(div().child("Billing")),
          ),
        )
        .child(div().id("query").test_support().aria_label(query_summary(cx))),
    )
    .child(Outlet::new())
}

/// The chain `use_matches` returns, which is what breadcrumbs are built from.
fn breadcrumbs(cx: &App) -> SharedString {
  SharedString::from(
    gpui_router::use_matches(cx)
      .iter()
      .map(|matched| matched.pathname.clone())
      .collect::<Vec<_>>()
      .join(" > "),
  )
}

/// The current query string plus one parsed value, which is what
/// `use_search_params` reads.
fn query_summary(cx: &App) -> SharedString {
  let search = gpui_router::use_location(cx).search.clone();
  let tab = gpui_router::use_search_params(cx).get("tab").unwrap_or("");

  SharedString::from(format!("{search}|{tab}"))
}

fn about_layout() -> impl IntoElement {
  div()
    .id("about-shell")
    .child(
      div()
        .id("link-relative-team")
        .test_support()
        .child(Link::new().to("team").child(div().child("Relative team link"))),
    )
    .child(Outlet::new())
}

/// Layout-style chrome nested inside an element route.
#[derive(Default, IntoElement, IntoLayout)]
pub struct SettingsShell {
  outlet: Outlet,
}

impl SettingsShell {
  pub fn new() -> Self {
    Self { outlet: Outlet::new() }
  }
}

impl RenderOnce for SettingsShell {
  fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
    div().child(self.outlet)
  }
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

fn team() -> impl IntoElement {
  div()
    .id("page")
    .test_support()
    .aria_label(SharedString::from("team"))
    .child(NavLink::new().to("/about").child(div().child("Back to about")))
}

fn settings_profile() -> impl IntoElement {
  div()
    .id("page")
    .test_support()
    .aria_label(SharedString::from("settings-profile"))
}

/// A route with children but no chrome of its own — React Router's pathless
/// layout route — renders the matched child directly.
pub struct GroupingApp;

impl Render for GroupingApp {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .size_full()
      .child(Routes::new().child(Route::new().path("settings").children(vec![
        Route::new().path("profile").element(|_, _| grouping_page("settings-profile")),
        Route::new().path("billing").element(|_, _| grouping_page("settings-billing")),
        Route::new().path("users/{id}").element(|_, cx| grouping_page_with_params(cx)),
        Route::new().path("account").children(vec![
          Route::new()
            .path("security")
            .element(|_, _| grouping_page("account-security")),
        ]),
      ])))
  }
}

fn grouping_page(name: &'static str) -> impl IntoElement {
  div().id("page").test_support().aria_label(SharedString::from(name))
}

fn grouping_page_with_params(cx: &App) -> impl IntoElement + use<> {
  let id = gpui_router::use_params(cx).get("id").cloned().unwrap_or_default();
  div()
    .id("page")
    .test_support()
    .aria_label(SharedString::from(format!("user-{id}")))
}

fn not_match() -> impl IntoElement {
  div()
    .id("page")
    .test_support()
    .aria_label(SharedString::from("not-match"))
    .child(NavLink::new().to("/").child(div().child("Go home")))
}

/// A guard-style redirect: `/` sends the application to `/about`.
pub struct RedirectApp;

impl Render for RedirectApp {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div().size_full().child(Routes::new().child(Route::new().children(vec![
        Route::new()
          .path("/")
          .element(|_, _| Redirect::to("/about").replace(true)),
        Route::new()
          .path("about")
          .element(|_, _| grouping_page("redirected-about")),
      ])))
  }
}

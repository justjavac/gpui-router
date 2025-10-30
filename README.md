# gpui-router

[![ci](https://github.com/justjavac/gpui-router/actions/workflows/build.yml/badge.svg)](https://github.com/justjavac/gpui-router/actions/workflows/build.yml)
[![Crate](https://img.shields.io/crates/v/gpui-router.svg)](https://crates.io/crates/gpui-router)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/gpui-router)](https://crates.io/crates/gpui-router)
[![Documentation](https://docs.rs/gpui-router/badge.svg)](https://docs.rs/gpui-router)
![License](https://img.shields.io/crates/l/gpui-router.svg)

A router for [GPUI](https://www.gpui.rs/) App, inspired by React-Router.

## Features

- Nested Routes
- Index Routes
- Dynamic Segments
- Wildcard Routes
- Navigation Links

## Usage

```rust
use gpui::prelude::*;
use gpui::{App, Application, Context, Window, WindowOptions, div};
use gpui_router::{NavLink, Outlet, Route, Routes, init as router_init};

struct HelloWorld {}

impl Render for HelloWorld {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .child(
        Routes::new().child(
          Route::new().path("/").element(layout()).children(vec![
              Route::new().index().element(home()),
              Route::new().path("about").element(about()),
              Route::new().path("dashboard").element(dashboard()),
              Route::new().path("{*not_match}").element(not_match()),
            ]),
        ),
      )
  }
}

fn layout() -> impl IntoElement {
  div()
    .child(NavLink::new().to("/").child(div().child("Home")))
    .child(NavLink::new().to("/about").child(div().child("About")))
    .child(NavLink::new().to("/dashboard").child(div().child("Dashboard")))
    .child(NavLink::new().to("/nothing-here").child(div().child("Not Match")))
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
    cx.activate(true);
    cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_cx| HelloWorld {}))
      .unwrap();
  });
}
```

## Examples

See the [examples](./crates/router/examples) folder for more usage examples.

## License

MIT License - Same as the original Rust implementation.
See [LICENSE](./LICENSE) for details.

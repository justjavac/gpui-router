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

## GPUI versions

`gpui-router` builds against either GPUI release line, selected by a feature:

| Feature | GPUI crate | Use it for |
| --- | --- | --- |
| `gpui` (default) | [`gpui`](https://crates.io/crates/gpui) 0.2.x | Applications that depend on GPUI directly |
| `gpui-pre` | [`gpui-pre`](https://crates.io/crates/gpui-pre) 0.3.x | Applications built with [`gpui-kit`](https://crates.io/crates/gpui-kit) |

```toml
# Plain GPUI application
gpui-router = "0.3"

# gpui-kit application, which re-exports GPUI as `gpui_kit::*`
gpui-router = { version = "0.3", default-features = false, features = ["gpui-pre"] }
```

Both backends expose the same router API; only the GPUI types behind it change.
Enable exactly one backend. `gpui-kit` pins an exact `gpui-pre` release, so both
crates have to resolve to the same 0.3.x version, otherwise Cargo compiles two
copies of GPUI and route elements no longer type-check.

`gpui-kit` applications only need `gpui-kit` and `gpui-router`: derive macros
expand through `gpui_router::__private::gpui`, so a crate named `gpui` is not
required. The bundled examples bootstrap GPUI directly and therefore build on
the default backend only.

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
          Route::new().path("/").element(|_, _| layout()).children(vec![
              Route::new().index().element(|_, _| home()),
              Route::new().path("about").element(|_, _| about()),
              Route::new().path("dashboard").element(|_, _| dashboard()),
              Route::new().path("{*not_match}").element(|_, _| not_match()),
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

**Note:** The `element()` method now accepts a closure that returns an `IntoElement`. This allows for lazy evaluation of route elements - they are only rendered when the route matches, improving performance when you have many routes.

## Examples

See the [examples](./crates/router/examples) folder for more usage examples.

## License

This project is licensed under the MIT License.
See [LICENSE](./LICENSE) for details.

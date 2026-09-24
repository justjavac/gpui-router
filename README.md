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

## Coming from React Router?

The API follows React Router, so most of the knowledge transfers directly:

| React Router | `gpui-router` |
| --- | --- |
| `<Routes>` / `createBrowserRouter` | `Routes::new().children(vec![...])` |
| `<Route path element index>` | `Route::new().path("users/:id").element(\|_, _\| ...)`, `.index()` |
| `<Outlet />` | `Outlet::new()` |
| `path="users/:id"` | the same, and `{id}` also works |
| `path="*"` | the same, and it matches `/` like React Router |
| `<Link to>` | `Link::new().to(...)` |
| `<NavLink to>` | `NavLink::new().to(...).active(...).end(...)` |
| `useParams()`, `useLocation()`, `useNavigate()` | `use_params(cx)`, `use_location(cx)`, `use_navigate(cx)` |
| `navigate(-1)`, `useSearchParams()`, `<Navigate>` | planned, see [the alignment plan](./docs/react-router-alignment.md) |
| `loader`, `action`, `errorElement` | not planned yet; the plan covers a design pass first |

Two deliberate differences: `Route::layout(...)` plus `#[derive(IntoLayout)]` has
no React Router equivalent (it is an optional, explicit alternative to an
element with an `Outlet`), and GPUI has no context lookup, so `useOutletContext`
becomes passing an entity instead.

## GPUI versions

`gpui-router` builds against either GPUI release line, selected by a feature:

| Feature | GPUI crate | Use it for |
| --- | --- | --- |
| `gpui` (default) | [`gpui`](https://crates.io/crates/gpui) 0.2.x | Applications that depend on GPUI directly |
| `gpui-pre` | [`gpui-pre`](https://crates.io/crates/gpui-pre) 0.3.x | Applications built with [`gpui-kit`](https://crates.io/crates/gpui-kit) |

```toml
# Plain GPUI application
gpui-router = "0.4"

# gpui-kit application, which re-exports GPUI as `gpui_kit::*`
gpui-router = { version = "0.4", default-features = false, features = ["gpui-pre"] }
```

Both backends expose the same router API; only the GPUI types behind it change.
`gpui-kit` pins an exact `gpui-pre` release, so both crates have to resolve to
the same 0.3.x version, otherwise Cargo compiles two copies of GPUI and route
elements no longer type-check.

Enable exactly one backend: the choice applies to a whole build graph, so the
members of one workspace cannot mix backends, and `--all-features` fails with a
`compile_error!` instead of silently mixing GPUI types.

Feature passthroughs follow the enabled backend through weak `dep?/feature`
references. `font-kit`, `macos-blade` and `runtime_shaders` only exist on the
0.2.x line: on `gpui-pre` they are no-ops, because `font-kit` is part of that
crate's default features and `runtime_shaders` belongs to the platform crate
that `gpui-kit` selects for you.

`gpui-kit` applications only need `gpui-kit` and `gpui-router`: derive macros
expand through `gpui_router::__private::gpui`, so a crate named `gpui` is not
required. The bundled examples bootstrap GPUI directly and therefore build on
the default backend only.

The GPUI test harness used by this crate (`gpui::test`, `TestAppContext`) needs
the backend's `test-support` feature, which the 0.2.x dev-dependency enables by
default. Without it the backend-specific tests are skipped:

```sh
cargo test                                                          # default backend, all tests
cargo test --no-default-features --features gpui-pre,test-support   # gpui-kit backend, all tests
```

## Usage

```rust
use gpui::prelude::*;
use gpui::{App, Application, Context, Window, WindowOptions, div};
use gpui_router::{NavLink, Route, Routes, init as router_init};

struct HelloWorld {}

impl Render for HelloWorld {
  fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    div()
      .child(
        div()
          .child(NavLink::new().to("/").child(div().child("Home")))
          .child(NavLink::new().to("/about").child(div().child("About")))
          .child(NavLink::new().to("/dashboard").child(div().child("Dashboard")))
          .child(NavLink::new().to("/nothing-here").child(div().child("Not Match"))),
      )
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

**Note:** Router state is global. `Routes` stores the matched location in a
process-wide `RouterState`, so call `gpui_router::init` once during startup and
render one `Routes` tree per window. Rendering without that call panics with the
line to add, instead of silently rendering the wrong thing.

### Reading the location

Hooks read the router state from any `Render` implementation or event handler:

| Hook | Returns |
| --- | --- |
| `use_location(cx)` | the current `Location` (a normalized pathname) |
| `use_pattern(cx)` | the route pattern that matched, for example `/users/:id` |
| `use_params(cx)` | the dynamic parameters of the current match |
| `use_navigate(cx)` | a closure that navigates to another path |

```rust
use gpui_router::{use_params, use_pattern};

fn user_page(cx: &App) -> impl IntoElement {
  let id = use_params(cx).get("id").cloned().unwrap_or_default();
  let pattern = use_pattern(cx).cloned();

  div().child(format!("{id} matched {pattern:?}"))
}
```

### Nested routes

Paths use React Router's syntax: `users/:id` for a dynamic segment and `*` for
a splat. The `{id}` and `{*splat}` spellings of the matcher keep working, so
existing routes do not have to change. A splat also matches its parent path,
the way React Router's `*` does: `path("*")` covers `/` and `files/*` covers
`/files` with an empty `params["*"]`, while a real route such as an index route
still wins that path.

Give a route children and render an `Outlet` where the matched child should
appear:

```rust
use gpui_router::{NavLink, Outlet, Route, Routes};

Routes::new().child(
  Route::new()
    .path("/")
    .element(|_, _| layout())
    .children(vec![
      Route::new().index().element(|_, _| home()),
      Route::new().path("dashboard").element(|_, _| dashboard()),
      Route::new().path("{*not_match}").element(|_, _| not_match()),
    ]),
)

fn layout() -> impl IntoElement {
  div()
    .child(NavLink::new().to("/").child(div().child("Home")))
    .child(NavLink::new().to("/dashboard").child(div().child("Dashboard")))
    .child(Outlet::new())
}
```

An outlet picks up the matched child while the route's element is built, so it
has to be created inside that element closure. Nesting deeper works the same
way: give a child route its own `.element(...)` plus children. Only the first
outlet of an element receives the child, and an element that never creates one
renders no child content.

This is the React Router shape, and the recommended one. When the shared chrome
needs its own type and state, an optional alternative is to implement `Layout`
and pass it with `Route::layout(...)`; `#[derive(IntoLayout)]` wires the outlet
of a struct that has an `outlet` field. See
[examples/nested_router.rs](./crates/router/examples/nested_router.rs) for a
complete two-level example of that style.

## Examples

See the [examples](./crates/router/examples) folder for more usage examples.

## License

This project is licensed under the MIT License.
See [LICENSE](./LICENSE) for details.

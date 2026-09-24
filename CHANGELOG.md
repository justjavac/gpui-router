# Changelog

All notable changes to this project are documented in this file. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `use_pattern(cx)` returns the route pattern that matched the current location, for example `/users/{id}` while the pathname is `/users/42`.
- `NavLink::element_id(...)` sets the id of the clickable element, which otherwise defaults to the target path.

### Removed

- `PathMatch`, `RouterState::path_match` and `Location::state` were public but never populated: `path_match` was only ever set to `None`, and `Location::state` held a `matchit::Params` value the router never wrote. `RouterState::matched_pattern` and `use_pattern` replace them. This is a breaking change, so the next release is 0.6.0.

## [0.5.0] - 2026-09-24

### Added

- Nested routes written as `Route::element(...)` plus `.children(...)` now render the matched child into the first `Outlet` created by that element ([#12], [#9]).

### Changed

- Element routes also match their own path, so a parent renders with an empty outlet when no child matches; an index route still owns the parent path.

### Removed

- The debug panic that 0.4.1 added for `element(...)` plus `children(...)`.

## [0.4.1] - 2026-09-23

### Added

- Debug builds panic when a route with an element is given children, instead of silently rendering nothing ([#11], [#9]).

## [0.4.0] - 2026-09-23

### Added

- Backend selection through features: `gpui` (default, crates.io 0.2.x) and `gpui-pre` (the GPUI line behind `gpui-kit`) ([#7], [#8]).
- `gpui-kit-compat`, an unpublished workspace member that consumes the router the way a `gpui-kit` application does, plus a CI workflow that runs the `gpui-pre` backend on Ubuntu, macOS and Windows.

### Changed

- `#[derive(IntoLayout)]` expands through `gpui_router::__private::gpui`, so applications no longer need a dependency named `gpui`.
- `Route::render` calls `RenderOnce::render` explicitly, which newer GPUI releases require.

### Fixed

- The bundled examples declare `required-features = ["gpui"]` so they build on the default backend only.

[#7]: https://github.com/justjavac/gpui-router/pull/7
[#8]: https://github.com/justjavac/gpui-router/pull/8
[#9]: https://github.com/justjavac/gpui-router/issues/9
[#11]: https://github.com/justjavac/gpui-router/pull/11
[#12]: https://github.com/justjavac/gpui-router/pull/12

[Unreleased]: https://github.com/justjavac/gpui-router/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/justjavac/gpui-router/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/justjavac/gpui-router/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/justjavac/gpui-router/compare/v0.3.0...v0.4.0

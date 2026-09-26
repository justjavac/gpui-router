# React Router alignment

Goal: a React Router user should be able to use `gpui-router` without
translating concepts. Names match, semantics match, and the shapes that cannot
match (Rust has no JSX, no hooks rules, GPUI has no context lookup) are
documented instead of invented.

Work happens feature by feature: a short plan, an implementation, a review, one
pull request per feature, and a `CHANGELOG.md` entry under `Unreleased`.

## Target version

The reference is **React Router v6.4+/v7 in declarative mode** — `<Routes>`,
`<Route>`, `<Outlet>`, `<Link>`, `<NavLink>`, `<Navigate>` and the hooks — which
is what an application uses before it opts into a data router or framework mode.
Where the crate still carries a v5-era name (`Redirect` for `<Navigate>`) the
[divergence register](#divergence-register) records it.

Two halves of React Router are explicitly out of scope: the **mutation** side
(`action`, `<Form>`, `useSubmit`, `useFetcher`) and the **framework** side
(`lazy`, `<ScrollRestoration>`, RSC). The loader/error half is a design
document, not a commitment ([data-apis.md](./data-apis.md)).

## Principles

1. **Names first.** `Routes`, `Route`, `Link`, `NavLink`, `Outlet`, `Redirect`,
   `useLocation`, `useParams`, `useMatches`, `useMatch`, `useNavigate`,
   `useSearchParams` map one-to-one onto React Router's API (snake case in
   Rust).
2. **Semantics over syntax.** `navigate(to)`, `navigate(to, { replace })`,
   `navigate(-1)` and `navigate(1)` become one `Navigator` with `push`,
   `replace`, `back` and `forward`.
3. **Extra concepts stay optional and explicit.** Pathless layout routes plus
   `Outlet` are the way to nest, as in React Router; `Route::layout(...)` and
   `#[derive(IntoLayout)]` remain as an optional alternative for chrome that
   needs its own type. Everything else the crate has and React Router does not
   (`init(cx)`, `Router`, `Navigator`, `Relative`, `SearchParamsSetter`) is
   listed in the divergence register with the reason it exists.
4. **React Router path syntax works out of the box** — `:id` and `*` — because
   `path="users/:id"` silently matching a literal path today is the single most
   confusing failure mode for a React Router user. Path syntax the matcher
   cannot implement yet (`:id?`) fails loudly instead of half-working.
5. **Failures are loud.** A misconfiguration should panic with a message that
   names the route, never render an empty page.

## Decisions

| Question | Decision |
| --- | --- |
| Path syntax | Accept `:id` / `*` (React Router) **and** `{id}` / `{*splat}` (matchit). `:id?` needs a priority rule against index and sibling routes first, so it panics until then ([#37], [#38]). |
| `Location::state` | A string map (`Option<BTreeMap<SharedString, SharedString>>`). Sorting it keeps `Clone`, `PartialEq` and the `Ord`/`PartialOrd` that `Location` already had. |
| `Link` | Add it. React Router users look for both `Link` and `NavLink`. |
| Relative `to` | Default to React Router's `relative="route"`; `Relative::Path` selects the absolute behaviour. |
| History | `push` / `replace` / `back` / `forward` land in Phase 2. |
| Data APIs | `loader` / `action` / `errorElement` need a design document first; not part of the initial phases. |

## Gap matrix

The "Today" column records the state before this plan started; the phases below
carry the current status.

| React Router | Today | Target | Phase |
| --- | --- | --- | --- |
| `:id`, `*` | `{id}`, `{*splat}` | both syntaxes, React Router preferred | 1 |
| `:id?` | silently a parameter named `id?` | rejected loudly until optional segments have a priority rule | 4 |
| `path="*"` matches `/` | does not match, so `/` renders nothing | matches, like React Router | 1 |
| Pathless layout route | works, undocumented; a route with children and no element/layout renders nothing | renders the matched child (transparent group) | 1 |
| Missing `init(cx)`, duplicate paths | silent in release, bare `matchit` panic | panic with context | 1 |
| `<Link>` | only `NavLink` | `Link` for plain links | 2 |
| `<Link replace>` | none | `replace(true)` on `Link`/`NavLink` | 4 ✅ ([#43]) |
| `navigate(-1)`, `{ replace: true }` | no history | `Navigator::{push, replace, back, forward}` | 2 |
| `navigate()` re-renders | state changes, nothing repaints | navigation schedules a frame | 4 |
| `useNavigationType()` | none | `Pop`/`Push`/`Replace` for the current entry | 4 ✅ ([#45]) |
| `useBlocker()` | none | guard unsaved changes before leaving a route | 4 |
| `useMatches()`, `useMatch(pattern)` | only `use_pattern` | both hooks | 2 |
| `useSearchParams()`, `search`, `hash` | `Location` only has `pathname`; `to="/x?q=1"` is treated as a literal path | `Location { pathname, search, hash, state, key }` plus read/write hooks | 2 |
| `<Navigate to replace />` | none | `Redirect::to(..).replace(..)` | 2 |
| `relative="route"`, `..` | every `to` is absolute | relative resolution | 2 |
| `location.state` | removed in 0.6 (the old field was unusable) | string map, set through `Navigator` | 2 |
| `useOutletContext` | none | not aligned: GPUI has no context lookup, pass entities instead | — |
| `lazy`, `ScrollRestoration`, case-insensitive matching | none / different | not aligned (web-only, or against Rust conventions); documented | — |
| `loader`, `action`, `errorElement`, `useLoaderData`, `useNavigation`, `useFetcher` | none | design document, then decide | 3 |

## Target API

```rust
// Paths: React Router first, matchit still accepted.
Route::new().path("users/:id")
Route::new().path("users/{id}")     // equivalent
Route::new().path("*")              // splat, and it matches the parent path too
// Route::new().path("docs/:page?") // optional segments are rejected until they
                                    // have a priority rule; see phase 4

// Structure: pathless layout route with an index and a splat, as in React Router.
Routes::new().basename("/app").children(vec![
  Route::new().element(|_, _| shell()).children(vec![
    Route::new().index().element(|_, _| home()),
    Route::new().path("users/:id").element(|_, cx| user_page(cx)),
    Route::new().path("*").element(|_, _| not_found()),
  ]),
])

// Links.
Link::new().to("/about").child("About")
NavLink::new()
  .to("/about")
  .active(|style| style.font_weight(FontWeight::BOLD))
  .end(true)
  .child("About")

// Navigation.
let mut nav = use_navigate(cx);
nav.push("/about");
nav.replace("/login");
nav.back();
nav.forward();

// Declarative redirect.
Redirect::to("/login").replace(true)

// Hooks.
use_location(cx)
use_params(cx)
use_matches(cx)
use_match(cx, "users/:id")
use_pattern(cx)
use_search_params(cx)
use_set_search_params(cx)
```

## Phases

Release mapping: phases 1 and 2 ship **together as 0.6.0**. Phase 2 changes the
return type of `use_navigate` and the meaning of a relative `to`, which are
breaking changes, so they cannot go into a 0.6.x patch. Phase 3 is a document,
and phase 4 is open work.

### Phase 1 — corrections and syntax alignment

1. ✅ Pathless groups render the matched child instead of nothing.
2. ✅ `:id` and `*` compile to the matcher; `{id}` keeps working. `:id?` still needs a priority rule against index and sibling routes, so it panics with the route path until phase 4 instead of compiling to a parameter named `page?` ([#37], [#38]).
3. ✅ A splat route also matches the parent path, so `path="*"` covers `/`. Aliases are registered after every real pattern, so an index or static route wins regardless of declaration order.
4. ✅ `init(cx)` panics in every build when missing; duplicate/invalid paths panic with the route that caused them.
5. ✅ Documentation: the README has a "Coming from React Router?" table, pathless layout routes and `*` semantics are documented, and the React Router nesting style is the recommended one with `Route::layout(...)` marked as an optional alternative.

### Phase 2 — the common API

6. ✅ `Link`, and `NavLink::case_sensitive` with React Router's case-insensitive default.
7. ✅ `Navigator` with `push` / `replace` / `back` / `forward` and a history stack (the last 100 locations).
8. ✅ `use_matches()` and `use_match(pattern)`, with relative patterns resolving like `to` does. ⬜ `Match` still carries only the pattern and the concrete pathname; per-match `params` (and an `id`) are phase 4, so `use_matches` cannot yet replace a React Router breadcrumb as-is.
9. ✅ `Location { pathname, search, hash, key }`, `use_search_params()`, `use_set_search_params()`.
10. ✅ `Redirect::to(..)`, including `replace(true)`.
11. ✅ Relative `to` resolution with `Relative::{Route, Path}`, including `..` climbing a route (`Route`, the default) or a path segment (`Path`). `Link`, `NavLink` and `Redirect` resolve against the route that rendered them, for the whole layout of their element tree ([#35], [#36]).
12. ✅ `gpui_router::prelude`, which the README's usage example now imports.
13. ✅ `Location::state`, set through `Navigator::state`, `Link::state`, `NavLink::state` and `Redirect::state`.

### Phase 3 — data APIs (design first)

✅ The design pass is [data-apis.md](./data-apis.md): it scopes a faithful subset
(`loader`, `error_element`, `use_loader_data`, `use_navigation`), the lifecycle
rules (parallel per navigation, revalidate on parameter or search change,
generation-based cancellation, error boundaries), the effort, and a
recommendation — implement the subset after 0.6.0, skip `action`/`useFetcher`/
`defer`. Whether to implement it is the maintainer's call; Phases 1 and 2 are
complete either way.

### Phase 4 — remaining alignment (open)

Ordered by how much each item changes what an application has to know, not by
effort. Nothing here is committed to a release yet.

1. ⬜ Make navigation schedule a frame ([#41]): navigation that does not
   repaint is a sharp edge a React Router user hits on the first day. ✅
   `Link`/`NavLink` gained `replace(true)` ([#39], [#43]).
2. ⬜ Settle the state model: per-window state, plus a snapshot of
   `matches`/`params` per `Routes` tree so a nested tree cannot rewrite what an
   outer tree recorded ([#40]). Do this before the data APIs, which would store
   loader data in the same state.
3. ⬜ `useBlocker` / `useBeforeUnload` (for unsaved changes). A desktop app
   needs the guard more than a web app does. ✅ `useNavigationType()` landed for
   transitions ([#44], [#45]).
4. ⬜ Per-match `params` and an `id` on `Match`, completing `use_matches`.
5. ⬜ One case-sensitivity rule for both route matching and `NavLink`.
6. ⬜ Optional segments (`:id?`) with a documented priority rule, replacing the
   loud panic from phase 1.
7. ⬜ `Navigate` as the v7 name for `Redirect`, and a `Location::state` that does
   not force applications to serialize values into strings.
8. ⬜ Let an `Outlet` that is built inside a child component's `render` (rather
   than inside the route's element closure) receive the matched child.

## Divergence register

Every React Router API this crate does not mirror exactly, and why. `open` items
are the work in [phase 4](#phase-4--remaining-alignment-open).

| React Router | `gpui-router` | State |
| --- | --- | --- |
| `<Routes>`, `<Route>`, `<Outlet>`, `<Link>`, `<NavLink>`, `useParams`, `useLocation`, `useMatches`, `useMatch`, `useSearchParams` | the same names (snake case) | aligned |
| `<Link replace>` | `Link::new().replace(true)`, `NavLink::replace(true)` | aligned ([#43]) |
| `useNavigationType` | `use_navigation_type(cx)`, a `NavigationType` | aligned ([#45]) |
| `useBlocker`, `useBeforeUnload` | missing | open (phase 4) |
| `navigate()` repaints the screen | `Navigator` mutates state, the caller refreshes the window | open ([#41]) |
| `useOutletContext`, `lazy`, `<ScrollRestoration>` | missing | deliberate: GPUI has no context lookup, and the other two are web-only |
| `<Navigate>` | `Redirect` | deliberate for now: a v5 name; `Navigate` is open work |
| `navigate(to, { relative, state, replace })`, `navigate(-1)` | `Navigator` with `relative`, `state`, `push`, `replace`, `back`, `forward` | adapted: no options object in Rust |
| `location.state` (any value) | `BTreeMap<SharedString, SharedString>` | adapted: values are strings |
| `useSearchParams()` (mutable) | `SearchParams` + `SearchParamsSetter` | adapted |
| one router per browser tab | one `RouterState` per application | open ([#40]) |
| route matching is case-insensitive by default | matching is case-sensitive, `NavLink` matching is not | inconsistent: phase 4 |
| `loader`, `errorElement`, `useNavigation`, `useLoaderData` | design only | open, not committed ([data-apis.md](./data-apis.md)) |
| `action`, `<Form>`, `useSubmit`, `useFetcher` | none | deliberate: mutations belong to entities and commands |

## Verification

The risk this plan has to manage is not "does it compile"; it is "does it behave
like React Router while a real frame is laid out". The relative-target bug fixed
in [#36] is the example: the unit tests called `Link::link_element(cx)`
directly, so they could not see that GPUI renders the components of an element
tree during layout, after the route's element closure returned, and resolution
silently fell back to the deepest match.

Rules for behaviour that only exists inside a frame:

1. A change to matching or navigation adds a frame-level test: draw the route
   tree into a test window, drive it with a simulated click or a draw, and assert
   the resulting `Location`. `crates/router/src/router_tests.rs` has the pattern
   (draw, `simulate_mouse_move`, draw again so hit testing sees the pointer,
   `simulate_click`, assert).
2. Behaviour the crate claims to share with React Router is verified against the
   documented behaviour, not against the implementation's current output; the
   parity matrix below lists where each one lives.
3. `cargo test` on both backends, `cargo clippy -- -D warnings` and the
   `gpui-kit` workflow are the gates. A feature is not done while a job is red.

| React Router behaviour | Covered by |
| --- | --- |
| `users/:id`, `{id}`, `*`, and a splat matching its parent path | `test_dynamic_segments_use_both_syntaxes`, `test_splat_is_exposed_as_star`, `test_catch_all_route_matches_root_with_an_empty_splat` |
| index and static routes win over dynamic and splat routes | `test_static_routes_win_over_dynamic_routes`, `test_index_route_wins_over_a_splat_parent_path` |
| a pathless layout route renders its matched child | `test_element_route_matches_its_own_path_without_an_index_child` |
| a relative `to` resolves against the route that rendered it | `test_layout_relative_link_resolves_against_the_layout_route`, `test_layout_relative_link_after_the_outlet_resolves_against_the_layout_route`, `test_layout_relative_redirect_resolves_against_the_layout_route` |
| `navigate(to, { replace })`, `navigate(-1)`, forward entries | `test_navigator_history` |
| `location.search`, `hash` and `state` | `test_use_search_params`, `test_navigator_location_state` |
| `<Navigate replace>` does not navigate twice | `test_layout_relative_redirect_resolves_against_the_layout_route` (draws the target twice) |
| `<Link replace>` reuses the history entry while a plain link adds one | `test_link_replace_reuses_the_current_history_entry`, `test_link_push_adds_a_history_entry` |
| `useNavigationType` reports how the entry was reached | `test_use_navigation_type` |
| optional segments are not silently mis-parsed | `test_react_router_optional_segments_are_rejected`, `test_matcher_optional_segments_are_rejected` |

## Risks

| Risk | What it costs | Where |
| --- | --- | --- |
| One `RouterState` per application | Windows cannot route independently, and a second `Routes` tree clears the first tree's `matches`/`params`, so outer chrome can read the wrong route | [#40], `crates/router/src/state.rs` |
| Navigation mutates state without a frame | `nav.push` in an event handler or task leaves the UI stale until something else repaints | [#41], `crates/router/src/hooks.rs` |
| Resolution happens while GPUI lays a tree out | A new element that resolves a target or reads the route chain must be wrapped in `RouteScope`, or it silently falls back to the deepest match | [#35], [#36], `crates/router/src/route.rs` |
| `Location::state` is a string map | React Router allows any value; the map exists to keep `Ord`/`PartialOrd` on `Location`, which nothing in the crate uses | Decisions table |
| Route matching is case-sensitive, `NavLink` is not | `/About` highlights the `/about` link but renders nothing | Divergence register |
| `Outlet` is scoped to the element closure | An `Outlet` built inside a child component's `render` renders empty | `crates/router/src/outlet.rs` |
| Matcher cache keys are 64-bit hashes of the route tree | A collision would silently reuse the wrong matcher | `crates/router/src/matcher.rs` |
| Matching allocates per frame | `match_normalized` rebuilds the parameter map, `record_match` allocates the concrete pathname | `crates/router/src/matcher.rs`, `crates/router/src/state.rs` |

## Migration notes

- `{id}` keeps working; `:id` is the documented spelling from 0.6 on.
- `Route::layout(...)` + `#[derive(IntoLayout)]` keep working; new code should
  prefer a pathless layout route with an `Outlet`.
- `Location::state` returns in 0.6.0 with a usable type; the previous field held
  a `matchit::Params` that nothing could write.
- `docs/:page?` and `docs/{page?}` panic instead of compiling to a parameter
  named `page?`; write an index route for `/docs` and a route for `/docs/:page`.

[#35]: https://github.com/justjavac/gpui-router/issues/35
[#36]: https://github.com/justjavac/gpui-router/pull/36
[#37]: https://github.com/justjavac/gpui-router/issues/37
[#38]: https://github.com/justjavac/gpui-router/pull/38
[#39]: https://github.com/justjavac/gpui-router/issues/39
[#40]: https://github.com/justjavac/gpui-router/issues/40
[#41]: https://github.com/justjavac/gpui-router/issues/41
[#43]: https://github.com/justjavac/gpui-router/pull/43
[#44]: https://github.com/justjavac/gpui-router/issues/44
[#45]: https://github.com/justjavac/gpui-router/pull/45

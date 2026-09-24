# React Router alignment

Goal: a React Router user should be able to use `gpui-router` without
translating concepts. Names match, semantics match, and the shapes that cannot
match (Rust has no JSX, no hooks rules, GPUI has no context lookup) are
documented instead of invented.

Work happens feature by feature: a short plan, an implementation, a review, one
pull request per feature, and a `CHANGELOG.md` entry under `Unreleased`.

## Principles

1. **Names first.** `Routes`, `Route`, `Link`, `NavLink`, `Outlet`, `Redirect`,
   `useLocation`, `useParams`, `useMatches`, `useMatch`, `useNavigate`,
   `useSearchParams` map one-to-one onto React Router's API (snake case in
   Rust).
2. **Semantics over syntax.** `navigate(to)`, `navigate(to, { replace })`,
   `navigate(-1)` and `navigate(1)` become one `Navigator` with `push`,
   `replace`, `back` and `forward`.
3. **No concepts React Router does not have.** Pathless layout routes plus
   `Outlet` are the way to nest; `Route::layout(...)` and `#[derive(IntoLayout)]`
   stay as an optional, explicit alternative.
4. **React Router path syntax works out of the box** — `:id`, `*`, and `:id?` —
   because `path="users/:id"` silently matching a literal path today is the
   single most confusing failure mode for a React Router user.
5. **Failures are loud.** A misconfiguration should panic with a message that
   names the route, never render an empty page.

## Decisions

| Question | Decision |
| --- | --- |
| Path syntax | Accept `:id` / `*` / `:id?` (React Router) **and** `{id}` / `{*splat}` (matchit). |
| `Location::state` | A string map (`Option<HashMap<SharedString, SharedString>>`); keeps `Clone` and `PartialEq`. |
| `Link` | Add it. React Router users look for both `Link` and `NavLink`. |
| Relative `to` | Default to React Router's `relative="route"`; `Relative::Path` selects the absolute behaviour. |
| History | `push` / `replace` / `back` / `forward` land in Phase 2. |
| Data APIs | `loader` / `action` / `errorElement` need a design document first; not part of the initial phases. |

## Gap matrix

| React Router | Today | Target | Phase |
| --- | --- | --- | --- |
| `:id`, `*`, `:id?` | `{id}`, `{*splat}` | both syntaxes, React Router preferred | 1 |
| `path="*"` matches `/` | does not match, so `/` renders nothing | matches, like React Router | 1 |
| Pathless layout route | works, undocumented; a route with children and no element/layout renders nothing | renders the matched child (transparent group) | 1 |
| Missing `init(cx)`, duplicate paths | silent in release, bare `matchit` panic | panic with context | 1 |
| `<Link>` | only `NavLink` | `Link` for plain links | 2 |
| `navigate(-1)`, `{ replace: true }` | no history | `Navigator::{push, replace, back, forward}` | 2 |
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
Route::new().path("docs/:page?")    // optional segment, expanded while compiling
Route::new().path("*")              // splat, and it matches the parent path too

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

### Phase 1 — corrections and syntax alignment (0.6.0)

1. ✅ Pathless groups render the matched child instead of nothing.
2. ✅ `:id` and `*` compile to the matcher; `{id}` keeps working. `:id?` is a follow-up, because an optional segment can collide with an index or sibling route and needs an explicit priority rule.
3. ✅ A splat route also matches the parent path, so `path="*"` covers `/`. Aliases are registered after every real pattern, so an index or static route wins regardless of declaration order.
4. ✅ `init(cx)` panics in every build when missing; duplicate/invalid paths panic with the route that caused them.
5. ✅ Documentation: the README has a "Coming from React Router?" table, pathless layout routes and `*` semantics are documented, and the React Router nesting style is the recommended one with `Route::layout(...)` marked as an optional alternative.

### Phase 2 — the common API (0.6.x)

6. ✅ `Link`, and `NavLink::case_sensitive` with React Router's case-insensitive default.
7. `Navigator` with `push` / `replace` / `back` / `forward` and a history stack.
8. `use_matches()` and `use_match(pattern)`.
9. `Location { pathname, search, hash, state, key }`, `use_search_params()`, `use_set_search_params()`.
10. `Redirect::to(..)`.
11. Relative `to` resolution with `Relative::{Route, Path}`.
12. `gpui_router::prelude`.

### Phase 3 — data APIs (design first)

`loader`, `action`, `errorElement`, `use_loader_data`, `use_navigation`. These
have to fit GPUI's async and entity model (when a match change triggers a
loader, where loading state lives, how errors surface). Skipping them does not
change the routing mental model, so they land after Phases 1 and 2.

## Migration notes

- `{id}` keeps working; `:id` is the documented spelling from 0.6 on.
- `Route::layout(...)` + `#[derive(IntoLayout)]` keep working; new code should
  prefer a pathless layout route with an `Outlet`.
- `Location::state` returns in Phase 2 with a usable type. The 0.6 removal only
  drops the previous `matchit::Params` field, which nothing could write.

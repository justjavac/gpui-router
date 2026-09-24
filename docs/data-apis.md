# Data APIs: `loader`, `action`, `errorElement`

Phase 3 of the [React Router alignment plan](./react-router-alignment.md) is a
design pass, not an implementation: React Router's data APIs are large, and a
desktop framework that already has an async runtime has to justify growing them.
This document records the scope, the API a faithful subset would have, the
lifecycle rules, the effort, and a recommendation.

## What React Router's data APIs provide

| API | What it does |
| --- | --- |
| `loader` | Runs during a navigation for each matched route, so a page renders with data instead of an empty state and a follow-up render. |
| `action` | Runs a mutation for a route, usually from a `<Form>` or `useSubmit`, then revalidates the loaders. |
| `errorElement` / `useRouteError` | Renders a route's error state when its loader or action fails, with the nearest boundary winning. |
| `useNavigation` | Tells the UI whether a navigation is idle or loading, so a progress bar is one hook away. |
| `useFetcher`, `defer`, `Await` | Fetching and streaming outside a navigation. |

The value is co-location: the route owns how its data is read, and the router
owns when it is read and how in-flight reads are cancelled.

## What GPUI already provides

- `cx.spawn` and `background_spawn` for async work, with entities as the place to
  put results.
- Immediate-mode rendering, so a loader can never block a frame; a design has to
  render now and update when data arrives, which is what React Router does while
  a navigation is loading.

The question is therefore not "how do we fetch data in Rust" but "should the
router own fetching, loading state and error boundaries". Today an application
keeps data in an entity and calls `cx.spawn`; the router adds nothing. The
reason to add loaders is parity: a React Router application that uses them
should not have to be restructured.

## Proposed subset

In scope if this is implemented:

```rust
Route::new()
  .path("users/:id")
  .loader(|params, cx| async move { fetch_user(params.get("id")).await })
  .error_element(|error, cx| error_page(error, cx))
  .element(|_, cx| user_page(cx))

fn user_page(cx: &App) -> impl IntoElement {
  match use_loader_data::<User>(cx) {
    Some(user) => profile(user),
    None => Spinner::new(), // still loading; the previous page stays visible
  }
}
```

- `Route::loader(...)`: an async closure that receives the route's parameters
  and returns its data. The data is type-erased (`Rc<dyn Any>`), because a route
  tree is not generic over its loaders; `use_loader_data::<T>(cx)` downcasts, and
  a mismatch panics with the route pattern in the message.
- `Route::error_element(...)`: renders instead of the route's element when its
  own or a descendant's loader fails, and `use_route_error(cx)` exposes the
  error to an element that wants to render it inline.
- `use_navigation(cx)`: `NavigationState::{Idle, Loading}`, so a progress
  indicator is one hook away.
- The router runs the loaders of the whole matched chain in parallel when the
  location changes, keeps rendering immediately (the previous page stays
  visible), and discards results whose generation is stale because the user
  navigated again.

Out of scope: `action`, `<Form>`, `useSubmit` and `useFetcher` (mutations belong
to entities and commands in a desktop application, and being faithful would
mean growing a form abstraction), `defer`/`Await` (streaming partial data),
`useRevalidator` (an explicit revalidate could come later, if at all), and
`shouldRevalidate` (replaced by the fixed rule below).

## Lifecycle rules (if implemented)

1. **When.** Loaders run on the initial render and on every location change,
   after matching and before the matched chain renders.
2. **Which.** Every route in the matched chain that has a loader, and only when
   that route's own parameters or search parameters changed compared to the
   previous match. A route whose inputs did not change keeps its data, which is
   React Router's default `shouldRevalidate` behaviour and avoids refetching a
   layout's data on every child navigation.
3. **Cancellation.** Each navigation gets a generation number. Results from an
   older generation never reach the state, and the task handles are dropped,
   which cancels the futures.
4. **Failure.** A failed loader marks the chain up to the nearest route with an
   `error_element`; without one, the route's element sees `use_route_error`, and
   otherwise the failure is logged and nothing renders.
5. **State.** Loader data lives per route pattern in `RouterState`, next to
   `params` and `matches`, so `use_matches` can grow per-match `data` later
   without another state channel.

## Effort and risk

| Piece | Effort | Risk |
| --- | --- | --- |
| Loader registration, generations, parallel execution, per-route caching | one PR | Async tests with the GPUI test executor; cancellation semantics need care |
| `error_element` and `use_route_error` | one PR | Boundaries interact with outlet injection and pathless groups |
| `use_navigation`, docs and a runnable example | one PR | Low |
| Per-match `data` in `use_matches` (optional) | small | Low |

## Recommendation

1. **Do not add the data APIs before 0.6.0 ships.** That release already carries
   breaking changes to `Location`, `use_navigate` and relative targets; a release
   that also changes how data is fetched is harder to review and to revert.
2. **Then implement the subset above, not the mutation side.** `loader`,
   `error_element`, `use_navigation` and `use_loader_data` are what a React
   Router user notices immediately, and they need route ownership; mutations fit
   GPUI's entity model better than a router-owned `action`.
3. **Keep the documented alternative.** Until then, the README should say how to
   do what a loader does today: hold the data in an entity, fetch with
   `cx.spawn`, and render the loading state from that entity.

If the answer is "do not implement", the alignment plan should record the data
APIs as a deliberate divergence, like `useOutletContext` and `lazy`.

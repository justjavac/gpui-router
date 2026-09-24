//! Compiles a route tree into a `matchit` matcher and caches the result.
//!
//! `Routes` and `Route` are elements, so applications rebuild them on every
//! render. Compiling the matcher on every render is wasteful — a 500 route tree
//! costs ~230 µs per level and frame — so compiled matchers are cached by a
//! fingerprint of the route tree: rebuilding a tree with the same paths reuses
//! the compiled matcher.

use crate::Route;
#[cfg(test)]
use crate::normalize_pathname;
use gpui::SharedString;
use hashbrown::HashMap;
use matchit::Router as MatchitRouter;
use std::{
  cell::RefCell,
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
  rc::Rc,
};

/// How many compiled matchers to keep around. Route trees are shallow, so this
/// covers every level of a few different trees.
const MAX_CACHED_MATCHERS: usize = 8;

/// A route as it appears in a route list: `Routes` stores routes by value,
/// while nested routes are boxed.
pub(crate) trait RouteNode {
  fn route(&self) -> &Route;
}

impl RouteNode for Route {
  fn route(&self) -> &Route {
    self
  }
}

impl RouteNode for Box<Route> {
  fn route(&self) -> &Route {
    self
  }
}

/// What a compiled matcher maps a pattern to: the index of the matching route
/// in the list it was compiled from, plus the pattern itself.
#[derive(Clone, Debug, PartialEq, Eq)]
struct RouteTarget {
  index: usize,
  pattern: SharedString,
}

type Matcher = MatchitRouter<RouteTarget>;
type RouteMap = MatchitRouter<RouteTarget>;

/// The route that matches a pathname, together with its dynamic parameters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MatchedRoute {
  /// Index of the matched route in the list it was matched against.
  pub(crate) index: usize,
  /// The route pattern that matched, for example `/users/{id}`.
  pub(crate) pattern: SharedString,
  /// The names and values of dynamic parameters in the pathname.
  pub(crate) params: HashMap<SharedString, SharedString>,
}

struct CachedMatcher {
  fingerprint: u64,
  matcher: Rc<Matcher>,
}

thread_local! {
  static MATCHER_CACHE: RefCell<Vec<CachedMatcher>> = const { RefCell::new(Vec::new()) };
}

#[cfg(test)]
thread_local! {
  /// Number of matchers compiled on this thread, for tests that assert the
  /// cache is reused. It is thread local because the cache is too.
  pub(crate) static BUILD_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Finds the route in `routes` that matches `pathname`, which must already be
/// normalized. This is the path taken while rendering, so it allocates nothing.
pub(crate) fn match_normalized<Node: RouteNode>(
  routes: &[Node],
  basename: &str,
  pathname: &str,
) -> Option<MatchedRoute> {
  if routes.is_empty() {
    return None;
  }

  let matcher = cached_matcher(routes, basename);
  let matched = matcher.at(pathname.as_ref()).ok()?;
  let params = matched
    .params
    .iter()
    .map(|(key, value)| (key.to_owned().into(), value.to_owned().into()))
    .collect();

  Some(MatchedRoute {
    index: matched.value.index,
    pattern: matched.value.pattern.clone(),
    params,
  })
}

/// Finds the route in `routes` that matches `pathname`, normalizing it first.
#[cfg(test)]
pub(crate) fn match_path<Node: RouteNode>(routes: &[Node], basename: &str, pathname: &str) -> Option<MatchedRoute> {
  match_normalized(routes, basename, normalize_pathname(pathname).as_ref())
}

fn cached_matcher<Node: RouteNode>(routes: &[Node], basename: &str) -> Rc<Matcher> {
  let fingerprint = fingerprint(routes, basename);

  let cached = MATCHER_CACHE.with(|cache| {
    cache
      .borrow()
      .iter()
      .find(|entry| entry.fingerprint == fingerprint)
      .map(|entry| Rc::clone(&entry.matcher))
  });
  if let Some(matcher) = cached {
    return matcher;
  }

  let matcher = Rc::new(compile(routes, basename));
  #[cfg(test)]
  BUILD_COUNT.with(|count| count.set(count.get() + 1));

  MATCHER_CACHE.with(|cache| {
    let mut cache = cache.borrow_mut();
    cache.insert(
      0,
      CachedMatcher {
        fingerprint,
        matcher: Rc::clone(&matcher),
      },
    );
    cache.truncate(MAX_CACHED_MATCHERS);
  });

  matcher
}

fn compile<Node: RouteNode>(routes: &[Node], basename: &str) -> Matcher {
  let mut matcher = MatchitRouter::new();
  for (index, node) in routes.iter().enumerate() {
    matcher.merge(route_map(node.route(), basename, index)).unwrap();
  }
  matcher
}

/// Registers a route's own path and its descendants, mapping every pattern to
/// the index this route has in its parent's list.
fn route_map(route: &Route, basename: &str, index: usize) -> RouteMap {
  let mut map = MatchitRouter::new();
  let path = route.full_path(basename);

  // Descendants belong to the route being registered, so they keep its index.
  for child in route.routes.iter() {
    map.merge(route_map(child, path.as_ref(), index)).unwrap();
  }

  // A route that renders an element also matches its own path, so the element
  // can render with an empty outlet when no child matches. A child that
  // registers the same path wins, which is what an index route does.
  if route.element.is_some() {
    let _ = map.insert(
      path.as_ref(),
      RouteTarget {
        index,
        pattern: path.clone(),
      },
    );
  }

  map
}

fn fingerprint<Node: RouteNode>(routes: &[Node], basename: &str) -> u64 {
  let mut hasher = DefaultHasher::new();
  basename.hash(&mut hasher);
  hash_routes(routes, &mut hasher);
  hasher.finish()
}

fn hash_routes<Node: RouteNode>(routes: &[Node], hasher: &mut impl Hasher) {
  routes.len().hash(hasher);
  for node in routes {
    let route = node.route();
    route.path.hash(hasher);
    route.element.is_some().hash(hasher);
    hash_routes(&route.routes, hasher);
  }
}

#[cfg(test)]
mod tests {
  use super::{BUILD_COUNT, compile, match_path};
  use crate::Route;
  use std::time::Instant;

  fn build_count() -> usize {
    BUILD_COUNT.with(|count| count.get())
  }

  fn routes() -> Vec<Route> {
    vec![
      Route::new().path("/").element(|_, _| "layout").children(vec![
        Route::new().index().element(|_, _| "home"),
        Route::new().path("about").element(|_, _| "about"),
      ]),
      Route::new().path("{*not_match}").element(|_, _| "not_match"),
    ]
  }

  fn wide_routes(count: usize) -> Vec<Route> {
    (0..count)
      .map(|index| {
        let path = if index % 5 == 0 {
          format!("section{index}/{{id}}")
        } else {
          format!("section{index}")
        };
        Route::new().path(path).element(|_, _| "page")
      })
      .collect()
  }

  /// Timing harness for the matcher cache. Run with:
  /// `cargo test --release --features runtime_shaders -- --ignored --nocapture`
  #[test]
  #[ignore = "timing harness, run manually with --ignored --nocapture"]
  fn bench_route_matching() {
    const ITERATIONS: u32 = 500;

    for count in [50usize, 200, 500] {
      let routes = wide_routes(count);
      let pathname = "/section1";

      let _ = match_path(&routes, "/", pathname);
      let start = Instant::now();
      for _ in 0..ITERATIONS {
        let _ = match_path(&routes, "/", pathname);
      }
      let cached = start.elapsed().as_secs_f64() * 1e6 / f64::from(ITERATIONS);

      let start = Instant::now();
      for _ in 0..ITERATIONS {
        let matcher = compile(&routes, "/");
        let _ = matcher.at(pathname);
      }
      let rebuilt = start.elapsed().as_secs_f64() * 1e6 / f64::from(ITERATIONS);

      println!(
        "routes={count:>4}  cached={cached:>8.2} µs  rebuild={rebuilt:>8.2} µs  ({:.0}x)",
        rebuilt / cached.max(0.001)
      );
    }
  }

  #[test]
  fn test_matching_reuses_the_compiled_matcher() {
    let routes = routes();

    let first = match_path(&routes, "/", "/about").unwrap();
    assert_eq!(first.index, 0);
    assert_eq!(first.pattern, "/about");
    assert!(first.params.is_empty());
    let builds = build_count();

    let second = match_path(&routes, "/", "/users/42").unwrap();
    assert_eq!(second.index, 1);
    assert_eq!(second.pattern, "/{*not_match}");
    assert_eq!(build_count(), builds);
  }

  #[test]
  fn test_rebuilding_the_same_route_tree_reuses_the_matcher() {
    let _ = match_path(&routes(), "/", "/about").unwrap();
    let before = build_count();
    let _ = match_path(&routes(), "/", "/about").unwrap();

    assert_eq!(
      build_count(),
      before,
      "an identical route tree should reuse the cached matcher"
    );
  }

  #[test]
  fn test_changed_route_tree_recompiles() {
    let _ = match_path(&routes(), "/", "/about").unwrap();
    let before = build_count();

    let changed = vec![Route::new().path("other").element(|_, _| "other")];
    let _ = match_path(&changed, "/", "/other").unwrap();

    assert!(build_count() > before);
  }
}

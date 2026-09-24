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
  /// Name of the splat parameter when the pattern ends in one.
  splat: Option<SharedString>,
  /// Whether the application wrote the splat as React Router's `*`.
  react_router_splat: bool,
  /// Set for the extra pattern that lets a splat match its parent path; the
  /// splat parameter is then present but empty.
  empty_splat: bool,
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
  let target = &matched.value;
  let mut params: HashMap<SharedString, SharedString> = matched
    .params
    .iter()
    .map(|(key, value)| (key.to_owned().into(), value.to_owned().into()))
    .collect();

  if let Some(splat) = target.splat.as_ref() {
    if target.empty_splat {
      params.insert(splat.clone(), SharedString::default());
    }

    // React Router names the splat parameter `*`, so expose it under that name
    // as well when the route was written as `path("*")`.
    if target.react_router_splat
      && let Some(value) = params.get(splat).cloned()
    {
      params.insert(SharedString::from("*"), value);
    }
  }

  Some(MatchedRoute {
    index: target.index,
    pattern: target.pattern.clone(),
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

  // Splat routes also match their parent path, where the splat is empty. Those
  // aliases are merged last and never win a conflict, so an index or static
  // sibling route keeps the path however the tree is ordered.
  for (index, node) in routes.iter().enumerate() {
    let _ = matcher.merge(splat_parent_map(node.route(), basename, index));
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
    let pattern = CompiledPattern::new(path.as_ref());
    let _ = map.insert(pattern.matcher.as_str(), pattern.route_target(index, path, false));
  }

  map
}

/// The extra patterns that let a splat route match its own parent path, for
/// example `/files` for `/files/*`.
fn splat_parent_map(route: &Route, basename: &str, index: usize) -> RouteMap {
  let mut map = MatchitRouter::new();
  let path = route.full_path(basename);

  for child in route.routes.iter() {
    let _ = map.merge(splat_parent_map(child, path.as_ref(), index));
  }

  if route.element.is_some() {
    let pattern = CompiledPattern::new(path.as_ref());
    if let Some(parent) = pattern.parent() {
      let _ = map.insert(parent.as_str(), pattern.route_target(index, path, true));
    }
  }

  map
}

/// Translates a React Router path pattern into the syntax the matcher uses:
/// `:id` becomes `{id}` and `*` becomes `{*splat}`. Patterns that already use
/// the `{id}` / `{*splat}` syntax pass through unchanged.
/// A route path as the application wrote it, together with the pattern it
/// compiles to.
struct CompiledPattern {
  /// The pattern in the matcher's syntax.
  matcher: String,
  /// The name of the splat parameter, when the pattern has one.
  splat: Option<SharedString>,
  /// Whether the splat was written as React Router's `*`.
  react_router_splat: bool,
}

impl CompiledPattern {
  /// Translates a React Router path pattern into the syntax the matcher uses:
  /// `:id` becomes `{id}` and `*` becomes `{*splat}`. Patterns that already use
  /// the `{id}` / `{*splat}` syntax pass through unchanged.
  fn new(path: &str) -> Self {
    let mut matcher = String::with_capacity(path.len());
    let mut splat = None;
    let mut react_router_splat = false;

    for (index, segment) in path.split('/').enumerate() {
      if index > 0 {
        matcher.push('/');
      }

      if segment == "*" {
        react_router_splat = true;
        splat = Some(SharedString::from("splat"));
        matcher.push_str("{*splat}");
      } else if let Some(name) = segment.strip_prefix("{*").and_then(|rest| rest.strip_suffix('}')) {
        splat = Some(SharedString::from(name.to_owned()));
        matcher.push_str(segment);
      } else if let Some(name) = segment.strip_prefix(':') {
        matcher.push('{');
        matcher.push_str(name);
        matcher.push('}');
      } else {
        matcher.push_str(segment);
      }
    }

    Self {
      matcher,
      splat,
      react_router_splat,
    }
  }

  /// The pattern without its trailing splat segment, which React Router's `*`
  /// also matches.
  fn parent(&self) -> Option<String> {
    self.splat.as_ref()?;
    let (parent, _) = self.matcher.rsplit_once('/')?;

    Some(if parent.is_empty() {
      "/".to_string()
    } else {
      parent.to_string()
    })
  }

  fn route_target(&self, index: usize, pattern: SharedString, empty_splat: bool) -> RouteTarget {
    RouteTarget {
      index,
      pattern,
      splat: self.splat.clone(),
      react_router_splat: self.react_router_splat,
      empty_splat,
    }
  }
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
  use super::{BUILD_COUNT, CompiledPattern, compile, match_path};
  use crate::Route;
  use gpui::SharedString;
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

  #[test]
  fn test_react_router_path_syntax() {
    assert_eq!(CompiledPattern::new("/users/:id").matcher, "/users/{id}");
    assert_eq!(CompiledPattern::new("/files/*").matcher, "/files/{*splat}");
    assert_eq!(CompiledPattern::new("/").matcher, "/");
    // matchit syntax and partial segments pass through.
    assert_eq!(CompiledPattern::new("/users/{id}").matcher, "/users/{id}");
    assert_eq!(CompiledPattern::new("/time:now").matcher, "/time:now");
  }

  #[test]
  fn test_dynamic_segments_use_both_syntaxes() {
    for path in ["users/:id", "users/{id}"] {
      let routes = vec![Route::new().path(path).element(|_, _| "user")];
      let matched = match_path(&routes, "/", "/users/42").unwrap();

      assert_eq!(matched.pattern, SharedString::from(format!("/{path}")));
      assert_eq!(matched.params.get("id").map(|value| value.as_ref()), Some("42"));
    }
  }

  #[test]
  fn test_splat_is_exposed_as_star() {
    let routes = vec![Route::new().path("files/*").element(|_, _| "files")];
    let matched = match_path(&routes, "/", "/files/a/b.txt").unwrap();

    assert_eq!(matched.pattern, "/files/*");
    assert_eq!(matched.params.get("splat").map(|value| value.as_ref()), Some("a/b.txt"));
    assert_eq!(matched.params.get("*").map(|value| value.as_ref()), Some("a/b.txt"));
  }
}

#[cfg(test)]
pub mod tests {
  use crate::{Layout, Outlet, Route, RouterState, Routes, normalize_pathname};
  use gpui::prelude::*;
  use gpui::{AnyElement, App, TestAppContext, VisualTestContext, Window};

  struct Basic {}

  #[derive(Default)]
  struct TestLayout {
    outlet: Outlet,
  }

  impl Layout for TestLayout {
    fn outlet(&mut self, element: AnyElement) {
      self.outlet = element.into();
    }

    fn render_layout(self: Box<Self>, _window: &mut Window, _cx: &mut App) -> AnyElement {
      self.outlet.into_any_element()
    }
  }

  impl Render for Basic {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
      Routes::new()
        .basename("/")
        .child(Route::new().index().element(|_, _| "home"))
        .child(Route::new().path("about").element(|_, _| "about"))
        .child(Route::new().path("dashboard").element(|_, _| "dashboard"))
        .child(Route::new().path("{*not_match}").element(|_, _| "not_match"))
    }
  }

  #[gpui::test]
  async fn test_router(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);
      assert_eq!(cx.global::<RouterState>().location.pathname, "/");
    });
    let window = cx.add_window(|_, _cx| Basic {});

    {
      let mut cx = VisualTestContext::from_window(window.into(), cx);
      assert!(!cx.simulate_close());
    }

    let view = cx.new(|_cx| {
      Routes::new()
        .basename("/")
        .child(Route::new().index().element(|_, _| "home"))
        .child(Route::new().path("about").element(|_, _| "about"))
        .child(Route::new().path("dashboard").element(|_, _| "dashboard"))
        .child(Route::new().path("{*not_match}").element(|_, _| "not_match"))
    });
    view.update(cx, |this, cx| {
      assert_eq!(cx.global::<RouterState>().location.pathname, "/");
      assert_eq!(this.routes().len(), 4);
    })
  }

  #[gpui::test]
  async fn test_lazy_element_evaluation(cx: &mut TestAppContext) {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    cx.update(|cx| {
      crate::init(cx);
    });

    // Counter to track how many times each element function is called
    let home_counter = Arc::new(AtomicU32::new(0));
    let about_counter = Arc::new(AtomicU32::new(0));

    let home_counter_clone = home_counter.clone();
    let about_counter_clone = about_counter.clone();

    // Create routes with elements that increment counters when called
    let _view = cx.new(|_cx| {
      Routes::new()
        .basename("/")
        .child(Route::new().index().element(move |_, _| {
          home_counter_clone.fetch_add(1, Ordering::SeqCst);
          "home"
        }))
        .child(Route::new().path("about").element(move |_, _| {
          about_counter_clone.fetch_add(1, Ordering::SeqCst);
          "about"
        }))
    });

    // At this point, neither element function should have been called yet
    // because we only created the Routes structure, not rendered it
    assert_eq!(
      home_counter.load(Ordering::SeqCst),
      0,
      "Home element should not be evaluated during route configuration"
    );
    assert_eq!(
      about_counter.load(Ordering::SeqCst),
      0,
      "About element should not be evaluated during route configuration"
    );
  }

  #[gpui::test]
  async fn test_static_routes_win_over_dynamic_routes(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      let routes = Routes::new()
        .basename("/")
        .child(Route::new().path("{id}").element(|_, _| "dynamic"))
        .child(Route::new().path("settings").element(|_, _| "settings"));

      let matched = routes.match_route("/settings").unwrap();
      Routes::apply_match(cx, normalize_pathname("/settings"), Some(&matched));

      assert_eq!(matched.pattern, "/settings");
      assert!(cx.global::<RouterState>().params.is_empty());
    });
  }

  #[gpui::test]
  async fn test_route_params_are_cleared_when_next_match_has_none(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      let routes = Routes::new()
        .basename("/")
        .child(Route::new().path("user/{id}").element(|_, _| "user"))
        .child(Route::new().path("about").element(|_, _| "about"));

      let user_match = routes.match_route("/user/42").unwrap();
      Routes::apply_match(cx, normalize_pathname("/user/42"), Some(&user_match));
      assert_eq!(
        cx.global::<RouterState>().params.get("id").map(|value| value.as_ref()),
        Some("42")
      );

      let about_match = routes.match_route("/about").unwrap();
      Routes::apply_match(cx, normalize_pathname("/about"), Some(&about_match));
      assert!(cx.global::<RouterState>().params.is_empty());
      assert_eq!(cx.global::<RouterState>().location.pathname, "/about");
    });
  }

  #[gpui::test]
  async fn test_apply_match_clears_params_when_route_is_unmatched(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      let routes = Routes::new()
        .basename("/")
        .child(Route::new().path("user/{id}").element(|_, _| "user"));

      let user_match = routes.match_route("/user/42").unwrap();
      Routes::apply_match(cx, normalize_pathname("/user/42"), Some(&user_match));
      assert_eq!(
        cx.global::<RouterState>().params.get("id").map(|value| value.as_ref()),
        Some("42")
      );

      Routes::apply_match(cx, normalize_pathname("/missing"), None);
      assert!(cx.global::<RouterState>().params.is_empty());
      assert_eq!(cx.global::<RouterState>().location.pathname, "/missing");
    });
  }

  #[gpui::test]
  async fn test_pathnames_are_normalized_before_matching(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      let routes = Routes::new()
        .basename("/")
        .child(Route::new().path("about").element(|_, _| "about"));

      let matched = routes.match_route("/about/").unwrap();
      Routes::apply_match(cx, normalize_pathname("/about/"), Some(&matched));

      assert_eq!(matched.pattern, "/about");
      assert_eq!(cx.global::<RouterState>().location.pathname, "/about");
    });
  }

  #[gpui::test]
  async fn test_match_route_returns_none_for_unknown_path_without_fallback(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      let routes = Routes::new()
        .basename("/")
        .child(Route::new().index().element(|_, _| "home"))
        .child(Route::new().path("about").element(|_, _| "about"));

      assert!(routes.match_route("/missing").is_none());
    });
  }

  #[gpui::test]
  async fn test_basename_matching_supports_relative_and_trailing_slashes(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      let routes = Routes::new()
        .basename("app/")
        .child(Route::new().index().element(|_, _| "home"))
        .child(Route::new().path("settings").element(|_, _| "settings"));

      let index_match = routes.match_route("/app").unwrap();
      assert_eq!(index_match.pattern, "/app");

      let settings_match = routes.match_route("/app/settings/").unwrap();
      assert_eq!(settings_match.pattern, "/app/settings");
    });
  }

  #[gpui::test]
  async fn test_nested_layout_static_routes_win_over_dynamic_siblings(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      let routes = Routes::new().child(
        Route::new().layout(TestLayout::default()).child(
          Route::new()
            .path("users")
            .layout(TestLayout::default())
            .child(Route::new().path("{id}").element(|_, _| "dynamic"))
            .child(Route::new().path("settings").element(|_, _| "settings")),
        ),
      );

      let matched = routes.match_route("/users/settings").unwrap();
      assert_eq!(matched.pattern, "/users/settings");
      assert!(matched.params.is_empty());
    });
  }

  #[gpui::test]
  async fn test_nested_index_route_matches_parent_path(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      let routes = Routes::new().child(
        Route::new().child(
          Route::new()
            .path("users")
            .layout(TestLayout::default())
            .child(Route::new().index().element(|_, _| "users-index")),
        ),
      );

      let matched = routes.match_route("/users").unwrap();
      assert_eq!(matched.pattern, "/users");
    });
  }

  #[gpui::test]
  async fn test_wildcard_routes_capture_remaining_segments(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      let routes = Routes::new()
        .basename("/")
        .child(Route::new().path("files/{*path}").element(|_, _| "files"));

      let matched = routes.match_route("/files/docs/readme.md").unwrap();
      assert_eq!(matched.pattern, "/files/{*path}");
      assert_eq!(
        matched.params.get("path").map(|value| value.as_ref()),
        Some("docs/readme.md")
      );
    });
  }

  #[gpui::test]
  async fn test_exact_routes_win_over_wildcards(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      let routes = Routes::new()
        .basename("/")
        .child(Route::new().path("files/new").element(|_, _| "new"))
        .child(Route::new().path("files/{*path}").element(|_, _| "files"));

      let matched = routes.match_route("/files/new").unwrap();
      assert_eq!(matched.pattern, "/files/new");
      assert!(matched.params.is_empty());
    });
  }

  #[gpui::test]
  async fn test_nested_wildcard_routes_match_unknown_descendants(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      let routes = Routes::new().child(
        Route::new().layout(TestLayout::default()).child(
          Route::new()
            .path("docs")
            .layout(TestLayout::default())
            .child(Route::new().path("intro").element(|_, _| "intro"))
            .child(Route::new().path("{*rest}").element(|_, _| "rest")),
        ),
      );

      let matched = routes.match_route("/docs/guides/install/windows").unwrap();
      assert_eq!(matched.pattern, "/docs/{*rest}");
      assert_eq!(
        matched.params.get("rest").map(|value| value.as_ref()),
        Some("guides/install/windows")
      );
    });
  }

  #[gpui::test]
  async fn test_multiple_dynamic_params_are_extracted(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      let routes = Routes::new()
        .basename("/")
        .child(Route::new().path("org/{org}/repo/{repo}").element(|_, _| "repo"));

      let matched = routes.match_route("/org/openai/repo/gpt-5").unwrap();
      assert_eq!(matched.pattern, "/org/{org}/repo/{repo}");
      assert_eq!(matched.params.get("org").map(|value| value.as_ref()), Some("openai"));
      assert_eq!(matched.params.get("repo").map(|value| value.as_ref()), Some("gpt-5"));
    });
  }

  #[gpui::test]
  async fn test_dynamic_routes_require_the_full_segment_structure(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      let routes = Routes::new()
        .basename("/")
        .child(Route::new().path("users/{id}").element(|_, _| "user"));

      assert!(routes.match_route("/users").is_none());
    });
  }

  #[gpui::test]
  async fn test_catch_all_route_does_not_match_root_without_segments(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);

      let routes = Routes::new()
        .basename("/")
        .child(Route::new().path("{*rest}").element(|_, _| "rest"));

      assert!(routes.match_route("/").is_none());
    });
  }
}

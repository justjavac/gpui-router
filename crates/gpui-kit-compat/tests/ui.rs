use gpui_kit::component::Root;
use gpui_kit::test::TestWindowExt;
use gpui_kit::{Context, TestAppContext, prelude::*, px, size};
use gpui_kit_compat::{DemoApp, GroupingApp, OutletApp, RedirectApp, root};

#[gpui_kit::test]
fn router_renders_kit_components_and_navigates(cx: &mut TestAppContext) {
  cx.update(gpui_kit::init);
  cx.update(gpui_router::init);

  let handle = cx.open_window(size(px(640.), px(480.)), |window, cx: &mut Context<Root>| {
    let view = cx.new(|_| DemoApp);
    root(view, window, cx)
  });

  cx.update_window(handle.into(), |_, window, cx| {
    window.render_frame(cx);
    assert_eq!(window.find("page").label(), Some("home"));
    assert!(
      window.try_find("cta").is_some(),
      "the routed page should render a gpui-kit button"
    );

    window.click("nav-about", cx);
    window.render_frame(cx);
    assert_eq!(window.find("page").label(), Some("about"));
  })
  .unwrap();
}

#[gpui_kit::test]
fn unknown_paths_fall_back_to_the_wildcard_route(cx: &mut TestAppContext) {
  cx.update(gpui_kit::init);
  cx.update(gpui_router::init);

  let handle = cx.open_window(size(px(640.), px(480.)), |window, cx: &mut Context<Root>| {
    let view = cx.new(|_| DemoApp);
    root(view, window, cx)
  });

  cx.update_window(handle.into(), |_, window, cx| {
    window.render_frame(cx);
    {
      let mut navigate = gpui_router::use_navigate(cx);
      navigate.push("/nothing-here");
    }
    window.render_frame(cx);
    assert_eq!(window.find("page").label(), Some("not-match"));
  })
  .unwrap();
}

#[gpui_kit::test]
fn element_routes_render_their_children_through_the_outlet(cx: &mut TestAppContext) {
  cx.update(gpui_kit::init);
  cx.update(gpui_router::init);

  let handle = cx.open_window(size(px(640.), px(480.)), |window, cx: &mut Context<Root>| {
    let view = cx.new(|_| OutletApp);
    root(view, window, cx)
  });

  cx.update_window(handle.into(), |_, window, cx| {
    window.render_frame(cx);
    assert_eq!(window.find("page").label(), Some("home"));

    window.click("nav-about", cx);
    window.render_frame(cx);
    assert_eq!(window.find("page").label(), Some("about"));

    // The "about" route nests another element route, so this exercises two
    // levels of outlet injection.
    window.click("nav-team", cx);
    window.render_frame(cx);
    assert_eq!(window.find("page").label(), Some("team"));
    assert_eq!(
      window.find("breadcrumbs").label(),
      Some("/ > /about > /about/team"),
      "use_matches reports the chain from the root to the leaf"
    );

    window.click("nav-about", cx);
    window.render_frame(cx);
    assert_eq!(window.find("page").label(), Some("about"));

    // A layout route nested inside the element route.
    window.click("nav-settings", cx);
    window.render_frame(cx);
    assert_eq!(window.find("page").label(), Some("settings-profile"));

    // A plain `Link` navigates too.
    window.click("link-team", cx);
    window.render_frame(cx);
    assert_eq!(window.find("page").label(), Some("team"));

    // History: going back returns to the location before the link.
    {
      let mut navigate = gpui_router::use_navigate(cx);
      navigate.back();
    }
    window.render_frame(cx);
    assert_eq!(window.find("page").label(), Some("settings-profile"));

    // A query string travels with the location without affecting matching.
    window.click("link-query", cx);
    window.render_frame(cx);
    assert_eq!(window.find("page").label(), Some("settings-profile"));
    assert_eq!(window.find("query").label(), Some("?tab=billing|billing"));

    // A relative target resolves against the route that renders the link:
    // this link lives in `/about`, so "team" means `/about/team`.
    window.click("nav-about", cx);
    window.render_frame(cx);
    window.click("link-relative-team", cx);
    window.render_frame(cx);
    assert_eq!(window.find("page").label(), Some("team"));

    // A link can carry location state, like React Router's `state` prop.
    window.click("link-state", cx);
    window.render_frame(cx);
    assert_eq!(window.find("page").label(), Some("about"));
    assert_eq!(window.find("state").label(), Some("/settings/profile"));
  })
  .unwrap();
}

#[gpui_kit::test]
fn pathless_groups_render_the_matched_child(cx: &mut TestAppContext) {
  cx.update(gpui_kit::init);
  cx.update(gpui_router::init);

  let handle = cx.open_window(size(px(640.), px(480.)), |window, cx: &mut Context<Root>| {
    let view = cx.new(|_| GroupingApp);
    root(view, window, cx)
  });

  cx.update_window(handle.into(), |_, window, cx| {
    window.render_frame(cx);
    assert!(window.try_find("page").is_none(), "nothing renders before a match");

    for (path, expected) in [
      ("/settings/profile", "settings-profile"),
      ("/settings/billing", "settings-billing"),
      ("/settings/users/42", "user-42"),
      ("/settings/account/security", "account-security"),
    ] {
      {
        let mut navigate = gpui_router::use_navigate(cx);
        navigate.push(path);
      }
      window.render_frame(cx);
      assert_eq!(window.find("page").label(), Some(expected));
    }
  })
  .unwrap();
}

#[gpui_kit::test]
fn redirects_navigate_while_rendering(cx: &mut TestAppContext) {
  cx.update(gpui_kit::init);
  cx.update(gpui_router::init);

  let handle = cx.open_window(size(px(640.), px(480.)), |window, cx: &mut Context<Root>| {
    let view = cx.new(|_| RedirectApp);
    root(view, window, cx)
  });

  cx.update_window(handle.into(), |_, window, cx| {
    window.render_frame(cx);
    window.render_frame(cx);
    assert_eq!(window.find("page").label(), Some("redirected-about"));

    let state = gpui_router::RouterState::global(cx);
    assert_eq!(state.location.pathname, "/about");
    assert_eq!(
      state.history.len(),
      1,
      "replace(true) replaced the initial entry instead of adding one"
    );

    window.render_frame(cx);
    assert_eq!(
      gpui_router::RouterState::global(cx).history.len(),
      1,
      "a redirect whose target is current does not navigate again"
    );
  })
  .unwrap();
}

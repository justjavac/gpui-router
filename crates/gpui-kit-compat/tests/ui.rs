use gpui_kit::component::Root;
use gpui_kit::test::TestWindowExt;
use gpui_kit::{Context, TestAppContext, prelude::*, px, size};
use gpui_kit_compat::{DemoApp, OutletApp, root};

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
      navigate("/nothing-here".into());
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

    window.click("nav-about", cx);
    window.render_frame(cx);
    assert_eq!(window.find("page").label(), Some("about"));

    // A layout route nested inside the element route.
    window.click("nav-settings", cx);
    window.render_frame(cx);
    assert_eq!(window.find("page").label(), Some("settings-profile"));
  })
  .unwrap();
}

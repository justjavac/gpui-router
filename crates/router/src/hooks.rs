use crate::RouterState;
use gpui::{App, SharedString};

/// Returns a function that lets you navigate programmatically in response to user interactions or effects.
pub fn use_navigate(cx: &mut App) -> impl FnMut(SharedString) + '_ {
  move |path: SharedString| {
    cx.global_mut::<RouterState>().pathname = path;
  }
}

#[cfg(test)]
pub mod tests {
  use super::use_navigate;
  use crate::RouterState;
  use gpui::TestAppContext;

  #[gpui::test]
  async fn test_use_navigate(cx: &mut TestAppContext) {
    cx.update(|cx| {
      crate::init(cx);
      assert_eq!(cx.global::<RouterState>().pathname, "/");

      {
        let mut navigate = use_navigate(cx);
        navigate("/about".into());
      }
      assert_eq!(cx.global::<RouterState>().pathname, "/about");

      {
        let mut navigate = use_navigate(cx);
        navigate("/dashboard".into());
      }
      assert_eq!(cx.global::<RouterState>().pathname, "/dashboard");

      {
        let mut navigate = use_navigate(cx);
        navigate("/".into());
      }
      assert_eq!(cx.global::<RouterState>().pathname, "/");

      {
        let mut navigate = use_navigate(cx);
        navigate("/nothing-here".into());
      }
      assert_eq!(cx.global::<RouterState>().pathname, "/nothing-here");
    });
  }
}

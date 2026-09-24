//! A router for GPUI applications, providing declarative routing capabilities.

#[cfg(all(feature = "gpui", feature = "gpui-pre"))]
compile_error!(
  "`gpui-router` supports one GPUI backend at a time: enable `gpui` (default) or `gpui-pre`. \
   For gpui-kit, use `gpui-router = { default-features = false, features = [\"gpui-pre\"] }`"
);

#[cfg(not(any(feature = "gpui", feature = "gpui-pre")))]
compile_error!("`gpui-router` needs a GPUI backend: enable `gpui` (default), or `gpui-pre` for gpui-kit applications");

// `gpui-pre` is aliased to `gpui` so the rest of the crate compiles unchanged
// against either backend; the crates.io dependency is already called `gpui`.
#[cfg(feature = "gpui-pre")]
extern crate gpui_pre as gpui;

mod hooks;
mod layout;
mod matcher;
mod nav_link;
mod outlet;
mod redirect;
mod route;
mod router;
// The GPUI test harness needs the backend's `test-support`: the 0.2 dev-dependency
// enables it by default, other backends enable it through the `test-support` feature.
#[cfg(all(test, any(feature = "gpui", feature = "test-support")))]
mod router_tests;
mod routes;
mod search;
mod state;

pub use gpui_router_macros::*;
pub use hooks::*;
pub use layout::*;
pub use nav_link::*;
pub use outlet::*;
pub use redirect::*;
pub use route::*;
pub use router::*;
pub use routes::*;
pub use search::*;
pub use state::*;
pub(crate) use state::{normalize_pathname, normalize_shared_pathname};

/// Implementation details referenced by `gpui-router-macros` output.
///
/// The active GPUI backend is re-exported here so derive macros do not require
/// applications to depend on a crate literally named `gpui`.
#[doc(hidden)]
pub mod __private {
  #[cfg(all(feature = "gpui", not(feature = "gpui-pre")))]
  pub use ::gpui;

  #[cfg(feature = "gpui-pre")]
  pub use ::gpui_pre as gpui;
}

/// Initializes the router system within a GPUI application context.
pub fn init(cx: &mut gpui::App) {
  RouterState::init(cx);
}

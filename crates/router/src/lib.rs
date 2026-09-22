//! A router for GPUI applications, providing declarative routing capabilities.

#[cfg(all(feature = "gpui-0-2", feature = "gpui-pre"))]
compile_error!(
  "`gpui-router` supports one GPUI backend at a time: enable `gpui-0-2` (default) or `gpui-pre`. \
   For gpui-kit, use `gpui-router = { default-features = false, features = [\"gpui-pre\"] }`"
);

#[cfg(not(any(feature = "gpui-0-2", feature = "gpui-pre")))]
compile_error!(
  "`gpui-router` needs a GPUI backend: enable `gpui-0-2` (default), or `gpui-pre` for gpui-kit applications"
);

// The active backend is aliased to `gpui` so the rest of the crate compiles
// unchanged against either the crates.io `gpui` crate or `gpui-pre`, which
// gpui-kit re-exports as `gpui`.
#[cfg(all(feature = "gpui-0-2", not(feature = "gpui-pre")))]
pub extern crate gpui;

#[cfg(feature = "gpui-pre")]
pub extern crate gpui_pre as gpui;

mod hooks;
mod layout;
mod nav_link;
mod outlet;
mod route;
mod router;
// The GPUI test harness needs the backend's `test-support`: the 0.2 dev-dependency
// enables it by default, other backends enable it through the `test-support` feature.
#[cfg(all(test, any(feature = "gpui-0-2", feature = "test-support")))]
mod router_tests;
mod routes;
mod state;

pub use gpui_router_macros::*;
pub use hooks::*;
pub use layout::*;
pub use nav_link::*;
pub use outlet::*;
pub use route::*;
pub use router::*;
pub use routes::*;
pub(crate) use state::normalize_pathname;
pub use state::*;

/// Implementation details referenced by `gpui-router-macros` output.
///
/// The active GPUI backend is re-exported here so derive macros do not require
/// applications to depend on a crate literally named `gpui`.
#[doc(hidden)]
pub mod __private {
  pub use crate::gpui;
}

/// Initializes the router system within a GPUI application context.
pub fn init(cx: &mut gpui::App) {
  RouterState::init(cx);
}

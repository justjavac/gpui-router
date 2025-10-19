//! A router for GPUI applications, providing declarative routing capabilities.

mod hooks;
mod layout;
mod nav_link;
mod outlet;
mod route;
mod router;
#[cfg(test)]
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
pub use state::*;

/// Initializes the router system within a GPUI application context.
pub fn init(cx: &mut gpui::App) {
  RouterState::init(cx);
}

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

pub use hooks::*;
pub use layout::*;
pub use nav_link::*;
pub use outlet::*;
pub use route::*;
pub use router::*;
pub use routes::*;
pub use state::*;
pub use gpui_router_macros::*;

pub fn init(cx: &mut gpui::App) {
  RouterState::init(cx);
}

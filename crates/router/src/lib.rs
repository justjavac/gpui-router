mod hooks;
mod nav_link;
mod outlet;
mod route;
mod router;
#[cfg(test)]
mod router_tests;
mod routes;
mod state;

pub use hooks::*;
pub use nav_link::*;
pub use outlet::*;
pub use route::*;
pub use router::*;
pub use routes::*;
pub use state::*;

pub fn init(cx: &mut gpui::App) {
  RouterState::init(cx);
}

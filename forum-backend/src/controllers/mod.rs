mod earnings;
mod posts;
mod register;

pub use earnings::EarningsController;
pub use posts::PostsController;
pub use register::RegisterController;

use axum::Router;
use crate::AppState;

/// Trait for controllers that provide routes
pub trait WebController {
    fn routes(state: AppState) -> Router<AppState>;
}

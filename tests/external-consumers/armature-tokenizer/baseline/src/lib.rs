// Compile the exact same downstream source against the published baseline.
#[path = "../../src/lib.rs"]
mod old_api;

pub use old_api::exercise_old_api;

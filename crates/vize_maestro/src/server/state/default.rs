//! The existing default constructor, separate from the growing state facade.
use super::ServerState;

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

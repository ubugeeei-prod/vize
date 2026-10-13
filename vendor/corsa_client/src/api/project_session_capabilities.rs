use std::sync::Arc;

use super::{CapabilitiesResponse, ProjectSession};
use crate::Result;

impl ProjectSession {
    /// Returns the capabilities of the underlying runtime.
    pub async fn describe_capabilities(&self) -> Result<Arc<CapabilitiesResponse>> {
        self.client().describe_capabilities().await
    }
}

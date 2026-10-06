//! Shared declaration options for native editor projections.

use vize_canon::virtual_ts::VirtualTsOptions;

use super::ServerState;

impl ServerState {
    /// Keep editor navigation and diagnostics in the same projection namespace.
    pub(crate) async fn editor_virtual_ts_options(&self) -> VirtualTsOptions {
        let mut options = self.virtual_ts_options();
        options.reference_paths = self
            .global_component_reference_paths()
            .await
            .iter()
            .map(|path| path.to_string_lossy().as_ref().into())
            .collect();
        options
    }
}

#[cfg(test)]
mod custody;
#[cfg(test)]
mod tests;

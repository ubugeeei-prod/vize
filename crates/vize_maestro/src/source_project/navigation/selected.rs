//! Explicit selected-template requests keep their own coarse original family.

use super::{NativeNavigationProject, NavigationRefusal};
use tower_lsp::lsp_types::{Location, Position, Url};

impl NativeNavigationProject<'_> {
    /// Every authored declaration site is returned; implicit event has none.
    pub async fn template_definition(
        &self,
        uri: &Url,
        position: Position,
    ) -> Result<Vec<Location>, NavigationRefusal> {
        let (query, _) = self
            .source
            .begin_query(uri)
            .map_err(NavigationRefusal::Host)?;
        let ready = query
            .run(|snapshot| async move {
                let worker = self.worker_for(snapshot, true)?;
                Ok((worker.profile(), worker.template_definition(position).await))
            })
            .await
            .map_err(NavigationRefusal::Host)?;
        let result = ready
            .publish(|response| self.checked_response(response))
            .map_err(NavigationRefusal::Host)?;
        if matches!(result, Err(NavigationRefusal::ConfigurationChanged)) {
            self.retire_changed_configuration(uri);
        }
        result
    }

    pub async fn template_references(
        &self,
        uri: &Url,
        position: Position,
        include_declaration: bool,
    ) -> Result<Vec<Location>, NavigationRefusal> {
        let (query, _) = self
            .source
            .begin_query(uri)
            .map_err(NavigationRefusal::Host)?;
        let ready = query
            .run(|snapshot| async move {
                let worker = self.worker_for(snapshot, true)?;
                Ok((
                    worker.profile(),
                    worker.references(position, include_declaration).await,
                ))
            })
            .await
            .map_err(NavigationRefusal::Host)?;
        let result = ready
            .publish(|response| self.checked_response(response))
            .map_err(NavigationRefusal::Host)?;
        if matches!(result, Err(NavigationRefusal::ConfigurationChanged)) {
            self.retire_changed_configuration(uri);
        }
        result
    }
}

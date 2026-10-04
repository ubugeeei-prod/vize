//! Standard linked-editing requests use a separate original lexical owner.
use super::{NativeNavigationProject, NavigationRefusal, profile::QueryFamily};
use tower_lsp::lsp_types::{LinkedEditingRanges, Position, Url};

impl NativeNavigationProject<'_> {
    pub(crate) fn retire_linked_editing(&self) {
        let retired = core::mem::take(&mut *self.names_workers.lock());
        for cached in retired.into_values() {
            if let Ok(worker) = cached.result {
                worker.retire();
            }
        }
    }

    pub async fn linked_editing(
        &self,
        uri: &Url,
        position: Position,
    ) -> Result<Option<LinkedEditingRanges>, NavigationRefusal> {
        let (query, _) = self
            .source
            .begin_query(uri)
            .map_err(NavigationRefusal::Host)?;
        let ready = query
            .run(|snapshot| async move {
                let worker = self.worker_family(snapshot, QueryFamily::Names)?;
                Ok((worker.profile(), worker.linked_editing(position).await))
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

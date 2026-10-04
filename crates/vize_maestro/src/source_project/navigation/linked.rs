//! Standard transport tickets and independent parser-only provider queries.
use super::{NativeNavigationProject, NavigationRefusal, names::NamesConfiguration};
use crate::server::NativeLinkedNamesTicket;
use tower_lsp::lsp_types::{LinkedEditingRanges, Position, Url};

impl NativeNavigationProject<'_> {
    pub async fn linked_editing(
        &self,
        uri: &Url,
        position: Position,
    ) -> Result<Option<LinkedEditingRanges>, NavigationRefusal> {
        let ticket = self.source.capture_names_parser()?;
        self.linked_editing_with_configuration(uri, position, ticket)
            .await
    }

    pub(crate) async fn linked_editing_configured(
        &self,
        uri: &Url,
        position: Position,
        ticket: NativeLinkedNamesTicket,
    ) -> Result<Option<LinkedEditingRanges>, NavigationRefusal> {
        self.linked_editing_with_configuration(uri, position, NamesConfiguration::Linked(ticket))
            .await
    }

    async fn linked_editing_with_configuration(
        &self,
        uri: &Url,
        position: Position,
        ticket: NamesConfiguration,
    ) -> Result<Option<LinkedEditingRanges>, NavigationRefusal> {
        let (query, _) = self
            .source
            .begin_query(uri)
            .map_err(NavigationRefusal::Host)?;
        let borrowed = &ticket;
        let ready = query
            .run(|snapshot| async move {
                let worker = self.names_worker(snapshot, borrowed)?;
                worker.linked_editing(position).await
            })
            .await
            .map_err(NavigationRefusal::Host)?;
        let result = ready
            .publish(|response| {
                // Real document→configuration publication. Never a generic
                // getter, worker map, store reentry, retirement or await.
                self.source
                    .with_names_configuration(&ticket, |_| response)
                    .and_then(|response| response)
            })
            .map_err(NavigationRefusal::Host)?;
        if matches!(result, Err(NavigationRefusal::ConfigurationChanged)) {
            // Current actual profiles decide retirement after guards drop.
            // Route-only staleness never drains a healthy provider owner.
            self.retire_changed_configuration(uri);
        }
        result
    }
}

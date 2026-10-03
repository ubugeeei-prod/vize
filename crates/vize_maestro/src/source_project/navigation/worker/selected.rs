//! Only owned selected query requests and responses cross the worker channel.

use super::{Command, NavigationRefusal, NavigationWorker, oneshot};
use tower_lsp::lsp_types::{Location, Position};

impl NavigationWorker {
    pub(in crate::source_project::navigation) async fn template_definition(
        &self,
        position: Position,
    ) -> Result<Vec<Location>, NavigationRefusal> {
        let (reply, response) = oneshot::channel();
        self.enqueue(Command::TemplateDefinition(position, reply))?;
        response
            .await
            .map_err(|_| NavigationRefusal::WorkerUnavailable)?
    }

    #[cfg(test)]
    pub(in crate::source_project::navigation) async fn selected_drain_barrier(
        &self,
        position: Position,
    ) -> Result<Inspection, NavigationRefusal> {
        let (reply, response) = oneshot::channel();
        // Tests may block only after resuming the actually parked receiver.
        self.commands
            .send(Command::SelectedInspect(position, reply))
            .unwrap();
        response
            .await
            .map_err(|_| NavigationRefusal::WorkerUnavailable)?
    }

    #[cfg(test)]
    pub(in crate::source_project::navigation) async fn selected_inspect(
        &self,
        position: Position,
    ) -> Result<Inspection, NavigationRefusal> {
        let (reply, response) = oneshot::channel();
        self.enqueue(Command::SelectedInspect(position, reply))?;
        response
            .await
            .map_err(|_| NavigationRefusal::WorkerUnavailable)?
    }
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::source_project::navigation) struct Inspection {
    pub(in crate::source_project::navigation) file: usize,
    pub(in crate::source_project::navigation) observation: usize,
    pub(in crate::source_project::navigation) descriptor: usize,
    pub(in crate::source_project::navigation) selected: usize,
    pub(in crate::source_project::navigation) handler_body: Option<usize>,
    pub(in crate::source_project::navigation) productions: usize,
}

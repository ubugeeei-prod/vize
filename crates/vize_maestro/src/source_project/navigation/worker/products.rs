use super::{Command, NavigationRefusal, NavigationWorker, oneshot};
use tower_lsp::lsp_types::{DocumentHighlight, Position};

pub(in crate::source_project::navigation) struct Request {
    pub(in crate::source_project::navigation) position: Position,
    pub(in crate::source_project::navigation) reply:
        oneshot::Sender<Result<Vec<DocumentHighlight>, NavigationRefusal>>,
}

impl NavigationWorker {
    pub(in crate::source_project::navigation) async fn symbol_sites(
        &self,
        position: Position,
    ) -> Result<Vec<DocumentHighlight>, NavigationRefusal> {
        let (reply, response) = oneshot::channel();
        self.enqueue(Command::Products(Request { position, reply }))?;
        response
            .await
            .map_err(|_| NavigationRefusal::WorkerUnavailable)?
    }
}

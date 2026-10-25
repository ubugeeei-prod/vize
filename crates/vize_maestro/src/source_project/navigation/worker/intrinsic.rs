//! Owned requests borrow the same bounded mailbox as other Program queries.

use super::{Command, NavigationRefusal, NavigationWorker, oneshot};
use crate::source_project::navigation::IntrinsicTagTarget;
use tower_lsp::lsp_types::Position;

pub(in crate::source_project::navigation) struct Request {
    pub(in crate::source_project::navigation) position: Position,
    pub(in crate::source_project::navigation) reply:
        oneshot::Sender<Result<Option<IntrinsicTagTarget>, NavigationRefusal>>,
}

impl Request {
    pub(in crate::source_project::navigation) fn refuse(self, refusal: NavigationRefusal) {
        if !self.reply.is_canceled() {
            let _ = self.reply.send(Err(refusal));
        }
    }
}

impl NavigationWorker {
    #[cfg(test)]
    pub(in crate::source_project::navigation) fn program_walks(&self) -> usize {
        self.control
            .walks
            .load(std::sync::atomic::Ordering::Acquire)
    }

    #[cfg(test)]
    pub(in crate::source_project::navigation) fn intrinsic_entries(&self) -> usize {
        self.control
            .intrinsic_entries
            .load(std::sync::atomic::Ordering::Acquire)
    }

    pub(in crate::source_project::navigation) async fn intrinsic_tag(
        &self,
        position: Position,
    ) -> Result<Option<IntrinsicTagTarget>, NavigationRefusal> {
        let (reply, response) = oneshot::channel();
        self.enqueue(Command::IntrinsicTag(Request { position, reply }))?;
        response
            .await
            .map_err(|_| NavigationRefusal::WorkerUnavailable)?
    }
}

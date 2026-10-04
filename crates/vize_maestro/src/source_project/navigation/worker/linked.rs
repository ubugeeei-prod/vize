//! Owned lexical requests share the actual bounded mailbox and live limit.
use super::{Command, NavigationRefusal, NavigationWorker, oneshot};
use tower_lsp::lsp_types::{LinkedEditingRanges, Position};

pub(in crate::source_project::navigation) enum Request {
    Ranges(
        Position,
        oneshot::Sender<Result<Option<LinkedEditingRanges>, NavigationRefusal>>,
    ),
    #[cfg(test)]
    Inspect(oneshot::Sender<Result<Inspection, NavigationRefusal>>),
}
impl Request {
    pub(in crate::source_project::navigation) fn refuse(self, refusal: NavigationRefusal) {
        match self {
            Self::Ranges(_, reply) => {
                if !reply.is_canceled() {
                    let _ = reply.send(Err(refusal));
                }
            }
            #[cfg(test)]
            Self::Inspect(reply) => {
                let _ = reply.send(Err(refusal));
            }
        }
    }
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::source_project::navigation) struct Inspection {
    pub(in crate::source_project::navigation) descriptor: usize,
    pub(in crate::source_project::navigation) selected: usize,
    pub(in crate::source_project::navigation) component: usize,
    pub(in crate::source_project::navigation) productions: usize,
}

impl NavigationWorker {
    pub(in crate::source_project::navigation) async fn linked_editing(
        &self,
        position: Position,
    ) -> Result<Option<LinkedEditingRanges>, NavigationRefusal> {
        let (reply, response) = oneshot::channel();
        self.enqueue(Command::LinkedEditing(Request::Ranges(position, reply)))?;
        response
            .await
            .map_err(|_| NavigationRefusal::WorkerUnavailable)?
    }
    #[cfg(test)]
    pub(in crate::source_project::navigation) async fn names_drain_barrier(
        &self,
    ) -> Result<Inspection, NavigationRefusal> {
        let (reply, response) = oneshot::channel();
        // Test-only barrier after releasing the actual parked receiver.
        self.commands
            .send(Command::LinkedEditing(Request::Inspect(reply)))
            .unwrap();
        response
            .await
            .map_err(|_| NavigationRefusal::WorkerUnavailable)?
    }

    #[cfg(test)]
    pub(in crate::source_project::navigation) async fn names_inspect(
        &self,
    ) -> Result<Inspection, NavigationRefusal> {
        let (reply, response) = oneshot::channel();
        self.enqueue(Command::LinkedEditing(Request::Inspect(reply)))?;
        response
            .await
            .map_err(|_| NavigationRefusal::WorkerUnavailable)?
    }
}

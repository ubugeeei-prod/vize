//! Owned module-source commands share the original bounded worker mailbox.

use super::super::retained::module_links::{ModuleOperandRefusal, ModuleOperands};
use super::{Command, NavigationRefusal, NavigationWorker, oneshot};

pub(in crate::source_project::navigation) struct Request {
    pub(in crate::source_project::navigation) reply:
        oneshot::Sender<Result<ModuleOperands, ModuleOperandRefusal>>,
}

impl Request {
    pub(in crate::source_project::navigation) fn refuse(self, refusal: NavigationRefusal) {
        if !self.reply.is_canceled() {
            let _ = self
                .reply
                .send(Err(ModuleOperandRefusal::Navigation(refusal)));
        }
    }
}

impl NavigationWorker {
    pub(in crate::source_project::navigation) async fn module_operands(
        &self,
    ) -> Result<ModuleOperands, ModuleOperandRefusal> {
        let (reply, response) = oneshot::channel();
        self.enqueue(Command::ModuleLinks(Request { reply }))
            .map_err(ModuleOperandRefusal::Navigation)?;
        response
            .await
            .map_err(|_| ModuleOperandRefusal::Navigation(NavigationRefusal::WorkerUnavailable))?
    }
}

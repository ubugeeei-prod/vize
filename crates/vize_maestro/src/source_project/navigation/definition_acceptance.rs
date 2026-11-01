//! One per-project test gate after real native computation, before publication.

use super::{Location, NativeNavigationProject, NavigationRefusal};
use futures::channel::oneshot;

pub(crate) type ReadyDefinition = Result<Option<Location>, NavigationRefusal>;
pub(crate) type PublicationPause = (oneshot::Receiver<ReadyDefinition>, oneshot::Sender<()>);

pub(super) struct PublicationGate {
    entered: oneshot::Sender<ReadyDefinition>,
    resume: oneshot::Receiver<()>,
}

impl NativeNavigationProject<'_> {
    pub(crate) fn pause_next_definition_publication_for_test(
        &self,
    ) -> Result<PublicationPause, &'static str> {
        let mut gate = self.definition_publication.lock();
        if gate.is_some() {
            return Err("definition publication is already held");
        }
        let (entered, observed) = oneshot::channel();
        let (release, resume) = oneshot::channel();
        *gate = Some(PublicationGate { entered, resume });
        Ok((observed, release))
    }

    pub(super) async fn wait_definition_publication_for_test(&self, actual: ReadyDefinition) {
        // Taking the single-use gate releases the lock before either await.
        // Other requests and the actual didChange route remain live.
        let gate = { self.definition_publication.lock().take() };
        if let Some(gate) = gate {
            let _ = gate.entered.send(actual);
            let _ = gate.resume.await;
        }
        // Original snapshot/config/cancellation checks still decide publication.
    }
}

//! Capture the exact editor process before shutdown consumes its owner.
use super::EditorLspSession;
use crate::lsp_client::retirement::{CloseHandle, CloseOwners};
use corsa::process::ProcessOwnership;
use serde_json::{Value, json};

impl EditorLspSession {
    fn retirement_binding(&self, api_role: &str) -> Value {
        let owner = match self.client.process_ownership() {
            ProcessOwnership::Owned(observer) => Some(observer.snapshot()),
            ProcessOwnership::Attached => None,
        };
        json!({"apiRole":api_role,"ownership":"attached","ownerExpectedPid":owner.as_ref().map(|owner| owner.expected_pid),"ownerSpawnSequence":owner.as_ref().map(|owner| owner.spawn_sequence),"attachment":self.configured_api.as_ref().map(|api| &api.session)})
    }

    pub(in crate::lsp_client) fn retirement_attachment(&self, role: &str) -> CloseOwners {
        let mut owners = CloseOwners::default();
        if self.configured_api.is_some()
            && matches!(self.client.process_ownership(), ProcessOwnership::Owned(_))
        {
            owners.attached_bindings.push(self.retirement_binding(role));
        } else {
            owners.missing_binding = true;
        }
        owners
    }

    pub(in crate::lsp_client) fn retirement_owners(&self, role: &str) -> CloseOwners {
        let mut owners = CloseOwners::default();
        owners
            .handles
            .push(CloseHandle::Editor(self.client.clone()));
        if let Some(api) = &self.configured_api {
            match api.client.process_ownership() {
                ProcessOwnership::Attached => owners.append(self.retirement_attachment(role)),
                ProcessOwnership::Owned(_) => {
                    owners.handles.push(CloseHandle::Api(api.client.clone()))
                }
            }
        }
        owners
    }
}

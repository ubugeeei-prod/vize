//! Typed handles are captured before session cleanup consumes their fields.
use super::CorsaProjectClient;
use corsa::{
    api::ApiClient,
    process::{ProcessOwnership, ProcessTerminalObserver},
    runtime::block_on,
};
use corsa_lsp::LspClient;
use serde_json::{Value, json};
use vize_l0::{String, cstr};

#[derive(Clone)]
pub(crate) enum CloseHandle {
    Api(ApiClient),
    Editor(LspClient),
}

impl CloseHandle {
    pub(crate) fn observer(&self) -> Option<ProcessTerminalObserver> {
        match match self {
            Self::Api(client) => client.process_ownership(),
            Self::Editor(client) => client.process_ownership(),
        } {
            ProcessOwnership::Owned(observer) => Some(observer),
            ProcessOwnership::Attached => None,
        }
    }

    pub(crate) fn acknowledged(&self) -> bool {
        self.observer()
            .is_some_and(|observer| observer.snapshot().custody_complete())
    }

    pub(crate) fn retry(&self) -> Result<(), String> {
        let result = match self {
            Self::Api(client) => block_on(client.abort_startup()),
            Self::Editor(client) => client.abort_startup(),
        };
        result
            .map(|_| ())
            .map_err(|error| cstr!("Exact owned retirement cleanup retry: {error}"))
    }
}

#[derive(Default)]
pub(crate) struct CloseOwners {
    pub(crate) handles: Vec<CloseHandle>,
    pub(crate) attached_bindings: Vec<Value>,
    pub(crate) missing_binding: bool,
}

impl CloseOwners {
    pub(crate) fn append(&mut self, other: Self) {
        for handle in other.handles {
            if let Some(observer) = handle.observer() {
                if !self
                    .handles
                    .iter()
                    .filter_map(CloseHandle::observer)
                    .any(|existing| existing.same_owner(&observer))
                {
                    self.handles.push(handle);
                }
            } else {
                self.missing_binding = true;
            }
        }
        self.attached_bindings.extend(other.attached_bindings);
        self.missing_binding |= other.missing_binding;
    }
}

impl CorsaProjectClient {
    pub(crate) fn retirement_owners(&self) -> CloseOwners {
        let mut owners = CloseOwners::default();
        if let Some(session) = &self.session {
            match session.client().process_ownership() {
                ProcessOwnership::Owned(_) => owners
                    .handles
                    .push(CloseHandle::Api(session.client().clone())),
                ProcessOwnership::Attached => {
                    if self.project_session_attached_to_editor
                        && let Some(editor) = &self.editor_lsp
                    {
                        owners.append(editor.retirement_attachment("project-api"));
                    } else {
                        owners.missing_binding = true;
                    }
                }
            }
        }
        if let Some(editor) = &self.editor_lsp {
            owners.append(editor.retirement_owners("editor"));
        }
        if let Some(authored) = &self.original_diagnosing_session {
            owners.append(authored.retirement_owners());
        }
        owners
    }
}

pub(crate) fn ownership_error(owner: &ProcessOwnership, error: &str) {
    if let ProcessOwnership::Owned(observer) = owner {
        observer.record_error(error);
    }
}

pub(crate) fn observation_json(observed: &corsa::process::ProcessCloseObservation) -> Value {
    let terminal = observed.terminal.as_ref().map(|terminal| {
        #[cfg(unix)]
        let signal = { use std::os::unix::process::ExitStatusExt; terminal.status.signal() };
        #[cfg(not(unix))]
        let signal: Option<i32> = None;
        json!({"pid":terminal.pid,"status":cstr!("{:?}",terminal.status),"exitCode":terminal.status.code(),"signal":signal,"killRequested":terminal.kill_requested,"timedOut":terminal.timed_out,"reaped":terminal.reaped})
    });
    let stderr = match &observed.stderr_receipts {
        None => json!({"mode":"inherited-server-stderr"}),
        Some(receipts) => {
            json!({"mode":"observed-editor","receipts":receipts.iter().map(|receipt| json!({"pid":receipt.pid,"completed":receipt.completed,"bytes":receipt.stderr,"stderrComplete":receipt.stderr_complete,"stderrError":receipt.stderr_error,"processError":receipt.process_error,"status":receipt.exit_status.map(|status| cstr!("{status:?}")),"timedOut":receipt.timed_out,"killRequested":receipt.kill_requested,"reaped":receipt.reaped})).collect::<Vec<_>>()})
        }
    };
    json!({"spawnSequence":observed.spawn_sequence,"expectedPid":observed.expected_pid,"role":observed.context,"terminal":terminal,"raw_errors":observed.raw_errors,"close_error":observed.close_error,"stderrCustody":stderr})
}

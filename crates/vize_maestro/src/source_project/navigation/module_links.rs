//! Original worker operands and the same ready query own every point link.
use super::retained::module_links::ModuleOperandRefusal;
use super::{NativeNavigationProject, NavigationRefusal};
use crate::{
    server::{ModuleLinkContextError, ModuleTargetError, TargetPolicy},
    source_project::ModuleLinkPublicationError,
};
use tower_lsp::{
    jsonrpc::{Error, ErrorCode},
    lsp_types::{DocumentLink, Url},
};

#[derive(Debug)]
pub(crate) struct ModuleDocumentLinkError(Refusal);

#[derive(Debug)]
enum Refusal {
    Navigation(NavigationRefusal),
    Operands(ModuleOperandRefusal),
    Publication(ModuleLinkPublicationError),
    Targets(ModuleTargetError),
}

impl NativeNavigationProject<'_> {
    #[cfg(test)]
    pub(crate) fn pause_module_link_worker(&self, uri: &Url) -> std::sync::mpsc::Sender<()> {
        let (query, _) = self.source.begin_query(uri).unwrap();
        self.worker(std::sync::Arc::clone(query.snapshot()))
            .unwrap()
            .pause()
    }

    /// Explicit original JS/TS Module links, never standard-route selection.
    pub(crate) async fn module_document_links(
        &self,
        uri: &Url,
    ) -> Result<Vec<DocumentLink>, ModuleDocumentLinkError> {
        let (query, _) = self.source.begin_query(uri).map_err(|error| {
            ModuleDocumentLinkError(Refusal::Navigation(NavigationRefusal::Host(error)))
        })?;
        let ready = query
            .run(|snapshot| async move {
                let worker = self
                    .worker(snapshot)
                    .map_err(ModuleOperandRefusal::Navigation)?;
                worker.module_operands().await
            })
            .await
            .map_err(|error| {
                ModuleDocumentLinkError(Refusal::Navigation(NavigationRefusal::Host(error)))
            })?;
        // Even unavailable context is qualified by this original source/cancel.
        let context = ready
            .capture_current_module_link_context()
            .map_err(|error| ModuleDocumentLinkError(Refusal::Publication(error)))?;
        let observed = match ready.prepared() {
            Ok(operands) if !operands.is_empty() => {
                let requests: Vec<&str> = operands.decoded_requests().collect();
                ready
                    .observe_module_targets(&context, &requests)
                    .map_err(|error| ModuleDocumentLinkError(Refusal::Targets(error)))?
            }
            _ => {
                // No physical EmptyBatch promotion and no unguarded error exit.
                return ready
                    .publish_with_module_link_context(&context, |response| {
                        response
                            .and_then(|operands| operands.into_links(&[]))
                            .map_err(|error| ModuleDocumentLinkError(Refusal::Operands(error)))
                    })
                    .map_err(|error| ModuleDocumentLinkError(Refusal::Publication(error)))?;
            }
        };
        let checked = observed
            .recheck()
            .map_err(|error| ModuleDocumentLinkError(Refusal::Targets(error)))?;
        ready
            .publish_with_module_targets(checked, |response, targets| {
                response
                    .and_then(|operands| operands.into_links(targets))
                    .map_err(|error| ModuleDocumentLinkError(Refusal::Operands(error)))
            })
            .map_err(|error| ModuleDocumentLinkError(Refusal::Targets(error)))?
    }
}

impl ModuleDocumentLinkError {
    /// Preserve the original navigation mapper; new failures have whole codes.
    pub(crate) fn into_rpc_error(
        self,
        navigation: impl FnOnce(NavigationRefusal) -> Error,
    ) -> Error {
        match self.0 {
            Refusal::Navigation(error)
            | Refusal::Operands(ModuleOperandRefusal::Navigation(error)) => navigation(error),
            Refusal::Operands(ModuleOperandRefusal::Input(_)) => rpc(
                ErrorCode::ServerError(-32013),
                "Native module input refused",
            ),
            Refusal::Operands(ModuleOperandRefusal::Sources(_)) => rpc(
                ErrorCode::ServerError(-32014),
                "Native original module sources refused",
            ),
            Refusal::Operands(ModuleOperandRefusal::TargetCardinality { .. }) => rpc(
                ErrorCode::ServerError(-32019),
                "Native module target cardinality refused",
            ),
            Refusal::Publication(ModuleLinkPublicationError::Source(error))
            | Refusal::Targets(ModuleTargetError::Source(error)) => {
                navigation(NavigationRefusal::Host(error))
            }
            Refusal::Publication(ModuleLinkPublicationError::Context(error))
            | Refusal::Targets(ModuleTargetError::Context(error)) => context_error(error),
            Refusal::Targets(
                ModuleTargetError::EventSuperseded
                | ModuleTargetError::ForeignSnapshot
                | ModuleTargetError::Changed(_),
            ) => rpc(
                ErrorCode::ContentModified,
                "Native module target superseded",
            ),
            Refusal::Targets(ModuleTargetError::Policy(TargetPolicy::UnsupportedPlatform)) => rpc(
                ErrorCode::ServerError(-32016),
                "Native module target platform unsupported",
            ),
            Refusal::Targets(ModuleTargetError::Policy(_)) => rpc(
                ErrorCode::ServerError(-32017),
                "Native module target policy refused",
            ),
            Refusal::Targets(ModuleTargetError::Io { .. }) => rpc(
                ErrorCode::ServerError(-32018),
                "Native module target unavailable",
            ),
        }
    }
}

fn context_error(error: ModuleLinkContextError) -> Error {
    match error {
        ModuleLinkContextError::Superseded
        | ModuleLinkContextError::Updating
        | ModuleLinkContextError::Retired(_) => rpc(
            ErrorCode::ContentModified,
            "Native module-link context superseded",
        ),
        ModuleLinkContextError::HostUnavailable
        | ModuleLinkContextError::NativeUnavailable
        | ModuleLinkContextError::MissingRoot
        | ModuleLinkContextError::ForeignSession => rpc(
            ErrorCode::ServerError(-32015),
            "Native module-link context unavailable",
        ),
    }
}

fn rpc(code: ErrorCode, message: &'static str) -> Error {
    let mut error = Error::new(code);
    error.message = message.into();
    error
}

#[cfg(test)]
mod tests;

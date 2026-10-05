//! Source text retained by one native snapshot, using protocol 5 from native 7.0.2.
//!
//! This reads only the SourceFile root's text and identities. It does not decode
//! the AST or structured metadata. The caller must acknowledge its overlays
//! before creating the snapshot; this provider neither spawns nor reads disk.

use crate::file_uri::path_to_file_uri;
use corsa::{
    CorsaError,
    api::{ApiClient, EncodedPayload, ManagedSnapshot, ProjectResponse, SnapshotHandle},
    runtime::block_on,
};
use serde_json::json;
use std::path::Path;
use vize_l0::{FxHashMap, String};

mod protocol;
#[cfg(test)]
mod tests;

/// A bounded refusal leaves the entire original response available for audit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceTextRefusal {
    ReleasedSnapshot,
    ProjectIdentity,
    SourceNames,
    SourceIdentity,
    AbsentSource,
    Header,
    Version,
    Sections,
    Root,
    Strings,
    Utf8,
}

/// Snapshot and API ownership are minted together; foreign handles are not inputs.
pub struct SnapshotSourceOwner<'client> {
    client: &'client ApiClient,
    snapshot: ManagedSnapshot,
    released: bool,
}

/// A project selected from this owner's actual snapshot and native source names.
pub struct SnapshotSourceProject<'owner> {
    owner: &'owner SnapshotSourceOwner<'owner>,
    descriptor: &'owner ProjectResponse,
    sources: FxHashMap<String, String>,
    source_names: Vec<String>,
}

/// Complete source text cannot outlive its project, snapshot or API owner.
///
/// ```
/// use corsa::api::ApiClient;
/// use std::path::Path;
/// use vize_canon::lsp_client::snapshot_source::{SnapshotSourceOwner, SourceTextOutcome};
/// fn scoped(client: &ApiClient) {
///     let mut owner = SnapshotSourceOwner::create(client).unwrap();
///     {
///         let project = owner.project(Path::new("/project/tsconfig.json")).unwrap().unwrap();
///         if let SourceTextOutcome::Complete(text) = project.read("file:///project/source.ts").unwrap() {
///             println!("{}", text.text());
///         }
///     }
///     owner.release().unwrap();
/// }
/// ```
///
/// ```compile_fail
/// use corsa::api::ApiClient;
/// use std::path::Path;
/// use vize_canon::lsp_client::snapshot_source::{
///     SnapshotSourceOwner, SnapshotSourceText, SourceTextOutcome,
/// };
/// fn escape(client: &ApiClient) -> SnapshotSourceText<'_> {
///     let owner = SnapshotSourceOwner::create(client).unwrap();
///     let project = owner.project(Path::new("/project/tsconfig.json")).unwrap().unwrap();
///     match project.read("file:///project/source.ts").unwrap() {
///         SourceTextOutcome::Complete(text) => text,
///         SourceTextOutcome::Refused { .. } => panic!("refused"),
///     }
/// }
/// ```
///
/// ```compile_fail
/// use corsa::api::ApiClient;
/// use std::path::Path;
/// use vize_canon::lsp_client::snapshot_source::{SnapshotSourceOwner, SourceTextOutcome};
/// fn read_after_release(client: &ApiClient) {
///     let mut owner = SnapshotSourceOwner::create(client).unwrap();
///     let project = owner.project(Path::new("/project/tsconfig.json")).unwrap().unwrap();
///     let outcome = project.read("file:///project/source.ts").unwrap();
///     owner.release().unwrap();
///     if let SourceTextOutcome::Complete(text) = outcome {
///         println!("{}", text.text());
///     }
/// }
/// ```
pub struct SnapshotSourceText<'project> {
    project: &'project SnapshotSourceProject<'project>,
    encoded: EncodedPayload,
    text: String,
    file_name: String,
    path: String,
    uri: String,
}

/// Unsupported or malformed responses never supply partially decoded text.
pub enum SourceTextOutcome<'project> {
    Complete(SnapshotSourceText<'project>),
    Refused {
        reason: SourceTextRefusal,
        encoded: Option<EncodedPayload>,
    },
}

impl<'client> SnapshotSourceOwner<'client> {
    /// Create a managed snapshot on the existing, acknowledged native attachment.
    pub fn create(client: &'client ApiClient) -> Result<Self, CorsaError> {
        Ok(Self {
            client,
            snapshot: block_on(client.update_snapshot(Default::default()))?,
            released: false,
        })
    }

    /// Read-only custody metadata; requests below use the private owned handle.
    pub fn handle(&self) -> &SnapshotHandle {
        &self.snapshot.handle
    }

    /// Admit exactly one configured project, with byte-exact config identity.
    pub fn project(
        &self,
        config: &Path,
    ) -> Result<Result<SnapshotSourceProject<'_>, SourceTextRefusal>, CorsaError> {
        if self.released {
            return Ok(Err(SourceTextRefusal::ReleasedSnapshot));
        }
        let [descriptor] = self.snapshot.projects.as_slice() else {
            return Ok(Err(SourceTextRefusal::ProjectIdentity));
        };
        if config.to_str() != Some(descriptor.config_file_name.as_str()) {
            return Ok(Err(SourceTextRefusal::ProjectIdentity));
        }
        let value = block_on(self.client.raw_json_request(
            "getSourceFileNames",
            json!({"snapshot":self.snapshot.handle,"project":descriptor.id}),
        ))?;
        let Ok(names) = serde_json::from_value::<Vec<String>>(value) else {
            return Ok(Err(SourceTextRefusal::SourceNames));
        };
        let Some(sources) = source_members(&names) else {
            return Ok(Err(SourceTextRefusal::SourceNames));
        };
        Ok(Ok(SnapshotSourceProject {
            owner: self,
            descriptor,
            sources,
            source_names: names,
        }))
    }

    /// Eagerly release while the API owner lives. Even a failed attempt retires
    /// this view; the caller retains the actual error and decides owner recovery.
    pub fn release(&mut self) -> Result<(), CorsaError> {
        self.released = true;
        block_on(self.snapshot.release())
    }
}

impl SnapshotSourceProject<'_> {
    pub fn descriptor(&self) -> &ProjectResponse {
        self.descriptor
    }

    pub fn contains_uri(&self, uri: &str) -> bool {
        self.sources.contains_key(uri)
    }

    /// Preserve the native response's original order without another query.
    pub fn source_names(&self) -> &[String] {
        &self.source_names
    }

    /// Fetch from this same snapshot/project, then verify the encoded root's
    /// exact native source name and path. URI aliases never acquire ownership.
    pub fn read(&self, uri: &str) -> Result<SourceTextOutcome<'_>, CorsaError> {
        let Some(name) = self.sources.get(uri) else {
            return Ok(SourceTextOutcome::Refused {
                reason: SourceTextRefusal::SourceIdentity,
                encoded: None,
            });
        };
        let Some(encoded) = block_on(self.owner.client.get_source_file(
            self.owner.snapshot.handle.clone(),
            self.descriptor.id.clone(),
            name.as_str(),
        ))?
        else {
            return Ok(SourceTextOutcome::Refused {
                reason: SourceTextRefusal::AbsentSource,
                encoded: None,
            });
        };
        match protocol::source_text(encoded.as_bytes(), name) {
            Ok((text, file_name, path)) => Ok(SourceTextOutcome::Complete(SnapshotSourceText {
                project: self,
                text: String::from(text),
                file_name: String::from(file_name),
                path: String::from(path),
                uri: String::from(uri),
                encoded,
            })),
            Err(reason) => Ok(SourceTextOutcome::Refused {
                reason,
                encoded: Some(encoded),
            }),
        }
    }
}

impl SnapshotSourceText<'_> {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn encoded_bytes(&self) -> &[u8] {
        self.encoded.as_bytes()
    }

    pub fn snapshot_handle(&self) -> &SnapshotHandle {
        self.project.owner.handle()
    }

    pub fn project_descriptor(&self) -> &ProjectResponse {
        self.project.descriptor()
    }
}

fn source_members(names: &[String]) -> Option<FxHashMap<String, String>> {
    let mut sources = FxHashMap::default();
    for name in names {
        if !Path::new(name.as_str()).is_absolute() || name.contains('\0') {
            return None;
        }
        let uri = path_to_file_uri(Path::new(name.as_str()));
        if sources.insert(uri, name.clone()).is_some() {
            return None;
        }
    }
    Some(sources)
}

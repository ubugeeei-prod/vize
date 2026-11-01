//! Test-carrier observation of one constructor usage on its existing owner.
//!
//! These are decoded SDK observations, not raw-wire or Vue-origin admission.
//! No emitter consumes them and every result retains `native_admission=false`.

use corsa::{
    CorsaError,
    api::{
        EncodedSourceFile, ManagedSnapshot, NodeHandle, ProjectResponse, ProjectSession,
        SignatureResponse, SnapshotHandle, SymbolResponse, TypeResponse,
    },
};
use serde::Serialize;
use serde_json::Value;
use std::{fmt, sync::Arc};
use vize_l0::String;

mod query;
#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SourceExpectation {
    pub(crate) file: String,
    pub(crate) path: String,
    pub(crate) text: String,
}

/// Offsets belong to the exact supplied source and use native UTF-16 units.
/// An anchor locates a request; its spelling never grants semantic authority.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct Anchor {
    pub(crate) start: u32,
    pub(crate) end: u32,
    pub(crate) position: u32,
    pub(crate) text: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct UsageRequest {
    pub(crate) usage_id: String,
    pub(crate) authored_file: String,
    pub(crate) authored_source: String,
    pub(crate) authored_usage: Anchor,
    /// Selected config name only; no authored config-file byte claim.
    pub(crate) config_file: String,
    pub(crate) generated: SourceExpectation,
    pub(crate) component_source: SourceExpectation,
    pub(crate) component_reference: Anchor,
    pub(crate) vue_module: Anchor,
    pub(crate) hosts: [Anchor; 2],
}

#[derive(Debug, Serialize)]
pub(crate) struct NativeSource {
    requested: String,
    encoded: Option<Vec<u8>>,
    decoded: Option<EncodedSourceFile>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "state", content = "value", rename_all = "camelCase")]
enum SignatureTypeParametersRaw {
    NotRequested,
    Requested,
    Received(Value),
}

#[derive(Debug, Serialize)]
pub(crate) struct SignatureReturn {
    signature: SignatureResponse,
    returned: Option<TypeResponse>,
    symbol: Option<SymbolResponse>,
    type_parameters_raw: SignatureTypeParametersRaw,
    type_parameters: Option<Vec<TypeResponse>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct HostObservation {
    anchor: Anchor,
    pair: Option<TypeResponse>,
    with_arguments: Option<SymbolResponse>,
    instance: Option<TypeResponse>,
    instance_symbol: Option<SymbolResponse>,
    props: Option<SymbolResponse>,
    props_type: Option<TypeResponse>,
}

#[derive(Debug, Default, Serialize)]
pub(crate) struct VueObservation {
    module: Option<SymbolResponse>,
    module_alias: Option<SymbolResponse>,
    exports: Option<Vec<SymbolResponse>>,
    public_instance: Option<SymbolResponse>,
    public_instance_alias: Option<SymbolResponse>,
    declared_instance: Option<TypeResponse>,
    props: Option<SymbolResponse>,
    props_type: Option<TypeResponse>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) enum Unavailable {
    ProjectIdentity,
    SourceMembership,
    AbsentSource(String),
    IncompleteSourceRecord(String),
    SourceIdentity(String),
    SourceText(String),
    Anchor(String),
    MissingQueryValue(String),
    ExportCardinality(usize),
    DeclarationMetadata(String),
    DeclarationRangeUnavailable(NodeHandle),
    /// This SDK reader does not expose an authenticated call-like AST node.
    SelectedSignatureRequiresNativeCallNode,
    /// Raw signature field presence is retained, but a closed nongeneric
    /// return is not admitted from typed outer/local metadata or empty arrays.
    GenericClosureNotAdmitted,
    /// Equality of declarations or instance shape alone is not admission.
    VueReturnOriginNotAdmitted,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObservationData {
    schema: &'static str,
    representation: &'static str,
    native_admission: bool,
    request: UsageRequest,
    snapshot: SnapshotHandle,
    snapshot_projects: Vec<ProjectResponse>,
    project: ProjectResponse,
    stage: &'static str,
    source_names: Option<Value>,
    pending_source: Option<String>,
    sources: Vec<NativeSource>,
    constructor_type: Option<TypeResponse>,
    construct_signatures_raw: Option<Value>,
    construct_signatures: Option<Vec<SignatureResponse>>,
    declared_returns: Vec<SignatureReturn>,
    hosts: Vec<HostObservation>,
    vue: VueObservation,
    unavailable: Vec<Unavailable>,
}

/// Handles cannot outlive the borrowed session or its retained snapshot here.
/// Explicit release/close can still fail requests; the original error is kept.
#[derive(Serialize)]
pub(crate) struct Observation<'session> {
    #[serde(skip)]
    _session: &'session ProjectSession,
    #[serde(skip)]
    _snapshot: Arc<ManagedSnapshot>,
    #[serde(flatten)]
    pub(crate) data: ObservationData,
}

pub(crate) struct ObservationFailure<'session> {
    pub(crate) error: CorsaError,
    pub(crate) partial: Observation<'session>,
}

impl fmt::Debug for ObservationFailure<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObservationFailure")
            .field("error", &self.error)
            .field("partial", &self.partial.data)
            .finish()
    }
}

impl ObservationData {
    fn new(request: UsageRequest, snapshot: SnapshotHandle, project: ProjectResponse) -> Self {
        Self {
            schema: "vize.constructor-vue-return-observation.v1",
            representation: "decoded SDK DTOs and source payloads; not whole wire",
            native_admission: false,
            request,
            snapshot,
            snapshot_projects: Vec::new(),
            project,
            stage: "project-identity",
            source_names: None,
            pending_source: None,
            sources: Vec::new(),
            constructor_type: None,
            construct_signatures_raw: None,
            construct_signatures: None,
            declared_returns: Vec::new(),
            hosts: Vec::new(),
            vue: VueObservation::default(),
            unavailable: vec![
                Unavailable::SelectedSignatureRequiresNativeCallNode,
                Unavailable::GenericClosureNotAdmitted,
                Unavailable::VueReturnOriginNotAdmitted,
            ],
        }
    }
}

pub(crate) async fn capture<'session>(
    session: &'session ProjectSession,
    request: UsageRequest,
) -> Result<Observation<'session>, ObservationFailure<'session>> {
    let snapshot = session.snapshot_lease();
    let mut observation = Observation {
        _session: session,
        data: ObservationData::new(request, snapshot.handle.clone(), session.project().clone()),
        _snapshot: snapshot,
    };
    let data = &mut observation.data;
    data.snapshot_projects = observation._snapshot.projects.clone();
    if !project_matches(
        &data.snapshot_projects,
        &data.project,
        &data.request.config_file,
    ) {
        data.unavailable.push(Unavailable::ProjectIdentity);
        return Ok(observation);
    }
    if !anchor_matches(&data.request.authored_source, &data.request.authored_usage) {
        data.unavailable
            .push(Unavailable::Anchor(data.request.usage_id.clone()));
        return Ok(observation);
    }
    match query::observe(session.client(), data).await {
        Ok(()) => Ok(observation),
        Err(error) => Err(ObservationFailure {
            error,
            partial: observation,
        }),
    }
}

fn project_matches(projects: &[ProjectResponse], selected: &ProjectResponse, config: &str) -> bool {
    selected.config_file_name == config
        && projects
            .iter()
            .filter(|project| project.id == selected.id)
            .count()
            == 1
        && projects.iter().any(|project| project == selected)
}

fn utf16_byte(text: &str, offset: u32) -> Option<usize> {
    let mut units = 0;
    for (index, character) in text.char_indices() {
        if units == offset {
            return Some(index);
        }
        units += character.len_utf16() as u32;
        if units > offset {
            return None;
        }
    }
    (units == offset).then_some(text.len())
}

fn anchor_matches(source: &str, anchor: &Anchor) -> bool {
    if anchor.start >= anchor.end || !(anchor.start..anchor.end).contains(&anchor.position) {
        return false;
    }
    let (Some(start), Some(end), Some(_)) = (
        utf16_byte(source, anchor.start),
        utf16_byte(source, anchor.end),
        utf16_byte(source, anchor.position),
    ) else {
        return false;
    };
    source.get(start..end) == Some(anchor.text.as_str())
}

fn source_matches(expected: &SourceExpectation, actual: &EncodedSourceFile) -> Option<Unavailable> {
    if actual.file_name != expected.file.as_str() || actual.path != expected.path.as_str() {
        Some(Unavailable::SourceIdentity(expected.file.clone()))
    } else if actual.text != expected.text.as_str() {
        Some(Unavailable::SourceText(expected.file.clone()))
    } else {
        None
    }
}

fn source_member(names: Option<&Value>, file: &str) -> bool {
    names.and_then(Value::as_array).is_some_and(|names| {
        names.iter().all(Value::is_string)
            && names
                .iter()
                .filter(|name| name.as_str() == Some(file))
                .count()
                == 1
    })
}

fn retain_signatures(
    data: &mut ObservationData,
    raw: Value,
) -> corsa::Result<Vec<SignatureResponse>> {
    // Retain the complete SDK-boundary value before a DTO default or a
    // decode error can erase missing/null/empty distinctions and extra fields.
    data.construct_signatures_raw = Some(raw.clone());
    data.construct_signatures = None;
    let signatures: Vec<SignatureResponse> = serde_json::from_value(raw)?;
    data.construct_signatures = Some(signatures.clone());
    Ok(signatures)
}

fn retain_signature_type_parameters(
    record: &mut SignatureReturn,
    response: corsa::Result<Value>,
) -> corsa::Result<()> {
    let raw = response?;
    // A native null is missing data, not the SDK wrapper's normalized [].
    // Keep the complete value even when the local typed decode fails.
    record.type_parameters_raw = SignatureTypeParametersRaw::Received(raw.clone());
    record.type_parameters = None;
    if !raw.is_null() {
        record.type_parameters = Some(serde_json::from_value(raw)?);
    }
    Ok(())
}

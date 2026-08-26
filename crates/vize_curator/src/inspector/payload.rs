//! Inspector payload, agent report, and playground URL helpers.

use std::fmt::Write as _;
use vize_s0::{String, ToCompactString};

use super::graph::{InspectorGraph, build_graph, line_count};
use vize_atelier_core::Allocator;
use vize_atelier_sfc::{
    SfcParseOptions, croquis::SfcCroquisOptions, croquis::analyze_sfc_descriptor, parse_sfc,
};
use vize_croquis::{CroquisSemanticSnapshot, CroquisSemanticSummary};

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InspectorTarget {
    Dom,
    Ssr,
    Vapor,
}

impl InspectorTarget {
    fn as_payload_str(self) -> &'static str {
        match self {
            Self::Dom => "dom",
            Self::Ssr => "ssr",
            Self::Vapor => "vapor",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InspectorTemplateSyntax {
    #[default]
    Standard,
    Strict,
    Quirks,
}

#[derive(Debug, Clone, Copy)]
pub struct InspectorOptions {
    pub custom_renderer: bool,
    pub template_syntax: InspectorTemplateSyntax,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct InspectorSourceFile {
    pub path: String,
    pub source: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectorPayload {
    version: u8,
    target: &'static str,
    selected_file: Option<String>,
    options: InspectorPayloadOptions,
    files: Vec<InspectorPayloadFile>,
    /// The Spolvero feed (P2-18): the payload's S1/S2 stage pages as the
    /// schema-versioned document `spolvero-feed.schema.json` commits.
    /// Kept as a pre-serialized value so `SpolveroFeed::to_json` stays the
    /// shape's only serializer (the feed's own snake_case keys included).
    spolvero: serde_json::Value,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InspectorPayloadOptions {
    custom_renderer: bool,
    template_syntax: InspectorTemplateSyntax,
}

#[derive(serde::Serialize)]
struct InspectorPayloadFile {
    path: String,
    source: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectorAgentReport {
    schema: &'static str,
    version: u8,
    generated_by: &'static str,
    playground_url: String,
    summary: InspectorAgentSummary,
    semantic_files: Vec<InspectorSemanticFile>,
    graph: InspectorGraph,
    payload: InspectorPayload,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InspectorAgentSummary {
    target: &'static str,
    selected_file: Option<String>,
    file_count: usize,
    source_bytes: usize,
    source_lines: usize,
    semantic: InspectorSemanticSummary,
    options: InspectorPayloadOptions,
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InspectorSemanticSummary {
    analyzed_files: usize,
    skipped_files: usize,
    sfc_parse_error_files: usize,
    template_parse_error_files: usize,
    scope_count: usize,
    scope_binding_count: usize,
    script_binding_count: usize,
    used_component_count: usize,
    component_registration_count: usize,
    component_usage_count: usize,
    passed_prop_count: usize,
    event_listener_count: usize,
    slot_usage_count: usize,
    used_directive_count: usize,
    prop_definition_count: usize,
    emit_definition_count: usize,
    model_definition_count: usize,
    provide_count: usize,
    inject_count: usize,
    reactivity_loss_count: usize,
    race_condition_count: usize,
    element_id_count: usize,
    undefined_ref_count: usize,
    import_statement_count: usize,
    re_export_count: usize,
    fallthrough_attr_risk_files: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct InspectorSemanticFile {
    path: String,
    snapshot: CroquisSemanticSnapshot,
}

pub fn build_payload(
    target: InspectorTarget,
    options: InspectorOptions,
    files: Vec<InspectorSourceFile>,
) -> InspectorPayload {
    let spolvero = super::spolvero::payload_spolvero(&files);
    let files: Vec<_> = files
        .into_iter()
        .map(|file| InspectorPayloadFile {
            path: file.path,
            source: file.source,
        })
        .collect();

    let selected_file = files.first().map(|file| file.path.clone());

    InspectorPayload {
        version: 1,
        target: target.as_payload_str(),
        selected_file,
        options: InspectorPayloadOptions {
            custom_renderer: options.custom_renderer,
            template_syntax: options.template_syntax,
        },
        files,
        spolvero,
    }
}

pub fn build_agent_report(
    payload: InspectorPayload,
    playground_url: String,
    files: Vec<InspectorSourceFile>,
) -> InspectorAgentReport {
    let source_bytes = files.iter().map(|file| file.source.len()).sum();
    let source_lines = files
        .iter()
        .map(|file| line_count(file.source.as_str()))
        .sum();
    let graph = build_graph(&files);
    let (semantic, semantic_files) = build_semantic_report(&files);
    let summary = InspectorAgentSummary {
        target: payload.target,
        selected_file: payload.selected_file.clone(),
        file_count: payload.files.len(),
        source_bytes,
        source_lines,
        semantic,
        options: InspectorPayloadOptions {
            custom_renderer: payload.options.custom_renderer,
            template_syntax: payload.options.template_syntax,
        },
    };

    InspectorAgentReport {
        schema: "vize.inspector.agent",
        version: 1,
        generated_by: "vize_curator",
        playground_url,
        summary,
        semantic_files,
        graph,
        payload,
    }
}

fn build_semantic_report(
    files: &[InspectorSourceFile],
) -> (InspectorSemanticSummary, Vec<InspectorSemanticFile>) {
    let mut summary = InspectorSemanticSummary::default();
    let mut semantic_files = Vec::new();
    for file in files {
        if !file.path.ends_with(".vue") {
            summary.skipped_files += 1;
            continue;
        }

        let Ok(descriptor) = parse_sfc(file.source.as_str(), SfcParseOptions::default()) else {
            summary.sfc_parse_error_files += 1;
            continue;
        };

        let template_allocator = Allocator::default();
        let template_root = descriptor.template.as_ref().map(|template| {
            let (root, parse_errors) =
                vize_atelier_core::parser::parse(&template_allocator, template.content.as_ref());
            if !parse_errors.is_empty() {
                summary.template_parse_error_files += 1;
            }
            root
        });
        let mut croquis_options = SfcCroquisOptions::for_lint();
        croquis_options
            .analyzer_options
            .collect_template_expressions = true;
        let croquis = analyze_sfc_descriptor(&descriptor, template_root.as_ref(), croquis_options);

        summary.analyzed_files += 1;
        let snapshot = compact_semantic_snapshot(croquis.semantic_snapshot());
        summary.add_croquis(snapshot.summary);
        semantic_files.push(InspectorSemanticFile {
            path: file.path.clone(),
            snapshot,
        });
    }

    (summary, semantic_files)
}

fn compact_semantic_snapshot(mut snapshot: CroquisSemanticSnapshot) -> CroquisSemanticSnapshot {
    for scope in &mut snapshot.scopes {
        if matches!(scope.kind, "univ" | "client" | "server" | "vue") {
            // Preserve the ambient scope topology without serializing the
            // same static JavaScript/Vue global tables for every SFC.
            scope.bindings.clear();
            scope.binding_count = 0;
        }
    }
    snapshot
}

impl InspectorSemanticSummary {
    fn add_croquis(&mut self, summary: CroquisSemanticSummary) {
        self.scope_count += summary.scope_count;
        self.scope_binding_count += summary.scope_binding_count;
        self.script_binding_count += summary.script_binding_count;
        self.used_component_count += summary.used_component_count;
        self.component_registration_count += summary.component_registration_count;
        self.component_usage_count += summary.component_usage_count;
        self.passed_prop_count += summary.passed_prop_count;
        self.event_listener_count += summary.event_listener_count;
        self.slot_usage_count += summary.slot_usage_count;
        self.used_directive_count += summary.used_directive_count;
        self.prop_definition_count += summary.prop_definition_count;
        self.emit_definition_count += summary.emit_definition_count;
        self.model_definition_count += summary.model_definition_count;
        self.provide_count += summary.provide_count;
        self.inject_count += summary.inject_count;
        self.reactivity_loss_count += summary.reactivity_loss_count;
        self.race_condition_count += summary.race_condition_count;
        self.element_id_count += summary.element_id_count;
        self.undefined_ref_count += summary.undefined_ref_count;
        self.import_statement_count += summary.import_statement_count;
        self.re_export_count += summary.re_export_count;
        if summary.may_lose_fallthrough_attrs() {
            self.fallthrough_attr_risk_files += 1;
        }
    }
}

pub fn serialize_payload(payload: &InspectorPayload) -> Result<String, serde_json::Error> {
    serde_json::to_string(payload).map(|json| json.to_compact_string())
}

pub fn serialize_agent_report(report: &InspectorAgentReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report).map(|json| json.to_compact_string())
}

pub fn build_playground_url(base: &str, payload_json: &str) -> String {
    let base_without_hash = base.split('#').next().unwrap_or(base);
    let separator = if base_without_hash.contains('?') {
        if base_without_hash.ends_with('?') || base_without_hash.ends_with('&') {
            ""
        } else {
            "&"
        }
    } else {
        "?"
    };

    let mut url = String::default();
    url.push_str(base_without_hash);
    url.push_str(separator);
    url.push_str("tab=inspector#inspector=");
    url.push_str(percent_encode(payload_json).as_str());
    url
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::default();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                let _ = write!(encoded, "%{byte:02X}");
            }
        }
    }
    encoded
}

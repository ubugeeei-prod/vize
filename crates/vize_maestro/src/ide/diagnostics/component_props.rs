//! Component-surface diagnostics that need template usage plus imported metadata.
#![allow(clippy::disallowed_methods, clippy::disallowed_macros)]

use tower_lsp::lsp_types::{
    CodeDescription, Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Url,
};
use vize_atelier_sfc::SfcDescriptor;
use vize_croquis::{Drawer, DrawerOptions};

use super::{DiagnosticService, LineIndex, sources};
use crate::ide::IdeContext;
use crate::ide::completion::template::component_metadata;
use crate::ide::definition::helpers;
use crate::server::ServerState;

impl DiagnosticService {
    pub(super) fn extend_component_required_prop_diagnostics(
        state: &ServerState,
        uri: &Url,
        content: &str,
        descriptor: &SfcDescriptor<'_>,
        line_index: &LineIndex<'_>,
        enabled: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !enabled {
            return;
        }

        let component_diags = Self::collect_component_required_prop_diagnostics(
            state, uri, content, descriptor, line_index,
        );
        tracing::info!(
            "collect: component required prop diagnostics: {}",
            component_diags.len()
        );
        diagnostics.extend(component_diags);
    }

    fn collect_component_required_prop_diagnostics(
        state: &ServerState,
        uri: &Url,
        content: &str,
        descriptor: &SfcDescriptor<'_>,
        line_index: &LineIndex<'_>,
    ) -> Vec<Diagnostic> {
        let Some(template) = descriptor.template.as_ref() else {
            return Vec::new();
        };
        let Some(template_content) = content.get(template.loc.start..template.loc.end) else {
            return Vec::new();
        };

        let allocator = vize_s0::Allocator::new();
        let (root, _) = vize_armature::parse(&allocator, template_content);
        let mut drawer = Drawer::with_options(DrawerOptions {
            analyze_template_scopes: true,
            track_usage: true,
            ..Default::default()
        });
        drawer.draw_template(&root);
        let croquis = drawer.finish();
        if croquis.component_usages.is_empty() {
            return Vec::new();
        }

        let metadata_ctx =
            IdeContext::with_content(state, uri, template.loc.start, content.to_string());
        let mut diagnostics = Vec::new();

        for usage in croquis.component_usages {
            if usage.has_spread_attrs || usage.props.iter().any(|prop| prop.name_is_dynamic) {
                continue;
            }

            let Some(metadata) = component_metadata(&metadata_ctx, usage.name.as_str()) else {
                continue;
            };
            let missing = metadata
                .props
                .iter()
                .filter(|prop| prop.required)
                .filter(|prop| {
                    !usage.props.iter().any(|passed| {
                        !passed.name_is_dynamic
                            && prop_names_match(passed.name.as_str(), prop.name.as_str())
                    })
                })
                .map(|prop| prop.name.clone())
                .collect::<Vec<_>>();
            if missing.is_empty() {
                continue;
            }

            let tag_name_start = template.loc.start + usage.start as usize + 1;
            let tag_name_end = tag_name_start + usage.name.len();
            let (start_line, start_col) = line_index.line_col(tag_name_start.min(content.len()));
            let (end_line, end_col) = line_index.line_col(tag_name_end.min(content.len()));

            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: start_line,
                        character: start_col,
                    },
                    end: Position {
                        line: end_line,
                        character: end_col,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String(
                    "component-required-props".to_string(),
                )),
                code_description: vue_props_code_description(),
                source: Some(sources::COMPONENTS.to_string()),
                message: required_props_message(usage.name.as_str(), &missing),
                ..Default::default()
            });
        }

        diagnostics
    }
}

fn prop_names_match(passed_name: &str, declared_name: &str) -> bool {
    passed_name == declared_name || helpers::kebab_to_camel(passed_name) == declared_name
}

fn required_props_message(component_name: &str, missing: &[String]) -> String {
    let props = missing
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let noun = if missing.len() == 1 { "prop" } else { "props" };
    format!(
        "<{component_name}> is missing required {noun}: {props}\n\nPass the prop in this template usage, or make it optional/provide a default in the child component."
    )
}

fn vue_props_code_description() -> Option<CodeDescription> {
    Url::parse("https://vuejs.org/guide/components/props.html#prop-validation")
        .ok()
        .map(|href| CodeDescription { href })
}

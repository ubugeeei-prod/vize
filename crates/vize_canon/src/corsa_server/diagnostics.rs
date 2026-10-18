use super::{CorsaServer, Diagnostic};
use crate::corsa_bridge::CorsaVueVirtualProject;
use crate::corsa_client::LspDiagnostic;
use crate::projection::{
    AssembledOrigin, AssemblyPolicy, AuthoredSource, FinishedDiagnostic, ProjectedDocument,
    assemble_diagnostics,
};
use vize_carton::{FxHashSet, String, cstr, line_index::LineIndex};

type DiagnosticKey = (String, String, u32, u32, Option<String>);

/// Collapse diagnostics that started at distinct virtual positions but map to
/// the same authored span. The batch checker already does this after mapping;
/// the persistent check server needs the same boundary because dependency
/// patches can make both a per-prop and whole-props check report one defect.
pub(super) fn dedup_diagnostics(diagnostics: &mut Vec<Diagnostic>) {
    let mut seen = FxHashSet::<DiagnosticKey>::default();
    diagnostics.retain(|diagnostic| {
        seen.insert((
            diagnostic.message.clone(),
            diagnostic.severity.clone(),
            diagnostic.line,
            diagnostic.column,
            diagnostic.code.clone(),
        ))
    });
}

impl CorsaServer {
    /// Run Corsa and map its virtual TypeScript diagnostics back to the SFC.
    pub(super) fn run_corsa(
        &mut self,
        project: &CorsaVueVirtualProject,
        source: &str,
        source_path: &std::path::Path,
        flags: &str,
    ) -> Result<Vec<Diagnostic>, String> {
        let report_unused = reports_unused(&self.working_dir(), source_path);
        self.with_project_session(source_path, flags, |client| {
            if let Some(project_root) = project.session_project_root.as_deref() {
                client.synchronize_materialized_project(
                    project_root,
                    project.session_config_path.as_deref(),
                    &project.materialized_changes,
                )?;
            }
            let documents: Vec<(&str, &str)> = project
                .documents
                .iter()
                .map(|(uri, content)| (uri.as_str(), content.as_str()))
                .collect();
            client.did_open_batch_fast(&documents)?;
            let corsa_diagnostics = client.request_diagnostics(&project.host.request_uri)?;
            let mut diagnostics =
                map_corsa_diagnostics(project, source, corsa_diagnostics, report_unused);
            diagnostics.sort_by(|left, right| {
                left.line
                    .cmp(&right.line)
                    .then(left.column.cmp(&right.column))
                    .then(left.code.cmp(&right.code))
                    .then(left.message.cmp(&right.message))
            });
            Ok(diagnostics)
        })
    }
}

fn map_corsa_diagnostics(
    project: &CorsaVueVirtualProject,
    source: &str,
    diagnostics: Vec<LspDiagnostic>,
    report_unused: bool,
) -> Vec<Diagnostic> {
    let virtual_line_index = LineIndex::new(&project.host.code);
    let source_line_index = LineIndex::new(source);
    let finished = diagnostics
        .into_iter()
        .filter_map(|diagnostic| {
            let start = virtual_line_index.line_col_to_offset(
                diagnostic.range.start.line,
                diagnostic.range.start.character,
            )?;
            let end = virtual_line_index
                .line_col_to_offset(diagnostic.range.end.line, diagnostic.range.end.character)
                .unwrap_or(start)
                .max(start);
            let code = diagnostic.code.map(|code| match code {
                serde_json::Value::Number(number) => cstr!("TS{number}"),
                serde_json::Value::String(code) => code.into(),
                _ => cstr!("{code:?}"),
            });
            Some(FinishedDiagnostic {
                document: 0,
                start,
                end,
                code: typescript_code(code.as_deref()),
                severity: diagnostic
                    .severity
                    .and_then(|value| u8::try_from(value).ok()),
                message: diagnostic.message,
                payload: code,
            })
        })
        .collect();
    let documents = [ProjectedDocument {
        generated: &project.host.code,
        import_map: &project.host.import_source_map,
        mapping: Some(&project.host.mapping),
        tsx: project.host.source_type.is_jsx(),
    }];
    // Assemble the whole authored file once, just like the batch checker and
    // Maestro. Point-wise projection cannot retain or coalesce generated
    // checker give-ups because they deliberately have no authored mapping.
    assemble_diagnostics(
        &AuthoredSource::vue(source),
        &documents,
        finished,
        AssemblyPolicy { report_unused },
    )
    .into_iter()
    .map(|assembled| {
        let (line, column) = source_line_index.line_col(assembled.start);
        let code = match assembled.origin {
            AssembledOrigin::Checker(raw) | AssembledOrigin::MissingVueImport(raw)
                if typescript_code(raw.as_deref()) == assembled.code =>
            {
                raw
            }
            _ => assembled.code.map(|code| cstr!("TS{code}")),
        };
        Diagnostic {
            message: assembled.message,
            severity: match assembled.severity {
                Some(2) => "warning".into(),
                Some(3) => "info".into(),
                Some(4) => "hint".into(),
                _ => "error".into(),
            },
            line: one_based(line),
            column: one_based(column),
            code,
        }
    })
    .collect()
}

/// `vize check` drops `TS6133` unless the tsconfig asks for unused declarations.
/// The server must use that same policy or the two paths disagree (TS-9).
fn reports_unused(project_root: &std::path::Path, source: &std::path::Path) -> bool {
    let root = std::fs::canonicalize(project_root).unwrap_or_else(|_| project_root.to_path_buf());
    let source = std::fs::canonicalize(source).unwrap_or_else(|_| source.to_path_buf());
    let tsconfig = crate::corsa_session_cache::project_tsconfig(&source, &root);
    let Ok(options) = crate::snapshot_tsconfig_compiler_options(&root, &tsconfig) else {
        return false;
    };
    ["noUnusedLocals", "noUnusedParameters"]
        .into_iter()
        .any(|name| {
            options
                .get(name)
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        })
}

fn typescript_code(code: Option<&str>) -> Option<u32> {
    let code = code?;
    code.strip_prefix("TS").unwrap_or(code).parse().ok()
}

fn one_based(position: u32) -> u32 {
    position.saturating_add(1)
}

#[expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#[cfg(test)]
mod tests {
    use super::{dedup_diagnostics, map_corsa_diagnostics, one_based};
    use crate::batch::ImportSourceMap;
    use crate::corsa_bridge::{CorsaVueVirtualDocument, CorsaVueVirtualProject};
    use crate::corsa_client::{LspDiagnostic, LspPosition, LspRange};
    use crate::virtual_ts::VizeMapping;
    use oxc_span::SourceType;
    use serde_json::json;

    const SOURCE: &str = "mapped\n";
    const VIRTUAL_SOURCE: &str = "helper\nmapped\n";

    fn project() -> CorsaVueVirtualProject {
        CorsaVueVirtualProject {
            host: CorsaVueVirtualDocument {
                request_uri: "file:///App.vue.ts".into(),
                code: VIRTUAL_SOURCE.into(),
                pre_rewrite_code: VIRTUAL_SOURCE.into(),
                mapping: crate::virtual_ts::ProjectionMapping::from_spans(vec![VizeMapping {
                    gen_range: 7..13,
                    src_range: 0..6,
                    sub_spans: Vec::new(),
                }]),
                import_source_map: ImportSourceMap::empty(),
                source_type: SourceType::ts(),
                virtual_suffix: ".ts",
                dependencies: Vec::new(),
                resolved_dependencies: Vec::new(),
                materialized_sources: Vec::new(),
                source_catalog: Default::default(),
                session_project_root: None,
            },
            documents: Vec::new(),
            session_project_root: None,
            session_config_path: None,
            materialized_changes: Default::default(),
        }
    }

    fn diagnostic(line: u32, character: u32, severity: i32) -> LspDiagnostic {
        LspDiagnostic {
            range: LspRange {
                start: LspPosition { line, character },
                end: LspPosition {
                    line,
                    character: character + 1,
                },
            },
            severity: Some(severity),
            code: Some(json!(6196)),
            source: Some("ts".into()),
            message: "generated diagnostic".into(),
        }
    }

    fn map(diagnostic: LspDiagnostic) -> Option<super::Diagnostic> {
        let project = project();
        map_corsa_diagnostics(&project, SOURCE, vec![diagnostic], true).pop()
    }

    #[test]
    fn one_based_positions_saturate_at_the_protocol_boundary() {
        assert_eq!(one_based(0), 1);
        assert_eq!(one_based(u32::MAX), u32::MAX);
    }

    #[test]
    fn unmapped_generated_diagnostics_are_dropped_for_every_severity() {
        assert!(map(diagnostic(0, 0, 1)).is_none());
        assert!(map(diagnostic(0, 0, 4)).is_none());
    }

    #[test]
    fn mapped_hint_keeps_its_authored_position() {
        let mapped = map(diagnostic(1, 0, 4)).expect("mapped diagnostic");
        assert_eq!(mapped.severity, "hint");
        assert_eq!(mapped.line, 1);
        assert_eq!(mapped.column, 1);
        assert_eq!(mapped.code.as_deref(), Some("TS6196"));
    }

    #[test]
    fn restores_an_unresolved_authored_vue_ts_specifier() {
        let marker = crate::batch::AUTHORED_VUE_TS_SENTINEL;
        let mut raw = diagnostic(1, 0, 1);
        raw.message = format!(
            "Cannot find module './Missing.vue.ts{marker}' or its corresponding type declarations."
        )
        .into();
        let source = "import Missing from './Missing.vue.ts';\n";
        let project = project();
        let mapped = map_corsa_diagnostics(&project, source, vec![raw], true)
            .pop()
            .expect("mapped diagnostic");

        assert_eq!(
            mapped.message,
            "Cannot find module './Missing.vue.ts' or its corresponding type declarations."
        );
    }

    #[test]
    fn mapped_duplicates_collapse_without_hiding_distinct_diagnostics() {
        let duplicate = super::Diagnostic {
            message: "Type mismatch".into(),
            severity: "error".into(),
            line: 7,
            column: 27,
            code: Some("TS2322".into()),
        };
        let mut diagnostics = vec![
            duplicate.clone(),
            duplicate,
            super::Diagnostic {
                message: "Type mismatch".into(),
                severity: "warning".into(),
                line: 7,
                column: 27,
                code: Some("TS2322".into()),
            },
        ];

        dedup_diagnostics(&mut diagnostics);

        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].severity, "error");
        assert_eq!(diagnostics[1].severity, "warning");
    }

    #[test]
    fn generated_checker_give_ups_reach_the_complete_server_packet() {
        for (raw_code, code) in [
            (json!(2563), 2563),
            (json!("TS2589"), 2589),
            (json!("2590"), 2590),
        ] {
            let mut raw = diagnostic(0, 0, 4);
            raw.code = Some(raw_code);
            let actual = map_corsa_diagnostics(&project(), SOURCE, vec![raw], true);
            assert_eq!(
                serde_json::to_value(actual).unwrap(),
                json!([{
                    "message": format!("Vize internal error: the type-check code generated for this file made the checker give up (TS{code}). Types in this and other files may be degraded. Please report this file to Vize."),
                    "severity": "error",
                    "line": 1,
                    "column": 1,
                    "code": format!("TS{code}")
                }])
            );
        }
    }

    #[test]
    fn server_assembly_keeps_one_generated_give_up_and_normal_authored_findings() {
        let source = "<script setup>const n = 1</script>\r\n<template><p>mapped</p></template>";
        let mut project = project();
        let mapped = source.find("mapped").unwrap();
        project.host.mapping =
            crate::virtual_ts::ProjectionMapping::from_spans(vec![VizeMapping::new(
                7..13,
                mapped..mapped + 6,
            )]);
        let raws = [
            (0, 0, 2563, 1),
            (0, 1, 2589, 1),
            (0, 2, 2590, 1),
            (0, 3, 2493, 1),
            (0, 4, 2353, 1),
            (1, 0, 2589, 2),
            (1, 1, 2322, 1),
        ]
        .into_iter()
        .map(|(line, character, code, severity)| {
            let mut raw = diagnostic(line, character, severity);
            raw.code = Some(json!(code));
            raw
        })
        .collect();
        let actual = map_corsa_diagnostics(&project, source, raws, true);
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            json!([
                {
                    "message": "Vize internal error: the type-check code generated for this file made the checker give up (TS2563). Types in this and other files may be degraded. Please report this file to Vize.",
                    "severity": "error", "line": 2, "column": 11, "code": "TS2563"
                },
                {
                    "message": "generated diagnostic",
                    "severity": "warning", "line": 2, "column": 14, "code": "TS2589"
                },
                {
                    "message": "generated diagnostic",
                    "severity": "error", "line": 2, "column": 15, "code": "TS2322"
                }
            ])
        );
    }
}

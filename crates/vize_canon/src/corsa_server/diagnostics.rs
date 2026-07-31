use super::{CorsaServer, Diagnostic};
use crate::batch::restore_virtual_vue_specifiers;
use crate::corsa_bridge::CorsaVueVirtualProject;
use crate::corsa_client::LspDiagnostic;
use vize_carton::{String, cstr, line_index::LineIndex};

impl CorsaServer {
    /// Run Corsa and map its virtual TypeScript diagnostics back to the SFC.
    pub(super) fn run_corsa(
        &mut self,
        project: &CorsaVueVirtualProject,
        source: &str,
    ) -> Result<Vec<Diagnostic>, String> {
        if self.corsa_client.is_none() {
            let client = crate::corsa_client::CorsaProjectClient::new(
                self.config.corsa_path.as_deref(),
                self.config.working_dir.as_deref(),
            )?;
            self.corsa_client = Some(client);
        }

        let client = self
            .corsa_client
            .as_mut()
            .expect("corsa_client must be initialized above");
        let documents: Vec<(&str, &str)> = project
            .documents
            .iter()
            .map(|(uri, content)| (uri.as_str(), content.as_str()))
            .collect();
        client.did_open_batch_fast(&documents)?;
        let corsa_diagnostics = client.request_diagnostics(&project.host.request_uri)?;
        let virtual_line_index = LineIndex::new(&project.host.code);
        let source_line_index = LineIndex::new(source);

        Ok(corsa_diagnostics
            .into_iter()
            .filter_map(|diagnostic| {
                map_corsa_diagnostic(
                    project,
                    source,
                    &virtual_line_index,
                    &source_line_index,
                    diagnostic,
                )
            })
            .collect())
    }
}

fn map_corsa_diagnostic(
    project: &CorsaVueVirtualProject,
    source: &str,
    virtual_line_index: &LineIndex<'_>,
    source_line_index: &LineIndex<'_>,
    diagnostic: LspDiagnostic,
) -> Option<Diagnostic> {
    let (line, column) = map_host_position_to_source(
        project,
        virtual_line_index,
        source_line_index,
        diagnostic.range.start.line,
        diagnostic.range.start.character,
    )?;
    let severity: String = match diagnostic.severity {
        Some(1) => "error".into(),
        Some(2) => "warning".into(),
        Some(3) => "info".into(),
        Some(4) => "hint".into(),
        _ => "error".into(),
    };
    let code = diagnostic.code.map(|code| match code {
        serde_json::Value::Number(number) => cstr!("TS{number}"),
        serde_json::Value::String(code) => code.into(),
        _ => cstr!("{code:?}"),
    });
    Some(Diagnostic {
        message: restore_virtual_vue_specifiers(&diagnostic.message, source),
        severity,
        line: one_based(line),
        column: one_based(column),
        code,
    })
}

fn one_based(position: u32) -> u32 {
    position.saturating_add(1)
}

fn map_host_position_to_source(
    project: &CorsaVueVirtualProject,
    virtual_line_index: &LineIndex<'_>,
    source_line_index: &LineIndex<'_>,
    line: u32,
    column: u32,
) -> Option<(u32, u32)> {
    let rewritten_offset = virtual_line_index.line_col_to_offset(line, column)?;
    let generated_offset = project
        .host
        .import_source_map
        .get_original_offset(u32::try_from(rewritten_offset).ok()?)
        as usize;
    let mapping = project
        .host
        .mappings
        .iter()
        .find(|mapping| mapping.gen_range.contains(&generated_offset))?;
    let (generated_range, source_range) = mapping
        .sub_span_for_gen(generated_offset)
        .map(|span| (&span.gen_range, &span.src_range))
        .unwrap_or((&mapping.gen_range, &mapping.src_range));
    let delta = generated_offset.saturating_sub(generated_range.start);
    let source_offset = source_range
        .start
        .saturating_add(delta)
        .min(source_range.end.saturating_sub(1));
    Some(source_line_index.line_col(source_offset))
}

#[cfg(test)]
mod tests {
    use oxc_span::SourceType;
    use serde_json::json;
    use vize_carton::line_index::LineIndex;

    use super::{map_corsa_diagnostic, one_based};
    use crate::batch::ImportSourceMap;
    use crate::corsa_bridge::{CorsaVueVirtualDocument, CorsaVueVirtualProject};
    use crate::corsa_client::{LspDiagnostic, LspPosition, LspRange};
    use crate::virtual_ts::VizeMapping;

    const SOURCE: &str = "mapped\n";
    const VIRTUAL_SOURCE: &str = "helper\nmapped\n";

    fn project() -> CorsaVueVirtualProject {
        CorsaVueVirtualProject {
            host: CorsaVueVirtualDocument {
                request_uri: "file:///App.vue.ts".into(),
                code: VIRTUAL_SOURCE.into(),
                pre_rewrite_code: VIRTUAL_SOURCE.into(),
                mappings: vec![VizeMapping {
                    gen_range: 7..13,
                    src_range: 0..6,
                    sub_spans: Vec::new(),
                }],
                import_source_map: ImportSourceMap::empty(),
                source_type: SourceType::ts(),
                virtual_suffix: ".ts",
            },
            documents: Vec::new(),
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
        map_corsa_diagnostic(
            &project,
            SOURCE,
            &LineIndex::new(&project.host.code),
            &LineIndex::new(SOURCE),
            diagnostic,
        )
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
        let mapped = map_corsa_diagnostic(
            &project,
            source,
            &LineIndex::new(&project.host.code),
            &LineIndex::new(source),
            raw,
        )
        .expect("mapped diagnostic");

        assert_eq!(
            mapped.message,
            "Cannot find module './Missing.vue.ts' or its corresponding type declarations."
        );
    }
}

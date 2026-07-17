use super::{CorsaServer, Diagnostic};
use crate::corsa_bridge::CorsaVueVirtualProject;
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
            .map(|diagnostic| {
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
                let (line, column) = map_host_position_to_source(
                    project,
                    &virtual_line_index,
                    &source_line_index,
                    diagnostic.range.start.line,
                    diagnostic.range.start.character,
                )
                .unwrap_or((
                    diagnostic.range.start.line,
                    diagnostic.range.start.character,
                ));
                Diagnostic {
                    message: diagnostic.message,
                    severity,
                    line: one_based(line),
                    column: one_based(column),
                    code,
                }
            })
            .collect())
    }
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
    use super::one_based;

    #[test]
    fn one_based_positions_saturate_at_the_protocol_boundary() {
        assert_eq!(one_based(0), 1);
        assert_eq!(one_based(u32::MAX), u32::MAX);
    }
}

//! Required-prop diagnostics over original component operations retained by a request.
use super::{
    Diagnostic, DiagnosticService, DiagnosticSeverity, IdeContext, LineIndex, NumberOrString,
    Position, Range, ServerState, Url, component_metadata, prop_names_match,
    required_props_message, sources, vue_props_code_description,
};
use vize_l0::SmallVec;
use vize_l1_to_l2::native_file::SetupComponentImports;
use vize_l2::query::ComponentPropIssue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentPropDiagnosticIssue {
    Document,
    SourceRevision,
    Props(ComponentPropIssue),
}

impl DiagnosticService {
    /// Collect the editor's required-prop diagnostics using the same project
    /// metadata, messages and UTF-16 ranges as an ordinary diagnostic request.
    pub fn imported_component_required_props(
        state: &ServerState,
        uri: &Url,
        imports: &SetupComponentImports<'_, '_>,
    ) -> Result<Vec<Diagnostic>, ComponentPropDiagnosticIssue> {
        let source = imports.source();
        let content = state
            .documents
            .text_snapshot(uri)
            .ok_or(ComponentPropDiagnosticIssue::Document)?;
        if !(core::ptr::eq(content.as_str(), source) || content.as_str() == source) {
            return Err(ComponentPropDiagnosticIssue::SourceRevision);
        }
        let Some(first) = imports.sites().first() else {
            return Ok(Vec::new());
        };
        let mut context =
            IdeContext::with_content(state, uri, imports.template_span().start as usize, content);
        context.nonexporting_setup = first.nonexporting_setup_span();
        let mut diagnostics = Vec::new();
        let mut lines = None;
        imports.try_for_each_in_postorder(|site| {
            let mut names = SmallVec::<[&str; 8]>::new();
            site.visit_passed_props(|prop| names.push(prop.name))
                .map_err(ComponentPropDiagnosticIssue::Props)?;
            let Some(metadata) = component_metadata(&context, site.tag()) else {
                return Ok(());
            };
            let missing: Vec<_> = metadata
                .props
                .iter()
                .filter(|required| {
                    required.required
                        && !names
                            .iter()
                            .any(|name| prop_names_match(name, &required.name))
                })
                .map(|prop| prop.name.clone())
                .collect();
            if missing.is_empty() {
                return Ok(());
            }
            let lines = lines.get_or_insert_with(|| LineIndex::new(source));
            let name = site.name_span();
            let (start_line, start_col) = lines.line_col(name.start as usize);
            let (end_line, end_col) = lines.line_col(name.end as usize);
            diagnostics.push(Diagnostic {
                range: Range::new(
                    Position::new(start_line, start_col),
                    Position::new(end_line, end_col),
                ),
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("component-required-props".into())),
                code_description: vue_props_code_description(),
                source: Some(sources::COMPONENTS.into()),
                message: required_props_message(site.tag(), &missing),
                ..Default::default()
            });
            Ok(())
        })?;
        Ok(diagnostics)
    }
}

#[cfg(test)]
mod tests;

//! The editor reports a name a template cannot resolve with the same instance
//! codes as batch checking.

use tower_lsp::lsp_types::{Diagnostic, NumberOrString};
use vize_canon::template_instance_names::{
    instance_diagnostic, is_lexical_lookup_failure, template_content_range,
};
use vize_s0::line_index::LineIndex;

pub(super) fn apply(source: &str, diagnostics: &mut [Diagnostic]) {
    if !diagnostics
        .iter()
        .any(|diagnostic| lexical_lookup_failure(diagnostic).is_some())
    {
        return;
    }
    let Some(template) = template_content_range(source) else {
        return;
    };
    let lines = LineIndex::new(source);
    for diagnostic in diagnostics {
        let Some(code) = lexical_lookup_failure(diagnostic) else {
            continue;
        };
        let position = diagnostic.range.start;
        let in_template = lines
            .line_col_to_offset(position.line, position.character)
            .is_some_and(|offset| template.contains(&offset));
        if !in_template {
            continue;
        }
        if let Some((code, message)) = instance_diagnostic(code, &diagnostic.message) {
            diagnostic.code = Some(NumberOrString::Number(code as i32));
            diagnostic.message = message.to_string();
        }
    }
}

fn lexical_lookup_failure(diagnostic: &Diagnostic) -> Option<u32> {
    let code = match diagnostic.code.as_ref()? {
        NumberOrString::Number(code) => u32::try_from(*code).ok()?,
        NumberOrString::String(code) => code.trim_start_matches("TS").parse().ok()?,
    };
    is_lexical_lookup_failure(code).then_some(code)
}

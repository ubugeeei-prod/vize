//! The `TEMPLATE_ERROR` message: its shape, written and read back here.
//!
//! A template gate fails with one [`SfcError`](crate::types::SfcError) whose message is this prefix
//! followed by the rendered error list
//! ([`CompilerErrorWithSource`](vize_atelier_core::CompilerErrorWithSource)'s
//! `Debug`). That text is pinned by fixtures and read by every host, so it does
//! not change; a reporter that wants the individual errors reads them back
//! with [`template_error_details`].

use vize_atelier_core::errors::{RenderedCompilerError, parse_rendered_error_list};

/// What a `TEMPLATE_ERROR` message starts with.
pub const TEMPLATE_ERRORS_PREFIX: &str = "Template compilation errors: ";

/// The errors a `TEMPLATE_ERROR` message lists, with spans into the template
/// block's content, or `None` when `message` is not exactly such a message.
#[must_use]
pub fn template_error_details(message: &str) -> Option<std::vec::Vec<RenderedCompilerError>> {
    parse_rendered_error_list(message.strip_prefix(TEMPLATE_ERRORS_PREFIX)?)
}

#[cfg(test)]
mod tests {
    use super::{TEMPLATE_ERRORS_PREFIX, template_error_details};
    use crate::{SfcParseOptions, compile_sfc_with_template_syntax, parse_sfc};
    use vize_atelier_core::TemplateSyntaxMode;

    #[test]
    fn a_strict_template_gate_reads_back_to_each_located_error() {
        let source = "<template>\n  <div />\n  <span class=\"a\" />\n</template>\n";
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        // A template-only SFC fails with the gate error itself; one with a
        // script reports it among the result's errors.
        let error = match compile_sfc_with_template_syntax(
            &descriptor,
            Default::default(),
            TemplateSyntaxMode::Strict,
        ) {
            Err(error) => error,
            Ok(result) => {
                let [error] = <[_; 1]>::try_from(result.errors).unwrap();
                error
            }
        };
        assert_eq!(error.code.as_deref(), Some("TEMPLATE_ERROR"));
        let details = template_error_details(&error.message).unwrap();
        let template = descriptor.template.as_ref().unwrap();
        let located: std::vec::Vec<_> = details
            .iter()
            .map(|detail| {
                let (start, end) = detail.span.unwrap();
                (
                    detail.code.as_str(),
                    template
                        .content
                        .get(start as usize..end as usize)
                        .unwrap_or_default(),
                )
            })
            .collect();
        assert_eq!(
            located,
            [
                ("UnexpectedSolidusInTag", "<div />"),
                ("UnexpectedSolidusInTag", "<span class=\"a\" />"),
            ]
        );
    }

    #[test]
    fn other_messages_are_not_template_error_lists() {
        assert_eq!(template_error_details("Script compilation failed"), None);
        let mut prefixed = vize_l0::String::from(TEMPLATE_ERRORS_PREFIX);
        prefixed.push_str("not a list");
        assert_eq!(template_error_details(&prefixed), None);
    }
}

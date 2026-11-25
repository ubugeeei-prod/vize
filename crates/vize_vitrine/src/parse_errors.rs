//! Shared parse-error message rendering for the binding boundaries.

use vize_atelier_core::{CompilerError, CompilerErrorWithSource};

/// The `Parse errors: [...]` message both bindings raise when a template
/// parse fails, with each error's covered source text rendered from its span.
///
/// Recoverable diagnostics, such as the rewrite of a self-closing `<div />`
/// that SFC templates accept or a duplicate attribute, do not fail the parse,
/// as they do not fail `compile`; `None` when only those were reported.
#[expect(
    clippy::disallowed_types,
    reason = "N-API and wasm-bindgen errors carry std `String`s"
)]
pub(crate) fn failure(errors: &[CompilerError], source: &str) -> Option<String> {
    if errors.iter().all(CompilerError::is_recoverable) {
        return None;
    }
    let fatal: Vec<CompilerError> = errors
        .iter()
        .filter(|error| !error.is_recoverable())
        .cloned()
        .collect();
    Some(
        vize_l0::cstr!(
            "Parse errors: {:?}",
            CompilerErrorWithSource::list(&fatal, source)
        )
        .into(),
    )
}

#[cfg(test)]
mod tests {
    use super::failure;
    use vize_atelier_core::options::{CustomElementMatcher, ParserOptions, TemplateSyntaxMode};
    use vize_atelier_core::parser::parse_with_options_custom_elements_and_template_syntax;
    use vize_l0::Allocator;

    /// `parseTemplate`'s outcome under the parser options both bindings use.
    fn template_failure(source: &str) -> Option<String> {
        let allocator = Allocator::new();
        let (_, errors) = parse_with_options_custom_elements_and_template_syntax(
            &allocator,
            source,
            ParserOptions {
                is_pre_tag: |tag| tag == "pre",
                ..Default::default()
            },
            CustomElementMatcher::default(),
            TemplateSyntaxMode::Standard,
        );
        failure(&errors, source)
    }

    #[test]
    fn self_closing_non_void_elements_are_not_parse_failures() {
        assert_eq!(template_failure("<Transition><div /></Transition>"), None);
        assert_eq!(template_failure("<div><span /><p /></div>"), None);
    }

    #[test]
    fn syntax_errors_still_fail_with_only_the_fatal_errors_listed() {
        let message = template_failure("<div /><span>").expect("a missing end tag fails");
        assert_eq!(message.matches("code: ").count(), 1, "{message}");
        assert_eq!(
            message.matches("code: MissingEndTag").count(),
            1,
            "{message}"
        );
    }
}

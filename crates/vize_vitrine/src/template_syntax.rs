#![expect(
    clippy::disallowed_macros,
    reason = "template syntax options are std `String`s decoded from N-API and wasm-bindgen"
)]
#![expect(
    clippy::disallowed_types,
    reason = "template syntax options are std `String`s decoded from N-API and wasm-bindgen"
)]

use vize_atelier_core::TemplateSyntaxMode;

pub(crate) fn resolve_template_syntax(
    template_syntax: Option<&str>,
) -> Result<TemplateSyntaxMode, String> {
    match template_syntax {
        Some("standard") => Ok(TemplateSyntaxMode::Standard),
        None => Ok(TemplateSyntaxMode::Standard),
        Some("strict") => Ok(TemplateSyntaxMode::Strict),
        Some("quirks") => Ok(TemplateSyntaxMode::Quirks),
        Some(value) => Err(format!(
            "Invalid templateSyntax `{value}`. Expected `standard`, `strict`, or `quirks`."
        )),
    }
}

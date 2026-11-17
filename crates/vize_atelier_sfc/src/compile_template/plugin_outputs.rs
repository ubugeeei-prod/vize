//! A template compiles only to an output its declared directives support.
//!
//! A `vue.plugins` plugin lists the outputs it supports (#13074). The SFC
//! compiler decides the output per file (SSR, a `vapor` script attribute or
//! request, otherwise VDOM), so the refusal belongs here rather than in each
//! host, which sees only request flags.

use vize_l0::ToCompactString;

use crate::types::{SfcError, SfcTemplateBlock, TemplateCompileOptions};

/// Fail closed when a declaring plugin does not support `output`.
pub(super) fn refuse_unsupported(
    options: &TemplateCompileOptions,
    template: &SfcTemplateBlock,
    output: &str,
) -> Result<(), SfcError> {
    let Some(message) = options
        .compiler_options
        .as_ref()
        .and_then(|compiler| compiler.directive_contracts.unsupported(output))
    else {
        return Ok(());
    };
    Err(SfcError {
        message,
        code: Some("PLUGIN_OUTPUT_UNSUPPORTED".to_compact_string()),
        loc: Some(template.loc.clone()),
    })
}

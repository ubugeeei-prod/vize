//! Module-scope statements whose authored text cannot be emitted as one
//! mapped line: a hoisted declaration that needs the SFC generics spliced in,
//! and a `<script setup>` import section that runs into the next statement.

use vize_carton::{String, append};

use super::generics::{generic_injection_point, references_any_identifier};
use super::glued_import::GluedImportSection;
use crate::virtual_ts::VizeMapping;

fn push_mapped(ts: &mut String, mappings: &mut Vec<VizeMapping>, text: &str, source: usize) {
    let gen_start = ts.len();
    ts.push_str(text);
    mappings.push(VizeMapping {
        gen_range: gen_start..ts.len(),
        src_range: source..source + text.len(),
        sub_spans: Vec::new(),
    });
}

/// Splice the SFC generic parameters into a hoisted type/interface declaration
/// that references them, so the reference resolves at module scope.
pub(super) fn emit_generic_injected(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    text: &str,
    source: usize,
    injection: Option<&(String, Vec<String>)>,
    type_name: Option<&str>,
) -> bool {
    let Some((defaults, names)) = injection else {
        return false;
    };
    let Some(inject_at) = type_name
        .filter(|_| references_any_identifier(text, names))
        .and_then(|type_name| generic_injection_point(text, type_name))
    else {
        return false;
    };
    let (prefix, suffix) = text.split_at(inject_at);
    push_mapped(ts, mappings, prefix, source);
    // Synthetic parameter list; no corresponding source span.
    append!(*ts, "<{defaults}>");
    // Avoid forming `>=` when the alias has no space before `=`.
    if suffix.starts_with('=') {
        ts.push(' ');
    }
    push_mapped(ts, mappings, suffix, source + prefix.len());
    ts.push('\n');
    true
}

impl GluedImportSection {
    /// Register the section as a module statement, so the setup body omits it.
    pub(super) fn plan(
        script: Option<&str>,
        split: Option<(usize, usize)>,
        module_spans: &mut Vec<(u32, u32)>,
    ) -> Option<Self> {
        let glued = Self::find(script?, split)?;
        module_spans.push(glued.span);
        module_spans.sort_unstable();
        Some(glued)
    }

    /// Emit the section without a line break. The missing terminator is then
    /// reported on the token behind it, which stands for the first token of the
    /// classic `<script>`.
    pub(super) fn emit(
        glued: Option<&Self>,
        span: (u32, u32),
        ts: &mut String,
        mappings: &mut Vec<VizeMapping>,
        script: &str,
        source_offset: &dyn Fn(usize) -> usize,
    ) -> bool {
        let Some(glued) = glued.filter(|glued| glued.span == span) else {
            return false;
        };
        let Some(section) = script.get(span.0 as usize..span.1 as usize) else {
            return false;
        };
        push_mapped(ts, mappings, section, source_offset(span.0 as usize));
        let token_start = ts.len();
        ts.push_str("export {};\n");
        mappings.push(VizeMapping {
            gen_range: token_start..token_start + "export".len(),
            src_range: source_offset(glued.classic_token.0 as usize)
                ..source_offset(glued.classic_token.1 as usize),
            sub_spans: Vec::new(),
        });
        true
    }
}

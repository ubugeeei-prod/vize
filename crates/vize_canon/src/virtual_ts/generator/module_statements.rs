//! Module-scope statements whose authored text cannot be emitted as one
//! mapped line: a hoisted declaration that needs the SFC generics spliced in,
//! and a `<script setup>` import section that runs into the next statement.

use vize_carton::{String, append};

use super::generics::{generic_injection_point, references_any_identifier};
use super::glued_import::GluedImportSection;
use crate::virtual_ts::VizeMapping;

fn push_mapped(
    ts: &mut crate::virtual_ts::TextBuf,
    mappings: &mut Vec<VizeMapping>,
    text: &str,
    source: usize,
) {
    let gen_start = ts.len();
    ts.push_str(text);
    mappings.push(VizeMapping {
        gen_range: gen_start..ts.len(),
        src_range: source..source + text.len(),
        sub_spans: Vec::new(),
    });
}

/// Whether a hoisted declaration receives the SFC generic parameters at module
/// scope: it names one of them and declares no parameter list of its own.
pub(super) fn is_generic_injected(
    text: &str,
    injection: Option<&(String, Vec<String>)>,
    type_name: &str,
) -> bool {
    injection.is_some_and(|(_, names)| {
        references_any_identifier(text, names) && generic_injection_point(text, type_name).is_some()
    })
}

/// Splice the SFC generic parameters into a hoisted type/interface declaration
/// that references them, so the reference resolves at module scope.
///
/// `injected` names every module declaration that receives them. A bare
/// reference to one of those inside this declaration forwards the parameters:
/// `type A<T = any> = { b: B }` would otherwise read `B<any>`, so setup's
/// `A<T>` would lose `T` behind the fallback default.
pub(super) fn emit_generic_injected(
    ts: &mut crate::virtual_ts::TextBuf,
    mappings: &mut Vec<VizeMapping>,
    text: &str,
    source: usize,
    injection: Option<&(String, Vec<String>)>,
    type_name: Option<&str>,
    injected: &[&str],
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
    // Parse only a body that spells an injected name at all.
    let spelled: Vec<String> = injected.iter().copied().map(String::from).collect();
    let forwarded = references_any_identifier(suffix, &spelled)
        .then(|| crate::virtual_ts::type_dependencies::module_type_reference_ends(text, injected))
        .flatten()
        .unwrap_or_default();
    let mut arguments = String::default();
    for (index, name) in names.iter().enumerate() {
        if index > 0 {
            arguments.push_str(", ");
        }
        arguments.push_str(name);
    }
    let mut from = inject_at;
    for end in forwarded.into_iter().filter(|&end| end > inject_at) {
        let Some(segment) = text.get(from..end) else {
            continue;
        };
        push_mapped(ts, mappings, segment, source + from);
        // Synthetic arguments; the authored reference keeps its own span.
        append!(*ts, "<{arguments}>");
        from = end;
    }
    push_mapped(
        ts,
        mappings,
        text.get(from..).unwrap_or_default(),
        source + from,
    );
    ts.push('\n');
    true
}

impl GluedImportSection {
    /// Register the section as a module statement, so the setup body omits it.
    pub(super) fn plan(
        script: Option<&str>,
        split: Option<(usize, usize)>,
        recover_lone_setup: bool,
        module_spans: &mut Vec<(u32, u32)>,
    ) -> Option<Self> {
        let script = script?;
        let glued = if let Some(glued) = Self::find(script, split) {
            glued
        } else {
            let glued = recover_lone_setup
                .then(|| Self::find_lone_setup(script))
                .flatten()?;
            // The recovered prefix AST can own an earlier valid import inside
            // this whole lone section. Emit that authored text only once.
            module_spans.retain(|span| span.0 < glued.span.0 || span.1 > glued.span.1);
            glued
        };
        module_spans.push(glued.span);
        module_spans.sort_unstable();
        Some(glued)
    }

    /// Emit the section without a line break. The token behind it maps to the
    /// actual classic/setup token where Vue reports the missing terminator.
    pub(super) fn emit(
        glued: Option<&Self>,
        span: (u32, u32),
        ts: &mut crate::virtual_ts::TextBuf,
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
            src_range: source_offset(glued.diagnostic_token.0 as usize)
                ..source_offset(glued.diagnostic_token.1 as usize),
            sub_spans: Vec::new(),
        });
        true
    }
}

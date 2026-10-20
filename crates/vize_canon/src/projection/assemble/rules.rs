//! Reportability and placement rules of the assembly pass.

use std::ops::Range;

use vize_atelier_sfc::{SfcDescriptor, SfcParseOptions, parse_sfc};
use vize_carton::{String, cstr};

use super::{
    AssembledDiagnostic, AssembledOrigin, AssemblyPolicy, AuthoredSource, FinishedDiagnostic,
};
use crate::batch::virtual_specifier_message::missing_vue_import_specifier_source;

/// Whether a checker diagnostic is reportable at all, independent of where it
/// lands.
pub fn is_reportable(
    code: Option<u32>,
    severity: Option<u8>,
    message: &str,
    policy: AssemblyPolicy,
) -> bool {
    match code {
        // Virtual-TS generation injects helper bindings that can trip TS2666
        // outside the user's source.
        Some(2666) => return false,
        // Native TypeScript exposes Node Buffer backing stores as
        // `ArrayBuffer | SharedArrayBuffer`, where the projects' pinned
        // TypeScript/@types/node baseline accepted `buffer.slice(...)` as
        // `ArrayBuffer`; stay on that baseline until the native checker can
        // select the project's exact lib surface.
        Some(2322) if is_array_buffer_backing_store_lib_mismatch(message) => return false,
        // TS7044 as a *suggestion* is an inference hint, not a finding; the
        // noImplicitAny family must otherwise surface (#966).
        Some(7044) if severity == Some(4) => return false,
        Some(6133) if !policy.report_unused => return false,
        _ => {}
    }
    !(is_unused_declaration(message) && names_generated_binding(message))
}

fn is_array_buffer_backing_store_lib_mismatch(message: &str) -> bool {
    message
        .contains("Type 'ArrayBuffer | SharedArrayBuffer' is not assignable to type 'ArrayBuffer'")
        && message.contains("SharedArrayBuffer")
}

fn is_unused_declaration(message: &str) -> bool {
    message.contains("is declared but")
        && (message.contains("never read") || message.contains("never used"))
}

/// Generated helper bindings (`__vize_*`) and the template's implicit
/// instance members are never authored declarations.
fn names_generated_binding(message: &str) -> bool {
    [
        "'__", "'$event'", "'$attrs'", "'$slots'", "'$refs'", "'$emit'",
    ]
    .into_iter()
    .any(|name| message.contains(name))
}

/// `TS5097` about an import specifier owned by Canon's generated project.
///
/// This covers both rewritten `.vue.ts`/`.vue.tsx` routes and the synthesized
/// `__vize_src` import used to load an external SFC script block. Authored
/// `.ts` imports must continue to report the diagnostic.
pub(super) fn is_generated_project_import_extension<P>(
    generated: &str,
    diagnostic: &FinishedDiagnostic<P>,
) -> bool {
    is_generated_project_import_extension_at(
        generated,
        diagnostic.code,
        &diagnostic.message,
        diagnostic.start,
        diagnostic.end,
    )
}

pub fn is_generated_project_import_extension_at(
    generated: &str,
    code: Option<u32>,
    message: &str,
    start: usize,
    end: usize,
) -> bool {
    if !is_ts5097_import_extension(code, message) {
        return false;
    }
    let Some(specifier) = specifier_at(generated, start, end) else {
        return false;
    };
    has_extension(specifier, &[".vue.ts", ".vue.tsx"])
        || (has_extension(specifier, &[".ts", ".tsx", ".mts", ".cts"])
            && import_statement_at(generated, start, end)
                .is_some_and(|statement| statement.contains("import __vize_src from")))
}

fn import_statement_at(source: &str, start: usize, end: usize) -> Option<&str> {
    let start = start.min(source.len());
    let end = end.min(source.len()).max(start);
    let line_start = source
        .get(..start)?
        .rfind(['\n', '\r'])
        .map_or(0, |at| at + 1);
    let line_end = source
        .get(end..)?
        .find(['\n', '\r'])
        .map_or(source.len(), |at| end + at);
    source.get(line_start..line_end)
}

/// `TS5097` on an authored `.vue` specifier: SFC imports are always allowed.
pub(crate) fn is_authored_vue_import_extension(
    authored: &str,
    code: Option<u32>,
    message: &str,
    start: usize,
    end: usize,
) -> bool {
    is_ts5097_import_extension(code, message)
        && specifier_at(authored, start, end)
            .is_some_and(|specifier| has_extension(specifier, &[".vue"]))
}

/// `TS5097` caused only by Canon adding a TypeScript extension while routing
/// an authored extensionless or `.vue` import into the virtual project.
pub(crate) fn is_rewritten_ts_import_extension(
    generated: &str,
    authored: &str,
    code: Option<u32>,
    message: &str,
    generated_range: Range<usize>,
    authored_range: Range<usize>,
) -> bool {
    if !is_ts5097_import_extension(code, message) {
        return false;
    }
    let Some(generated) = specifier_at(generated, generated_range.start, generated_range.end)
    else {
        return false;
    };
    let Some(authored) = specifier_at(authored, authored_range.start, authored_range.end) else {
        return false;
    };
    has_extension(generated, &[".ts", ".tsx", ".mts", ".cts"])
        && !has_extension(authored, &[".ts", ".tsx", ".mts", ".cts"])
}

/// TypeScript's CLI formatter only supplies the diagnostic start, so recover
/// the quoted module specifier that contains that zero-width position. LSP
/// diagnostics usually carry the whole literal and take the same path.
fn specifier_at(source: &str, start: usize, end: usize) -> Option<&str> {
    let start = start.min(source.len());
    let end = end.min(source.len()).max(start);
    let line_start = source
        .get(..start)?
        .rfind(['\n', '\r'])
        .map_or(0, |at| at + 1);
    let line_end = source
        .get(start..)?
        .find(['\n', '\r'])
        .map_or(source.len(), |at| start + at);
    let line = source.get(line_start..line_end)?;
    let local_start = start - line_start;
    let local_end = end - line_start;
    let mut quote: Option<(char, usize)> = None;
    let mut escaped = false;

    for (offset, character) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        match quote {
            Some((delimiter, open)) if character == delimiter => {
                let close = offset + character.len_utf8();
                if open <= local_end && local_start <= close {
                    return line.get(open + delimiter.len_utf8()..offset);
                }
                quote = None;
            }
            None if matches!(character, '\'' | '"' | '`') => quote = Some((character, offset)),
            _ => {}
        }
    }
    None
}

fn has_extension(specifier: &str, extensions: &[&str]) -> bool {
    let path = specifier
        .split_once(['?', '#'])
        .map_or(specifier, |(path, _)| path);
    extensions.iter().any(|extension| path.ends_with(extension))
}

fn is_ts5097_import_extension(code: Option<u32>, message: &str) -> bool {
    code == Some(5097) && message.contains("allowImportingTsExtensions")
}

#[cfg(test)]
mod import_extension_tests {
    use super::{
        is_authored_vue_import_extension, is_generated_project_import_extension,
        is_rewritten_ts_import_extension,
    };
    use crate::projection::FinishedDiagnostic;

    const MESSAGE: &str = "An import path can only end with a '.ts' extension when 'allowImportingTsExtensions' is enabled.";

    fn diagnostic(source: &str, needle: &str) -> FinishedDiagnostic<()> {
        let start = source.find(needle).unwrap();
        FinishedDiagnostic {
            document: 0,
            start,
            end: start,
            code: Some(5097),
            severity: Some(1),
            message: MESSAGE.into(),
            payload: (),
        }
    }

    #[test]
    fn generated_vue_extension_is_found_from_a_zero_width_cli_diagnostic() {
        let source = "import Panel from './Panel.vue.ts';\n";
        assert!(is_generated_project_import_extension(
            source,
            &diagnostic(source, "'./Panel.vue.ts'")
        ));
    }

    #[test]
    fn generated_external_script_route_is_found_from_its_internal_binding() {
        let source = "import __vize_src from './component.jsx.ts';\nexport default __vize_src;\n";
        assert!(is_generated_project_import_extension(
            source,
            &diagnostic(source, "'./component.jsx.ts'")
        ));
        let authored = "import component from './component.ts';\n";
        assert!(!is_generated_project_import_extension(
            authored,
            &diagnostic(authored, "'./component.ts'")
        ));
    }

    #[test]
    fn authored_vue_is_suppressed_but_authored_ts_is_not() {
        for (source, suppressed) in [
            ("import Panel from './Panel.vue';\n", true),
            ("import { value } from './value.ts';\n", false),
        ] {
            let start = source.find('\'').unwrap();
            assert_eq!(
                is_authored_vue_import_extension(source, Some(5097), MESSAGE, start, start),
                suppressed
            );
        }
    }

    #[test]
    fn generated_typescript_extension_is_suppressed_only_for_an_authored_rewrite() {
        let generated = "import { value } from '/virtual/src/value.ts';\n";
        let extensionless = "import { value } from './value';\n";
        let explicit = "import { value } from './value.ts';\n";
        let generated_start = generated.find("'/virtual").unwrap();
        let generated_end = generated.find("';").unwrap() + 1;
        for (authored, suppressed) in [(extensionless, true), (explicit, false)] {
            let authored_start = authored.find("'./value").unwrap();
            let authored_end = authored.find("';").unwrap() + 1;
            assert_eq!(
                is_rewritten_ts_import_extension(
                    generated,
                    authored,
                    Some(5097),
                    MESSAGE,
                    generated_start..generated_end,
                    authored_start..authored_end,
                ),
                suppressed
            );
        }
    }
}

/// One elaboration line of an exhaustiveness failure
/// (`Type 'x' is not assignable to type 'never'.`), however indented.
pub fn is_exhaustiveness_elaboration(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("Type '")
        && line.contains(" is not assignable to type ")
        && line.ends_with('.')
}

/// Checker codes that mean the checker stopped analyzing part of the program:
/// TS2563 (control flow analysis limit), TS2589 (instantiation depth) and
/// TS2590 (union too complex). In generated code they have no authored range,
/// and dropping them hides the cause of degraded types elsewhere.
const CHECKER_GIVE_UP_CODES: [u32; 3] = [2563, 2589, 2590];

pub(super) fn is_checker_give_up(code: Option<u32>) -> bool {
    code.is_some_and(|code| CHECKER_GIVE_UP_CODES.contains(&code))
}

/// The end of a give-up's generated range. A give-up the checker reports
/// with no length still covers its whole statement (`check({ ...props });`).
pub(super) fn give_up_statement_end(generated: &str, start: usize, end: usize) -> usize {
    if end > start {
        return end;
    }
    generated
        .get(start..)
        .and_then(|rest| rest.find(";\n"))
        .map_or(end, |offset| start + offset)
}

/// The internal-error diagnostic for a checker give-up in unmapped generated
/// code, placed on the template block (or the file start) of the authored file.
pub(super) fn generated_give_up<P>(
    authored: &AuthoredSource<'_>,
    code: Option<u32>,
    payload: P,
) -> AssembledDiagnostic<P> {
    let start = authored.template().map_or(0, |template| template.start);
    let code_text = code.map_or_else(String::default, |code| cstr!("TS{}", code));
    AssembledDiagnostic {
        start,
        end: start,
        code,
        severity: Some(1),
        message: cstr!(
            "Vize internal error: the type-check code generated for this file made the checker give up ({}). Types in this and other files may be degraded. Please report this file to Vize.",
            code_text
        ),
        origin: AssembledOrigin::GeneratedGiveUp(payload),
    }
}

/// Keep only the first generated give-up of one authored file.
pub(super) fn keep_first_generated_give_up<P>(assembled: &mut Vec<AssembledDiagnostic<P>>) {
    let mut seen = false;
    assembled.retain(|diagnostic| {
        if !matches!(diagnostic.origin, AssembledOrigin::GeneratedGiveUp(_)) {
            return true;
        }
        !core::mem::replace(&mut seen, true)
    });
}

/// A `TS2307` for a generated `.vue` import the projection has no range for
/// lands on the authored specifier literal.
pub(super) fn missing_vue_import<P>(
    authored: &str,
    code: Option<u32>,
    severity: Option<u8>,
    message: String,
    payload: P,
) -> Option<AssembledDiagnostic<P>> {
    if code != Some(2307) {
        return None;
    }
    let specifier = missing_vue_import_specifier_source(&message)?;
    let start = authored_specifier_literal_offset(authored, specifier)
        .or_else(|| authored.find(specifier))?;
    let end = start + specifier.len();
    let message = crate::batch::restore_virtual_vue_specifiers(&message, authored);
    Some(AssembledDiagnostic {
        start,
        end,
        code,
        severity,
        message,
        origin: AssembledOrigin::MissingVueImport(payload),
    })
}

fn authored_specifier_literal_offset(source: &str, authored: &str) -> Option<usize> {
    ['\'', '"', '`'].into_iter().find_map(|quote| {
        let quoted = cstr!("{quote}{authored}{quote}");
        source.find(quoted.as_str()).map(|offset| offset + 1)
    })
}

/// The SFC template block: the whole root `<template>` element when the block
/// has a root match, otherwise its content. The one definition behind
/// `SfcBlockType::Template` in `vize check` and the template range of the
/// assembly pass.
pub(crate) fn sfc_template_block_range(descriptor: &SfcDescriptor) -> Option<Range<usize>> {
    let template = descriptor.template.as_ref()?;
    let (start, end) = if template.has_root_match() {
        (template.loc.tag_start, template.loc.tag_end)
    } else {
        (template.loc.start, template.loc.end)
    };
    (end > start).then_some(start..end)
}

pub(super) fn template_block_range(source: &str) -> Option<Range<usize>> {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).ok()?;
    sfc_template_block_range(&descriptor)
}

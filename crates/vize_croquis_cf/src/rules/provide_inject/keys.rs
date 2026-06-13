use crate::diagnostics::{CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity};
use crate::registry::FileId;
use vize_carton::{CompactString, cstr};
use vize_croquis::provide::ProvideKey;

pub(super) fn provide_key_display(key: &ProvideKey) -> CompactString {
    match key {
        ProvideKey::String(s) | ProvideKey::Symbol(s) => s.clone(),
    }
}

pub(super) fn provide_key_identity(key: &ProvideKey) -> CompactString {
    match key {
        ProvideKey::String(s) => cstr!("string:{s}"),
        ProvideKey::Symbol(s) => cstr!("symbol:{s}"),
    }
}

pub(super) fn create_string_key_diagnostic(
    file_id: FileId,
    key: &CompactString,
    is_provide: bool,
    start: u32,
    end: u32,
) -> CrossFileDiagnostic {
    let api_name = if is_provide { "provide" } else { "inject" };
    CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::ProvideInjectWithoutSymbol {
            key: key.clone(),
            is_provide,
        },
        DiagnosticSeverity::Warning,
        file_id,
        start,
        cstr!(
            "{}('{}') uses a string injection key; prefer Symbol/InjectionKey for typed, collision-safe dependency flow",
            api_name,
            key
        ),
    )
    .with_end_offset(end)
    .with_suggestion(cstr!(
        "Define an InjectionKey, for example `const {}Key: InjectionKey<...> = Symbol('{}')`, then use it in provide() and inject()",
        key,
        key
    ))
}

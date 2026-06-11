//! Converting SFC parse/compile errors into [`Diagnostic`]s, locating them
//! within the original source, and computing per-block source ranges used by
//! the SFC source map.

use std::path::Path;

use vize_carton::{String as CompactString, cstr};

use vize_atelier_sfc::{SfcDescriptor, SfcError, validate_script_setup_semantics_located};

use crate::batch::source_map::SfcBlockRange;
use crate::batch::{Diagnostic, SfcBlockType};

/// Run only the script-setup semantic validators on this SFC. We deliberately
/// avoid `compile_sfc` here — it would do template codegen and script transform
/// work that doubles the wall time of `vize check` (see the regression on PR
/// #675). The validator covers the diagnostics TypeScript cannot derive on its
/// own; parse-level errors are already collected above.
pub(super) fn collect_sfc_compile_diagnostic(
    path: &Path,
    source: &str,
    descriptor: &SfcDescriptor,
) -> Option<Diagnostic> {
    let script_setup = descriptor.script_setup.as_ref()?;

    // Cheap pre-filter: the only validator we currently run targets
    // `const { ... = ... } = defineProps<...>()`. Skip the OXC parse entirely
    // when none of those tokens appear, which is the common case for app
    // components without destructured typed props.
    if !script_setup_has_validator_candidates(&script_setup.content) {
        return None;
    }

    match validate_script_setup_semantics_located(
        &script_setup.content,
        script_setup.loc.start,
        source,
    ) {
        Ok(()) => None,
        Err(error) => Some(sfc_error_to_diagnostic(path, source, descriptor, &error)),
    }
}

/// Cheap byte-level filter — must be a strict superset of the patterns the
/// underlying validators actually fire on, so we never miss a real diagnostic.
fn script_setup_has_validator_candidates(content: &str) -> bool {
    // Validator needs: typed defineProps (`defineProps<...>`) AND a destructure
    // pattern (`{ ... = ... } = defineProps`). The combined presence of these
    // two substrings is a tight enough filter for typical app code.
    content.contains("defineProps<") && content.contains("= defineProps")
}

fn sfc_error_to_diagnostic(
    path: &Path,
    source: &str,
    descriptor: &SfcDescriptor,
    error: &SfcError,
) -> Diagnostic {
    let (line, column, block_type) = if let Some(loc) = error.loc.as_ref() {
        // BlockLocation lines/columns are 1-based; Diagnostic stores them 0-based.
        let line = (loc.start_line as u32).saturating_sub(1);
        let column = (loc.start_column as u32).saturating_sub(1);
        (line, column, None)
    } else {
        let (offset, block_type) = default_diagnostic_offset(descriptor);
        let (line, column) = line_column_for_offset(source, offset);
        (line, column, Some(block_type))
    };

    let message = match error.code.as_deref() {
        Some(code) => cstr!("Vue compile error [{}]: {}", code, error.message),
        None => cstr!("Vue compile error: {}", error.message),
    };

    Diagnostic {
        file: path.to_path_buf(),
        line,
        column,
        message,
        code: None,
        severity: 1,
        block_type,
    }
}

/// Best-effort fallback location for SFC compile errors that carry no `loc`.
/// Points at the start of the most relevant block so the diagnostic lands
/// somewhere clickable instead of at file offset 0.
///
/// Block selection is shared with the rest of the toolchain via
/// [`crate::batch::sfc_block_fallback_offset`] (#1389); when the descriptor has
/// no blocks we keep the historical `(0, Script)` default.
fn default_diagnostic_offset(descriptor: &SfcDescriptor) -> (u32, SfcBlockType) {
    crate::batch::sfc_block_fallback_offset(descriptor)
        .map(|(offset, block_type)| (offset as u32, block_type))
        .unwrap_or((0, SfcBlockType::Script))
}

pub(super) fn invalid_sfc_fallback_virtual_ts() -> CompactString {
    "declare const __vize_component: any;\nexport default __vize_component;\n".into()
}

pub(super) fn diagnostic_for_offset(
    path: &Path,
    source: &str,
    start: u32,
    message: CompactString,
    block_type: SfcBlockType,
) -> Diagnostic {
    let (line, column) = line_column_for_offset(source, start);
    Diagnostic {
        file: path.to_path_buf(),
        line,
        column,
        message,
        code: None,
        severity: 1,
        block_type: Some(block_type),
    }
}

fn line_column_for_offset(source: &str, offset: u32) -> (u32, u32) {
    // LSP `Position.character` is in UTF-16 code units. Shared, UTF-16-correct
    // implementation lives in `vize_carton::line_index` (#1389). The previous
    // local copy emitted *byte* columns, which mismatched the editor's
    // coordinate system on any line containing multi-byte characters (the
    // #1223 class of bug).
    vize_carton::line_index::offset_to_line_col(source, offset as usize)
}

pub(super) fn collect_sfc_block_ranges(descriptor: &SfcDescriptor) -> Vec<SfcBlockRange> {
    let mut blocks = Vec::with_capacity(3);
    if let Some(template) = descriptor.template.as_ref() {
        push_block_range(
            &mut blocks,
            template.loc.start as u32,
            template.content.len() as u32,
            SfcBlockType::Template,
        );
    }
    if let Some(script) = descriptor.script.as_ref() {
        push_block_range(
            &mut blocks,
            script.loc.start as u32,
            script.content.len() as u32,
            SfcBlockType::Script,
        );
    }
    if let Some(script_setup) = descriptor.script_setup.as_ref() {
        push_block_range(
            &mut blocks,
            script_setup.loc.start as u32,
            script_setup.content.len() as u32,
            SfcBlockType::ScriptSetup,
        );
    }
    blocks
}

fn push_block_range(
    blocks: &mut Vec<SfcBlockRange>,
    start: u32,
    len: u32,
    block_type: SfcBlockType,
) {
    if len == 0 {
        return;
    }
    blocks.push(SfcBlockRange {
        start,
        end: start + len,
        block_type,
    });
}

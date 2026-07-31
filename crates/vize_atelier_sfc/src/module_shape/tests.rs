use super::{SfcModuleShape, analyze_module_shape, utf16_offset};

fn shape(code: &str) -> SfcModuleShape {
    analyze_module_shape(code).expect("emitted module should parse")
}

/// The shape the SFC compiler emits for a script-setup component: `_sfc_main`
/// declared, then exported by name.
#[test]
fn sfc_main_declared_then_default_exported() {
    let code = "const _sfc_main = { name: 'X' }\nexport default _sfc_main\n";
    let shape = shape(code);

    assert!(shape.has_default_export);
    assert!(shape.has_sfc_main_defined);
    assert!(shape.default_export_is_sfc_main);
    let start = shape.default_export_start.unwrap() as usize;
    let keyword_end = shape.default_export_keyword_end.unwrap() as usize;
    assert_eq!(&code[start..keyword_end], "export default");
    assert_eq!(
        &code[start..shape.default_export_end.unwrap() as usize],
        "export default _sfc_main"
    );
}

/// A default export that is not the `_sfc_main` identifier: the consumer may not
/// insert before it, so the flag has to distinguish the two.
#[test]
fn an_inline_default_export_is_not_sfc_main() {
    let shape = shape("export default { name: 'X' }\n");
    assert!(shape.has_default_export);
    assert!(!shape.default_export_is_sfc_main);
    assert!(!shape.has_sfc_main_defined);
}

/// A template-only SFC exports `render` by name and has no default export; the
/// consumer builds `_sfc_main` itself from that branch.
#[test]
fn a_named_render_export_is_recognized() {
    let shape = shape("export function render(_ctx, _cache) { return null }\n");
    assert!(shape.has_named_render_export);
    assert!(!shape.has_default_export);
    assert!(!shape.has_named_ssr_render_export);
}

#[test]
fn a_named_ssr_render_export_is_recognized() {
    let shape = shape("export function ssrRender(_ctx, _push) {}\n");
    assert!(shape.has_named_ssr_render_export);
    assert!(!shape.has_named_render_export);
}

/// `export { render }` reaches the same flags as `export function render`.
#[test]
fn a_specifier_list_counts_as_a_named_export() {
    let shape = shape("function render() {}\nexport { render }\n");
    assert!(shape.has_named_render_export);
}

/// A renamed specifier is recognized by its *exported* name, which is what the
/// consumer's `_sfc_main.render = render` assignment refers to.
#[test]
fn a_renamed_specifier_uses_the_exported_name() {
    let shape = shape("function _r() {}\nexport { _r as render }\n");
    assert!(shape.has_named_render_export);
}

/// `export const _sfc_main = …` defines it just as a bare declaration does.
#[test]
fn an_exported_const_declares_sfc_main() {
    let shape = shape("export const _sfc_main = {}\n");
    assert!(shape.has_sfc_main_defined);
}

/// The reason this parses instead of scanning: `export default` inside a string
/// or a comment in the user's own `<script>` reaches the emitted module verbatim
/// and must not be mistaken for the real one.
#[test]
fn export_default_inside_a_string_or_comment_is_not_a_default_export() {
    let shape = shape("const doc = 'export default {}'\n// export default {}\n");
    assert!(!shape.has_default_export);
    assert_eq!(shape.default_export_start, None);
}

/// And when both are present, the offsets point at the real statement rather
/// than at the first textual occurrence.
#[test]
fn offsets_skip_a_decoy_occurrence() {
    let code = "const doc = 'export default {}'\nconst _sfc_main = {}\nexport default _sfc_main\n";
    let shape = shape(code);
    let start = shape.default_export_start.unwrap() as usize;
    assert_eq!(
        &code[start..shape.default_export_end.unwrap() as usize],
        "export default _sfc_main"
    );
}

/// An unusual gap between the two keywords is measured rather than assumed, so
/// the splice point stays correct for a user-authored default export.
#[test]
fn the_keyword_end_measures_the_actual_gap() {
    let code = "export   default {}\n";
    let shape = shape(code);
    let start = shape.default_export_start.unwrap() as usize;
    assert_eq!(
        &code[start..shape.default_export_keyword_end.unwrap() as usize],
        "export   default"
    );
}

/// A comment between the two keywords cannot be measured, and the offset is
/// then absent rather than wrong: the consumer re-derives it from its own parse,
/// where an offset pointing at the statement start would have spliced
/// `const _sfc_main =` in front of the `export default` it replaces.
#[test]
fn an_unmeasurable_keyword_gap_has_no_keyword_end() {
    let shape = shape("export /* why */ default {}\n");
    assert!(shape.has_default_export);
    assert_eq!(shape.default_export_keyword_end, None);
    assert!(shape.default_export_start.is_some());
}

/// The offsets are byte offsets, and the JS consumer slices in UTF-16 code
/// units, so a module with non-ASCII text before its default export -- a
/// template with a non-Latin string literal -- needs converting at the boundary.
#[test]
fn non_ascii_text_moves_the_byte_and_utf16_offsets_apart() {
    let code = "const _hoisted = \"ようこそ\"\nexport default {}\n";
    let shape = shape(code);
    let start = shape.default_export_start.unwrap();
    assert_eq!(&code[start as usize..start as usize + 14], "export default");

    // Four 3-byte characters stand where the JS side counts four code units.
    let utf16_start = utf16_offset(code, start).unwrap();
    assert_eq!(utf16_start, start - 8);
    assert_eq!(
        code.encode_utf16()
            .skip(utf16_start as usize)
            .take(14)
            .collect::<Vec<_>>(),
        "export default".encode_utf16().collect::<Vec<_>>()
    );
}

/// An all-ASCII module -- the common case -- needs no conversion at all.
#[test]
fn an_ascii_prefix_keeps_the_offset() {
    let code = "export default {}\n";
    assert_eq!(utf16_offset(code, 7), Some(7));
    assert_eq!(utf16_offset(code, 0), Some(0));
}

/// Unparseable output yields `None` so the consumer falls back to its own
/// analysis rather than acting on a half-filled shape.
#[test]
fn unparseable_output_has_no_shape() {
    assert_eq!(analyze_module_shape("const = = =\n"), None);
}

/// The TypeScript emit parses too, without the caller saying which it produced.
#[test]
fn the_typescript_emit_parses() {
    let shape = shape("const _sfc_main = {} as const\nexport default _sfc_main\n");
    assert!(shape.has_default_export);
    assert!(shape.default_export_is_sfc_main);
}

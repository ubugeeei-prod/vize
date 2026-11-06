#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use super::{SfcModuleShape, analyze_module_shape};

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

#[test]
fn hmr_import_facts_exclude_types_and_preserve_runtime_binding_kinds() {
    let shape = shape(
        "import type { Gone } from './types';\n\
         import value, { type Hidden, named as local } from './dep';\n\
         import * as namespace from './namespace';\n\
         export default function Example() { return import('./later') }\n",
    );
    assert_eq!(
        shape.imports,
        [
            super::SfcModuleImport {
                source: "./dep".into(),
                local: "value".into(),
                namespace: false,
            },
            super::SfcModuleImport {
                source: "./dep".into(),
                local: "local".into(),
                namespace: false,
            },
            super::SfcModuleImport {
                source: "./namespace".into(),
                local: "namespace".into(),
                namespace: true,
            },
        ]
    );
    assert!(!shape.has_side_effect_import);
    assert!(shape.has_dynamic_import);
    assert_eq!(shape.default_export_local.as_deref(), Some("Example"));
}

#[test]
fn preserved_typescript_hmr_facts_follow_runtime_type_erasure() {
    use super::finalize_module_output;

    let source = "import { BadgeType } from './types';\n\
                  import { value } from './runtime';\n\
                  interface Props { badges: BadgeType[] }\n\
                  const props: Props = { badges: [] };\n\
                  export default { props, value };\n";
    let (code, shape) = finalize_module_output(source.into(), true);
    assert_eq!(code, source);
    assert_eq!(
        shape.unwrap().imports,
        [super::SfcModuleImport {
            source: "./runtime".into(),
            local: "value".into(),
            namespace: false,
        }]
    );
}

#[test]
fn hmr_import_facts_refuse_bare_imports_and_runtime_reexports() {
    let runtime =
        shape("import './effect'; export * from './all'; export { value } from './named';\n");
    assert!(runtime.has_side_effect_import);
    assert!(runtime.has_reexport);

    let type_only = shape(
        "import type {} from './types'; export type * from './all'; export { type Value } from './named';\n",
    );
    assert!(!type_only.has_side_effect_import);
    assert!(!type_only.has_reexport);
    assert!(type_only.imports.is_empty());
}

#[test]
fn finalized_shapes_match_the_exact_output_after_typescript_stripping() {
    use super::finalize_module_output;
    use crate::compile_script::typescript::{ensure_javascript_output, is_plain_javascript};

    for source in [
        "const doc = '😀 export default {}'; const _sfc_main = {}; export default _sfc_main;",
        "export function render(_ctx, _cache) { return null }",
        "const _sfc_main: { name: string } = { name: 'X' }; export default _sfc_main;",
        "enum Kind { X = 1 }; const _sfc_main = { kind: Kind.X }; export default _sfc_main;",
        "const _sfc_main = {} as const; export default _sfc_main; export function ssrRender() {}",
        "// export default decoy\nexport default { text: 'é😀' };",
        "export const value = <number>1; export default { value };",
    ] {
        for preserve_typescript in [false, true] {
            let expected_code = if preserve_typescript {
                source.into()
            } else {
                ensure_javascript_output(source.into())
            };
            let expected_shape = analyze_module_shape(&expected_code);
            let (code, shape) = finalize_module_output(source.into(), preserve_typescript);
            assert_eq!(code, expected_code);
            assert_eq!(shape, expected_shape);
            if !preserve_typescript {
                assert!(is_plain_javascript(&code));
            }
            if let Some(start) = shape.as_ref().and_then(|shape| shape.default_export_start) {
                let start = start as usize;
                assert_eq!(
                    code.get(start..start + "export default".len()),
                    Some("export default")
                );
            }
        }
    }
}

#[test]
fn invalid_or_jsx_output_keeps_its_original_fallback() {
    use super::finalize_module_output;
    for source in ["const = = =", "export default <div />", "return {}"] {
        for preserve_typescript in [false, true] {
            let (code, shape) = finalize_module_output(source.into(), preserve_typescript);
            assert_eq!(code, source);
            assert_eq!(shape, None);
        }
    }
}

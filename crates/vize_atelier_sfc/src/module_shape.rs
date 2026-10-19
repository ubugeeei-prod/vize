//! The shape of an emitted SFC module, so the JS plugins do not have to
//! re-derive it by parsing that module again with oxc (#3425).
//!
//! `generateOutput` in the Vite plugin needs six facts about the module the
//! compiler just produced — whether it has a default export, whether it defines
//! `_sfc_main`, whether it exports a named `render`/`ssrRender`, and where the
//! default export's `export default` keyword starts and ends — so it can splice
//! in `__scopeId` and the CSS-modules assignment. It recovered them by running
//! `parseProgram` over the emitted code, which the #3425 profile measured at
//! ~77 ms against ~49 ms for the native compilation it post-processes. Most of
//! that is not the parse but `jsonParseAst`, the cost of materializing oxc's AST
//! as JavaScript objects across the NAPI boundary.
//!
//! Computing it here removes that boundary crossing entirely: the same oxc parse
//! runs natively and only eight scalars cross.
//!
//! ## Why this is derived from the finished module rather than recorded while
//! writing it
//!
//! The offsets could be recorded at each `export default` write site instead —
//! `compile/output_module.rs`, `compile/empty_component.rs`, and the eight
//! branches in `compile_script/inline/compiler/component_output.rs`. But two
//! later passes rewrite the module wholesale: `ensure_javascript_output` can
//! reprint the entire program through `Codegen`, and `finalize_output_mode`
//! rewrites `export default` to `return` line by line for `mode: "function"`.
//! Every recorded offset would have to be invalidated by both, and an offset
//! that is stale rather than absent is worse than no offset at all — the
//! consumer splices at a byte position, so a wrong one corrupts the module
//! silently.
//!
//! Deriving from the finished `code` is sound by construction: it runs after
//! every pass, so there is nothing left to invalidate.
//!
//! ## Why a parse rather than a scan
//!
//! `analyzeFastDefaultOutput` on the JS side already string-scans, and bails to
//! a parse whenever the module contains `_sfc_main`, `export {`, or a named
//! render export. It bails because `export default` can appear inside a string
//! or a comment in the user's own `<script>`, which is concatenated into this
//! output verbatim. The same hazard applies here, and the offsets are used for
//! splicing, so this parses.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Declaration, ExportDefaultDeclarationKind, Program, Statement, VariableDeclaration,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::profile;

/// The identifier the SFC compiler binds a component to before exporting it.
const SFC_MAIN: &str = "_sfc_main";

/// `export default` — the keyword pair whose end the consumer splices after.
/// What the JS side would otherwise learn by parsing the emitted module.
///
/// Byte offsets are into the emitted `code`, not into the authored SFC.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SfcModuleShape {
    pub has_default_export: bool,
    pub has_sfc_main_defined: bool,
    pub has_named_render_export: bool,
    pub has_named_ssr_render_export: bool,
    /// Start of the `export default` statement.
    pub default_export_start: Option<u32>,
    /// End of the `export default` keyword pair, i.e. where the exported
    /// expression begins. The consumer inserts text before this.
    pub default_export_keyword_end: Option<u32>,
    /// End of the whole `export default` statement.
    pub default_export_end: Option<u32>,
    /// Whether the default export is the bare `_sfc_main` identifier, which is
    /// what lets the consumer insert before it rather than rewriting it.
    pub default_export_is_sfc_main: bool,
}

/// Analyze an emitted module, or `None` when it does not parse.
///
/// `None` rather than a partly-filled shape: the consumer falls back to its own
/// analysis, which is exactly the behaviour before this existed. A module that
/// fails to parse here is one the build is going to fail on anyway.
pub fn analyze_module_shape(code: &str) -> Option<SfcModuleShape> {
    let allocator = Allocator::default();
    // TypeScript is a superset, so this parses both the JS and the TS emit
    // without the caller having to say which it produced.
    let parsed = profile!(
        "atelier.sfc.module_shape.parse",
        Parser::new(&allocator, code, SourceType::ts()).parse()
    );
    if !parsed.diagnostics.is_empty() {
        return None;
    }

    Some(shape_from_program(code, &parsed.program))
}

/// Finalize compiler output and derive its shape from the same parse.
///
/// Most output is already JavaScript. Its validation AST also provides the
/// module shape, avoiding a second parse for every Vite/Nuxt component. When
/// TypeScript must be stripped, analyze the rewritten bytes so splice offsets
/// still describe exactly the module returned to the bundler.
pub fn finalize_module_output(
    code: vize_carton::String,
    preserve_typescript: bool,
) -> (vize_carton::String, Option<SfcModuleShape>) {
    let (code, shape, _) = finalize(code, preserve_typescript, None);
    (code, shape)
}

/// [`finalize_module_output`] that also carries `map` (the module's source
/// map, as `SfcCompileResult::map` holds it) through a TypeScript strip, so the
/// returned map describes the returned bytes (Davinci P3-9).
pub fn finalize_module_output_with_map(
    code: vize_carton::String,
    preserve_typescript: bool,
    map: Option<serde_json::Value>,
) -> (
    vize_carton::String,
    Option<SfcModuleShape>,
    Option<serde_json::Value>,
) {
    finalize(code, preserve_typescript, map)
}

type Finalized = (
    vize_carton::String,
    Option<SfcModuleShape>,
    Option<serde_json::Value>,
);

fn finalize(
    code: vize_carton::String,
    preserve_typescript: bool,
    map: Option<serde_json::Value>,
) -> Finalized {
    if preserve_typescript {
        let shape = analyze_module_shape(&code);
        return (code, shape, map);
    }

    let shape = {
        let allocator = Allocator::default();
        let parsed = profile!(
            "atelier.script.js.probe",
            Parser::new(&allocator, &code, SourceType::mjs()).parse()
        );
        parsed
            .diagnostics
            .is_empty()
            .then(|| shape_from_program(&code, &parsed.program))
    };
    if shape.is_some() {
        return (code, shape, map);
    }

    let stripped = crate::compile_script::typescript::strip_typescript_for_emitter_traced(
        &code,
        map.is_some(),
    );
    let Some((stripped, reprint)) = stripped else {
        let shape = analyze_module_shape(&code);
        return (code, shape, map);
    };
    let map = map.zip(reprint).and_then(|(map, reprint)| {
        crate::module_map::remap_reprinted(&map, &code, &stripped, &reprint)
    });
    let shape = analyze_module_shape(&stripped);
    (stripped, shape, map)
}

fn shape_from_program(code: &str, program: &Program<'_>) -> SfcModuleShape {
    let mut shape = SfcModuleShape::default();
    for statement in &program.body {
        match statement {
            Statement::ExportDefaultDeclaration(export) => {
                let span = export.span();
                shape.has_default_export = true;
                shape.default_export_start = Some(span.start);
                shape.default_export_end = Some(span.end);
                shape.default_export_keyword_end =
                    keyword_end(code, span.start).or(Some(span.start));
                shape.default_export_is_sfc_main = exports_sfc_main(&export.declaration);
            }
            Statement::VariableDeclaration(declaration) => {
                if declares_sfc_main(declaration) {
                    shape.has_sfc_main_defined = true;
                }
            }
            Statement::ExportNamedDeclaration(export) => {
                for name in exported_names(export) {
                    match name {
                        "render" => shape.has_named_render_export = true,
                        "ssrRender" => shape.has_named_ssr_render_export = true,
                        _ => {}
                    }
                }
                if let Some(Declaration::VariableDeclaration(declaration)) = &export.declaration
                    && declares_sfc_main(declaration)
                {
                    shape.has_sfc_main_defined = true;
                }
            }
            _ => {}
        }
    }
    shape
}

/// End of the `export default` keyword pair starting at `start`.
///
/// The emitter always writes the two words separated by a single space, but a
/// user's own `<script>` reaches this output verbatim and may not, so the gap is
/// measured rather than assumed.
fn keyword_end(code: &str, start: u32) -> Option<u32> {
    let rest = code.get(start as usize..)?;
    let after_export = rest.strip_prefix("export")?;
    let gap = after_export.len() - after_export.trim_start().len();
    let after_gap = after_export.trim_start();
    if !after_gap.starts_with("default") {
        return None;
    }
    let consumed = "export".len() + gap + "default".len();
    Some(start + u32::try_from(consumed).ok()?)
}

fn exports_sfc_main(declaration: &ExportDefaultDeclarationKind<'_>) -> bool {
    matches!(
        declaration,
        ExportDefaultDeclarationKind::Identifier(identifier) if identifier.name == SFC_MAIN
    )
}

fn declares_sfc_main(declaration: &VariableDeclaration<'_>) -> bool {
    declaration.declarations.iter().any(|declarator| {
        declarator
            .id
            .get_binding_identifier()
            .is_some_and(|identifier| identifier.name == SFC_MAIN)
    })
}

/// Names a `export ...` statement introduces, whether through a declaration or
/// a specifier list.
fn exported_names<'a>(
    export: &'a oxc_ast::ast::ExportNamedDeclaration<'a>,
) -> impl Iterator<Item = &'a str> {
    let from_declaration = export
        .declaration
        .iter()
        .filter_map(|declaration| match declaration {
            Declaration::FunctionDeclaration(function) => {
                function.id.as_ref().map(|id| id.name.as_str())
            }
            Declaration::ClassDeclaration(class) => class.id.as_ref().map(|id| id.name.as_str()),
            _ => None,
        });
    let from_variables = export
        .declaration
        .iter()
        .filter_map(|declaration| match declaration {
            Declaration::VariableDeclaration(variables) => Some(variables),
            _ => None,
        })
        .flat_map(|variables| {
            variables
                .declarations
                .iter()
                .filter_map(|declarator| declarator.id.get_binding_identifier())
                .map(|identifier| identifier.name.as_str())
        });
    let from_specifiers = export
        .specifiers
        .iter()
        .map(|specifier| specifier.exported.name().as_str());
    from_declaration
        .chain(from_variables)
        .chain(from_specifiers)
}

#[cfg(test)]
mod tests {
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
        let code =
            "const doc = 'export default {}'\nconst _sfc_main = {}\nexport default _sfc_main\n";
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
}

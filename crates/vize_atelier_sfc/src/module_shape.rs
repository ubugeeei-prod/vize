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
use oxc_ast::ast::{Declaration, ExportDefaultDeclarationKind, Statement, VariableDeclaration};
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

    let mut shape = SfcModuleShape::default();
    for statement in &parsed.program.body {
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
    Some(shape)
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
mod tests;

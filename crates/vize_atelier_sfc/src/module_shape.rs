//! The shape of an emitted SFC module, so the JS plugins do not have to
//! re-derive it by parsing that module again with oxc (#3425).
//!
//! Output assembly needs export geometry so it can splice in `__scopeId` and
//! CSS-module assignments. HMR also needs the value imports that survive type
//! elision, plus conservative side-effect, re-export and dynamic-import flags
//! (#11619). Those consumers previously recovered the facts by parsing the
//! finished JavaScript again. The #3425 profile measured one such `parseProgram`
//! at ~77 ms against ~49 ms for the native compilation it post-processes; most
//! of that was `jsonParseAst`, the cost of materializing oxc's AST as JavaScript
//! objects across the NAPI boundary.
//!
//! Computing the complete shape here removes that boundary crossing entirely:
//! the existing native parse supplies compact scalars and import records.
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
    Comment, Declaration, ExportDefaultDeclarationKind, ImportDeclarationSpecifier,
    ImportExpression, Program, Statement, VariableDeclaration,
};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use oxc_syntax::{identifier::is_white_space, line_terminator::is_line_terminator};
use vize_carton::profile;

/// The identifier the SFC compiler binds a component to before exporting it.
const SFC_MAIN: &str = "_sfc_main";

/// One runtime import binding in the finished emitted module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfcModuleImport {
    pub source: vize_carton::String,
    pub local: vize_carton::String,
    pub namespace: bool,
}

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
    /// Local binding exported by default, for identifiers and named declarations.
    pub default_export_local: Option<vize_carton::String>,
    /// Runtime import bindings after type-only imports have been excluded.
    pub imports: Vec<SfcModuleImport>,
    /// A bare runtime import cannot be compared without replaying its effects.
    pub has_side_effect_import: bool,
    /// Re-exported live bindings do not have a local value to snapshot.
    pub has_reexport: bool,
    /// Dynamic dependencies keep HMR reuse conservative.
    pub has_dynamic_import: bool,
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

    shape_from_program(code, &parsed.program)
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
            .flatten()
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

fn shape_from_program(code: &str, program: &Program<'_>) -> Option<SfcModuleShape> {
    let mut shape = SfcModuleShape::default();
    let mut dynamic_imports = DynamicImports::default();
    dynamic_imports.visit_program(program);
    shape.has_dynamic_import = dynamic_imports.found;
    for statement in &program.body {
        match statement {
            Statement::ExportDefaultDeclaration(export) => {
                let span = export.span();
                shape.has_default_export = true;
                shape.default_export_start = Some(span.start);
                shape.default_export_end = Some(span.end);
                shape.default_export_keyword_end =
                    Some(keyword_end(code, span.start, &program.comments)?);
                shape.default_export_is_sfc_main = exports_sfc_main(&export.declaration);
                shape.default_export_local = default_export_local(&export.declaration);
            }
            Statement::ImportDeclaration(import) => collect_import(import, &mut shape),
            Statement::ExportAllDeclaration(export) => {
                if !export.export_kind.is_type() {
                    shape.has_reexport = true;
                }
            }
            Statement::VariableDeclaration(declaration) => {
                if declares_sfc_main(declaration) {
                    shape.has_sfc_main_defined = true;
                }
            }
            Statement::ExportNamedDeclaration(export) => {
                if export.source.is_some()
                    && !crate::compile_script::statement_sections::is_type_only_export(export)
                {
                    shape.has_reexport = true;
                }
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

#[derive(Default)]
struct DynamicImports {
    found: bool,
}

impl<'a> Visit<'a> for DynamicImports {
    fn visit_import_expression(&mut self, _expression: &ImportExpression<'a>) {
        self.found = true;
    }
}

fn collect_import(import: &oxc_ast::ast::ImportDeclaration<'_>, shape: &mut SfcModuleShape) {
    if import.import_kind.is_type() {
        return;
    }
    let source = import.source.value.as_str();
    let Some(specifiers) = &import.specifiers else {
        shape.has_side_effect_import = true;
        return;
    };
    if specifiers.is_empty() {
        shape.has_side_effect_import = true;
        return;
    }
    for specifier in specifiers {
        let (local, namespace) = match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                if specifier.import_kind.is_type() {
                    continue;
                }
                (specifier.local.name.as_str(), false)
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                (specifier.local.name.as_str(), false)
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                (specifier.local.name.as_str(), true)
            }
        };
        shape.imports.push(SfcModuleImport {
            source: source.into(),
            local: local.into(),
            namespace,
        });
    }
}

/// End of the `export default` keyword pair starting at `start`.
///
/// The emitter always writes the two words separated by a single space, but a
/// user's own `<script>` reaches this output verbatim and may not, so the gap is
/// measured rather than assumed. Comments come from the existing module parse.
fn keyword_end(code: &str, start: u32, comments: &[Comment]) -> Option<u32> {
    let rest = code.get(start as usize..)?;
    let after_export = rest.strip_prefix("export")?;
    let mut cursor = code.len() - after_export.len();
    let mut comment_index = comments.partition_point(|comment| comment.span.end as usize <= cursor);
    loop {
        let rest = code.get(cursor..)?;
        let after_space = rest.trim_start_matches(|c| is_white_space(c) || is_line_terminator(c));
        cursor += rest.len() - after_space.len();
        let Some(comment) = comments
            .get(comment_index)
            .filter(|comment| comment.span.start as usize == cursor)
        else {
            break;
        };
        cursor = comment.span.end as usize;
        comment_index += 1;
    }
    code.get(cursor..)?.strip_prefix("default")?;
    u32::try_from(cursor + "default".len()).ok()
}

fn exports_sfc_main(declaration: &ExportDefaultDeclarationKind<'_>) -> bool {
    matches!(
        declaration,
        ExportDefaultDeclarationKind::Identifier(identifier) if identifier.name == SFC_MAIN
    )
}

fn default_export_local(
    declaration: &ExportDefaultDeclarationKind<'_>,
) -> Option<vize_carton::String> {
    match declaration {
        ExportDefaultDeclarationKind::Identifier(identifier) => {
            Some(identifier.name.as_str().into())
        }
        ExportDefaultDeclarationKind::FunctionDeclaration(function) => function
            .id
            .as_ref()
            .map(|identifier| identifier.name.as_str().into()),
        ExportDefaultDeclarationKind::ClassDeclaration(class) => class
            .id
            .as_ref()
            .map(|identifier| identifier.name.as_str().into()),
        _ => None,
    }
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

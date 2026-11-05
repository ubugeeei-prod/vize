//! TypeScript erasure of a `<script setup>` module's authored pieces, and the
//! import elision that the erased module needs.
//!
//! Only authored text holds TypeScript: render code, component glue and
//! rebuilt imports are emitted as JavaScript. Each authored piece is erased on
//! its own, and its semantic pass reports the names it uses as values. An
//! import is kept when the finished module uses it as a value. This is the
//! rule oxc's TypeScript transform applies with `only_remove_type_imports:
//! false`, and compiler-sfc with esbuild's defaults: `import type` and inline
//! `type` specifiers go, side-effect imports stay, and tsconfig's
//! `verbatimModuleSyntax` / `importsNotUsedAsValues` are not read.

use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, Program, Statement};
use oxc_codegen::Codegen;
use oxc_parser::{ParseOptions, Parser};
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_span::{GetSpan, SourceType};
use oxc_transformer::{JsxOptions, TransformOptions, Transformer, TypeScriptOptions};
use vize_carton::{FxHashSet, String, ToCompactString, profile};

use super::typescript::print_javascript;
use crate::module_map::{Runs, apply_edits};
use crate::script::ScriptCompileContext;

/// The value names the module's pieces use, collected while erasing them.
#[derive(Default)]
pub(crate) struct Erasure {
    values: FxHashSet<String>,
    /// A piece could not be analyzed, so its names are unknown.
    unknown: bool,
}

impl Erasure {
    /// An erasure for JavaScript output, with `ctx`'s macro arguments erased.
    pub(crate) fn for_javascript(javascript: bool, ctx: &mut ScriptCompileContext) -> Option<Self> {
        let mut erasure = javascript.then(Self::default)?;
        erasure.macro_arguments(ctx);
        Some(erasure)
    }

    /// `code` erased when there is an erasure, untraced.
    pub(crate) fn erase_statements(erasure: Option<&mut Self>, code: String) -> String {
        match erasure {
            Some(erasure) => erasure.statements(&code, false).0,
            None => code,
        }
    }

    /// Erase TypeScript from a statement list; `trace` asks for provenance.
    pub(crate) fn statements(&mut self, code: &str, trace: bool) -> (String, Option<Runs>) {
        let allocator = Allocator::default();
        let Some(program) = profile!(
            "atelier.script.erase.statements",
            self.erase(&allocator, code, true)
        ) else {
            return (code.to_compact_string(), None);
        };
        print_javascript(&program, code, trace)
    }

    /// Erase TypeScript from one expression. Its names count only once the
    /// emitted text holding it is recorded ([`Self::generated`]): a macro
    /// argument the compiler drops uses nothing.
    pub(crate) fn expression(&mut self, code: &str) -> String {
        const PREFIX: &str = "x = ";
        const SUFFIX: &str = ";\n";
        let mut wrapped = String::with_capacity(code.len() + PREFIX.len() + 2);
        wrapped.push_str(PREFIX);
        wrapped.push_str(code);
        wrapped.push_str("\n;");
        let allocator = Allocator::default();
        let Some(program) = profile!(
            "atelier.script.erase.expression",
            self.erase(&allocator, &wrapped, false)
        ) else {
            return code.to_compact_string();
        };
        let (printed, _) = print_javascript(&program, &wrapped, false);
        match printed
            .strip_prefix(PREFIX)
            .and_then(|rest| rest.strip_suffix(SUFFIX))
        {
            Some(expression) if program.body.len() == 1 => expression.to_compact_string(),
            _ => {
                self.unknown = true;
                code.to_compact_string()
            }
        }
    }

    /// Erase the macro arguments the compiler copies into component options
    /// and `setup`: runtime props and emits, `defineOptions`,
    /// `defineExpose` and props-destructure defaults.
    pub(crate) fn macro_arguments(&mut self, ctx: &mut ScriptCompileContext) {
        let macros = &mut ctx.macros;
        for call in [
            &mut macros.define_props,
            &mut macros.define_emits,
            &mut macros.define_options,
            &mut macros.define_expose,
        ]
        .into_iter()
        .flatten()
        {
            if !call.args.trim().is_empty() {
                call.args = self.expression(&call.args);
            }
        }
        if let Some(destructure) = macros.props_destructure.as_mut() {
            for binding in destructure.bindings.values_mut() {
                self.optional_expression(&mut binding.default);
            }
        }
    }

    pub(crate) fn optional_expression(&mut self, code: &mut Option<String>) {
        if let Some(code) = code {
            *code = self.expression(code);
        }
    }

    /// [`Self::javascript`] for one expression.
    pub(crate) fn javascript_expression(&mut self, code: &str) {
        if code.trim().is_empty() {
            return;
        }
        let mut wrapped = String::with_capacity(code.len() + 3);
        wrapped.push('(');
        wrapped.push_str(code);
        wrapped.push_str("\n)");
        self.javascript(&wrapped);
    }

    /// Record the value names of JavaScript kept as written. A top-level
    /// `return` is accepted: generated setup tails end in one.
    pub(crate) fn javascript(&mut self, code: &str) {
        let allocator = Allocator::default();
        let options = ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        };
        let parsed = profile!(
            "atelier.script.erase.javascript",
            Parser::new(&allocator, code, SourceType::mjs())
                .with_options(options)
                .parse()
        );
        if parsed.panicked || !parsed.diagnostics.is_empty() {
            self.unknown = true;
            return;
        }
        let semantic = SemanticBuilder::new().build(&parsed.program);
        self.note_free_values(&semantic.semantic.into_scoping());
    }

    /// Record a name generated code uses as a value.
    pub(crate) fn note(&mut self, name: &str) {
        self.values.insert(name.to_compact_string());
    }

    pub(crate) fn uses(&self, name: &str) -> bool {
        self.values.contains(name)
    }

    /// Whether imports may be elided: every piece's names are known.
    pub(crate) fn is_complete(&self) -> bool {
        !self.unknown
    }

    /// `import` without the specifiers the module never uses as values;
    /// `None` when none is left. Side-effect imports are kept as written.
    pub(crate) fn elide(&self, import: &str) -> Option<String> {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, import, SourceType::ts()).parse();
        let mut program = parsed.program;
        let [Statement::ImportDeclaration(declaration)] = program.body.as_mut_slice() else {
            return Some(import.to_compact_string());
        };
        if !parsed.diagnostics.is_empty() {
            return Some(import.to_compact_string());
        }
        if declaration.import_kind.is_type() {
            return None;
        }
        let Some(specifiers) = declaration.specifiers.as_mut() else {
            return Some(import.to_compact_string());
        };
        if specifiers.is_empty() {
            declaration.specifiers = None;
            return Some(print_import(&program, import));
        }
        let before = specifiers.len();
        specifiers.retain(|specifier| match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(named) => {
                !named.import_kind.is_type() && self.uses(&named.local.name)
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                self.uses(&default.local.name)
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace) => {
                self.uses(&namespace.local.name)
            }
        });
        if specifiers.is_empty() {
            return None;
        }
        if specifiers.len() == before {
            return Some(import.to_compact_string());
        }
        Some(print_import(&program, import))
    }

    /// `code` erased; with `record`, its free value names are noted.
    fn erase<'a>(
        &mut self,
        allocator: &'a Allocator,
        code: &'a str,
        record: bool,
    ) -> Option<Program<'a>> {
        // The module goal: a setup body may `await` at its top level.
        let source_type = SourceType::ts().with_module(true);
        let parsed = crate::script::parse_script_text(allocator, code, source_type);
        if parsed.panicked || !parsed.diagnostics.is_empty() {
            self.unknown = true;
            return None;
        }
        let mut program = parsed.program;
        // `with_enum_eval(true)`: oxc's enum transform needs evaluated members.
        let semantic = SemanticBuilder::new()
            .with_excess_capacity(2.0)
            .with_enum_eval(true)
            .build(&program);
        if !semantic.diagnostics.is_empty() {
            self.unknown = true;
            return None;
        }
        let scoping = semantic.semantic.into_scoping();
        if record {
            self.note_free_values(&scoping);
        }
        // `lang="tsx"` keeps its JSX for the bundler's JSX plugin.
        let jsx = if program.source_type.is_jsx() {
            JsxOptions::disable()
        } else {
            JsxOptions::default()
        };
        let options = TransformOptions {
            typescript: TypeScriptOptions {
                only_remove_type_imports: false,
                ..TypeScriptOptions::default()
            },
            jsx,
            ..TransformOptions::default()
        };
        let transformed = Transformer::new(allocator, std::path::Path::new(""), &options)
            .build_with_scoping(scoping, &mut program);
        if !transformed.diagnostics.is_empty() {
            self.unknown = true;
            return None;
        }
        Some(program)
    }

    /// The names a piece reads or writes as values without declaring them.
    /// Type positions and `typeof` queries are not values.
    fn note_free_values(&mut self, scoping: &Scoping) {
        for (name, references) in scoping.root_unresolved_references() {
            if references
                .iter()
                .any(|&id| scoping.get_reference(id).flags().is_value())
            {
                self.values.insert(name.as_str().to_compact_string());
            }
        }
    }
}

fn print_import(program: &Program<'_>, import: &str) -> String {
    let printed = Codegen::new().with_source_text(import).build(program).code;
    let mut line = printed.trim_end().to_compact_string();
    line.push('\n');
    line
}

/// The module with the compiler's own imports (`code[..end]`) left without
/// the specifiers the rest of the module never names, and its provenance in
/// `code`; `None` when every specifier has a recorded value reference.
pub(crate) fn elide_generated_imports(
    code: &str,
    end: usize,
    erasure: &Erasure,
) -> Option<(String, Runs)> {
    let head = code.get(..end)?;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, head, SourceType::mjs()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    for statement in &parsed.program.body {
        let Statement::ImportDeclaration(declaration) = statement else {
            continue;
        };
        let Some(specifiers) = declaration.specifiers.as_ref() else {
            continue;
        };
        let kept: Vec<&str> = specifiers
            .iter()
            .filter(|specifier| erasure.uses(&specifier.local().name))
            .filter_map(|specifier| {
                head.get(specifier.span().start as usize..specifier.span().end as usize)
            })
            .collect();
        if kept.len() == specifiers.len() {
            continue;
        }
        if kept.is_empty() {
            let (start, mut stop) = (
                declaration.span.start as usize,
                declaration.span.end as usize,
            );
            if head
                .get(stop..)
                .is_some_and(|after| after.starts_with('\n'))
            {
                stop += 1;
            }
            edits.push((start, stop, String::default()));
        } else if let (Some(first), Some(last)) = (specifiers.first(), specifiers.last()) {
            let joined: String = kept.join(", ").into();
            edits.push((
                first.span().start as usize,
                last.span().end as usize,
                joined,
            ));
        }
    }
    if edits.is_empty() {
        return None;
    }
    Some(apply_edits(
        code,
        edits
            .iter()
            .map(|(start, stop, text)| (*start, *stop, text.as_str())),
    ))
}

/// Emit a compiler-owned runtime reference and record it at its write site.
pub(crate) fn emit_runtime_helper(
    output: &mut Vec<u8>,
    erasure: Option<&mut Erasure>,
    alias: &str,
) {
    output.extend_from_slice(alias.as_bytes());
    if let Some(erasure) = erasure {
        erasure.note(alias);
    }
}

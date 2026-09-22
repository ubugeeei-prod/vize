//! The `Bindings` spec's input relations, read by an independent parse.
//!
//! One [`Decl`] per name a top-level `<script setup>` statement declares,
//! in statement order, with the syntactic [`Form`] the rules classify. The
//! walk is deliberately naive: it re-parses the script, looks at top-level
//! statements only and records shapes — it classifies nothing.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, ImportDeclarationSpecifier, Statement, VariableDeclarationKind,
    VariableDeclarator,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::CompactString;

mod shapes;

use shapes::{init_shape, pattern_source};

/// The declaring keyword of a variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarKind {
    /// `const`, `using`, `await using`.
    Const,
    /// `let`, `var`.
    Mutable,
}

/// A variable initializer's shape, seen through `(…)`, `as`, `satisfies`
/// and `!` for [`Init::Call`] and the literal/function/aggregate classes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Init {
    /// No initializer.
    Absent,
    /// A literal: string, number, boolean, null, bigint, an
    /// interpolation-free template, or `-<number>`.
    Literal,
    /// An arrow function or function expression.
    Function,
    /// An object or array literal.
    Aggregate,
    /// A call whose callee is a plain identifier.
    Call(CompactString),
    /// A bare identifier (a possible API alias: `const r = ref`).
    Identifier(CompactString),
    /// Anything else.
    Other,
}

/// How a destructuring pattern's source reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternSource {
    /// `defineProps(…)` or `withDefaults(defineProps(…), …)`, unwrapped.
    DefineProps,
    /// A `defineModel(…)` call (array patterns only).
    DefineModel,
    /// The raw initializer is a function expression.
    Function,
    /// Any other initializer.
    Other,
    /// No initializer.
    Absent,
}

/// The syntactic form a declared name comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Form {
    /// `import { a }` / `import { a as b }`.
    NamedImport,
    /// `import a` / `import * as a`.
    DefaultImport,
    /// `import type …` / `import { type a }`.
    TypeImport,
    /// `function f() {}` / `class C {}` / a runtime `enum E {}`.
    Declaration,
    /// `const x = …` / `let x = …`.
    Simple(VarKind, Init),
    /// A name bound inside an object or array pattern.
    Pattern {
        kind: VarKind,
        source: PatternSource,
        /// The name is a direct property value (or its default's target)
        /// of an object pattern — what a props destructure binds.
        direct: bool,
    },
}

/// One declared name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decl {
    pub name: CompactString,
    pub form: Form,
    /// The identifier span, recorded where production records one.
    pub span: Option<(u32, u32)>,
}

/// Why a script falls outside the `Bindings` spec's scope.
pub const SKIP_PARSE: &str = "parse-error";
/// A renamed import out of `vue`: production resolves call aliases.
pub const SKIP_VUE_ALIAS: &str = "vue-import-alias";

/// Read every top-level declaration of a `<script setup>` source.
///
/// # Errors
///
/// A skip reason when the source is outside the spec's scope.
pub fn extract(source: &str, tsx: bool) -> Result<Vec<Decl>, &'static str> {
    let allocator = Allocator::default();
    let path = if tsx { "script.tsx" } else { "script.ts" };
    let source_type = SourceType::from_path(path).unwrap_or_default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return Err(SKIP_PARSE);
    }
    let mut decls = Vec::new();
    for statement in &parsed.program.body {
        read_statement(statement, &mut decls)?;
    }
    Ok(decls)
}

fn read_statement(statement: &Statement<'_>, out: &mut Vec<Decl>) -> Result<(), &'static str> {
    match statement {
        Statement::ImportDeclaration(import) => {
            let from_vue = import.source.value.as_str() == "vue";
            for specifier in import.specifiers.iter().flatten() {
                let (local, form) = match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(named) => {
                        let type_only = import.import_kind.is_type() || named.import_kind.is_type();
                        let renamed = named.imported.name().as_str() != named.local.name.as_str();
                        if from_vue && renamed && !type_only {
                            return Err(SKIP_VUE_ALIAS);
                        }
                        let form = if type_only {
                            Form::TypeImport
                        } else {
                            Form::NamedImport
                        };
                        (&named.local, form)
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                        (&default.local, import_form(import.import_kind.is_type()))
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace) => {
                        (&namespace.local, import_form(import.import_kind.is_type()))
                    }
                };
                out.push(Decl {
                    name: CompactString::new(local.name.as_str()),
                    form,
                    span: Some((local.span.start, local.span.end)),
                });
            }
        }
        Statement::FunctionDeclaration(function) => {
            if let Some(id) = &function.id {
                out.push(named_declaration(
                    id.name.as_str(),
                    id.span.start,
                    id.span.end,
                ));
            }
        }
        Statement::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                out.push(named_declaration(
                    id.name.as_str(),
                    id.span.start,
                    id.span.end,
                ));
            }
        }
        Statement::TSEnumDeclaration(enumeration)
            if !enumeration.r#const && !enumeration.declare =>
        {
            let id = &enumeration.id;
            out.push(named_declaration(
                id.name.as_str(),
                id.span.start,
                id.span.end,
            ));
        }
        Statement::VariableDeclaration(declaration) => {
            let kind = match declaration.kind {
                VariableDeclarationKind::Let | VariableDeclarationKind::Var => VarKind::Mutable,
                _ => VarKind::Const,
            };
            for declarator in &declaration.declarations {
                read_declarator(declarator, kind, out);
            }
        }
        _ => {}
    }
    Ok(())
}

fn import_form(type_only: bool) -> Form {
    if type_only {
        Form::TypeImport
    } else {
        Form::DefaultImport
    }
}

fn named_declaration(name: &str, start: u32, end: u32) -> Decl {
    Decl {
        name: CompactString::new(name),
        form: Form::Declaration,
        span: Some((start, end)),
    }
}

fn read_declarator(declarator: &VariableDeclarator<'_>, kind: VarKind, out: &mut Vec<Decl>) {
    let init = declarator.init.as_ref();
    match &declarator.id {
        BindingPattern::BindingIdentifier(id) => out.push(Decl {
            name: CompactString::new(id.name.as_str()),
            form: Form::Simple(kind, init.map_or(Init::Absent, init_shape)),
            span: Some((id.span.start, id.span.end)),
        }),
        BindingPattern::ObjectPattern(object) => {
            let source = pattern_source(init, false);
            for property in &object.properties {
                let direct = direct_name(&property.value).is_some();
                push_pattern(&property.value, kind, source, direct, out);
            }
            if let Some(rest) = &object.rest {
                let direct = direct_name(&rest.argument).is_some();
                push_pattern(&rest.argument, kind, source, direct, out);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            let source = pattern_source(init, true);
            for element in array.elements.iter().flatten() {
                push_pattern(element, kind, source, false, out);
            }
            if let Some(rest) = &array.rest {
                push_pattern(&rest.argument, kind, source, false, out);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            push_pattern(&assign.left, kind, PatternSource::Absent, false, out);
        }
    }
}

/// The identifier a props destructure binds for this property value.
fn direct_name<'p>(pattern: &'p BindingPattern<'_>) -> Option<&'p str> {
    match pattern {
        BindingPattern::BindingIdentifier(id) => Some(id.name.as_str()),
        BindingPattern::AssignmentPattern(assign) => direct_name(&assign.left),
        _ => None,
    }
}

fn push_pattern(
    pattern: &BindingPattern<'_>,
    kind: VarKind,
    source: PatternSource,
    direct: bool,
    out: &mut Vec<Decl>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => out.push(Decl {
            name: CompactString::new(id.name.as_str()),
            form: Form::Pattern {
                kind,
                source,
                direct,
            },
            span: None,
        }),
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                push_pattern(&property.value, kind, source, false, out);
            }
            if let Some(rest) = &object.rest {
                push_pattern(&rest.argument, kind, source, false, out);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                push_pattern(element, kind, source, false, out);
            }
            if let Some(rest) = &array.rest {
                push_pattern(&rest.argument, kind, source, false, out);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            push_pattern(&assign.left, kind, source, direct, out);
        }
    }
}

//! Lifting plain-`<script>` `namespace` declarations out of `__setup()`.
//!
//! The plain-`<script>` body is moved inside `__setup()` so its diagnostics stay
//! anchored to user code (see [`super::plain_exports`]). A `namespace` cannot
//! survive that move: TypeScript only accepts one at the top level of a module
//! or namespace, so a namespace left in the function body raises TS1235, and the
//! `__setup()` export bridge never reaches it either — a consumer importing the
//! name gets TS2614. Both are false: the authored `<script>` body *is* module
//! scope in the real SFC.
//!
//! So the declaration is emitted at module scope verbatim. Three consequences
//! have to be handled for that relocation to type-check:
//!
//! 1. **Merging.** A namespace merges with a same-named `function`, `class` or
//!    `enum`. Leaving that partner behind as `export const C = …` makes the
//!    module-scope namespace collide with a block-scoped variable (TS2451 /
//!    TS2300), so the partner declaration is hoisted with the namespace and
//!    dropped from the export bridge, reproducing the authored merge exactly.
//! 2. **Capture.** The namespace body may read a binding that stays inside
//!    `__setup()`. Those names are re-declared at module scope as ambient
//!    aliases of the setup return (`ReturnType<typeof __setup>["x"]`), which
//!    keeps the original type without duplicating the initializer.
//! 3. **Ordering.** The bridge's `export const x = …` is emitted *after* the
//!    namespace, so a captured name that is also exported cannot rely on it
//!    (TS2448, used before declaration). For those names the ambient alias
//!    carries the `export` instead and the bridge skips the value line.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Declaration, IdentifierReference, Statement, TSModuleDeclarationName};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};
use vize_carton::{CompactString, FxHashSet, String as VizeString, append};

use super::plain_exports::{PlainScriptExport, collect_declaration_exports};

/// Plain-`<script>` declarations that have to be emitted at module scope so a
/// `namespace` is not stranded inside `__setup()`.
#[derive(Debug, Default)]
pub(crate) struct NamespaceHoistPlan {
    /// Script-relative spans to emit at module scope verbatim.
    spans: Vec<(u32, u32)>,
    /// Names whose own declaration moved to module scope (merge partners). The
    /// export bridge must not re-declare them.
    hoisted: FxHashSet<CompactString>,
    /// `__setup()`-scoped bindings the hoisted bodies read, in source order.
    captured: Vec<PlainScriptExport>,
}

impl NamespaceHoistPlan {
    /// Only a plain `<script>` needs the plan: when a `<script setup>` block is
    /// present the whole plain block is already emitted at module scope.
    pub(crate) fn collect(
        script: Option<&str>,
        has_script_setup: bool,
        has_plain_script_scope: bool,
    ) -> Self {
        if has_script_setup || !has_plain_script_scope {
            return Self::default();
        }
        script.map(Self::from_script).unwrap_or_default()
    }

    fn from_script(script: &str) -> Self {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, script, SourceType::ts().with_module(true)).parse();
        let parsed = if parsed.panicked {
            Parser::new(&allocator, script, SourceType::tsx().with_module(true)).parse()
        } else {
            parsed
        };
        if parsed.panicked {
            return Self::default();
        }

        let mut spans = Vec::new();
        let mut namespace_names = FxHashSet::default();
        for statement in &parsed.program.body {
            let Some((declaration, span)) = top_level_declaration(statement) else {
                continue;
            };
            let Declaration::TSModuleDeclaration(module) = declaration else {
                continue;
            };
            // `declare module "pkg"` is an augmentation, not a mergeable name.
            if let TSModuleDeclarationName::Identifier(id) = &module.id {
                namespace_names.insert(CompactString::new(id.name.as_str()));
            }
            spans.push((span.start, span.end));
        }
        if spans.is_empty() {
            return Self::default();
        }

        let mut hoisted = FxHashSet::default();
        for statement in &parsed.program.body {
            let Some((declaration, span)) = top_level_declaration(statement) else {
                continue;
            };
            let Some(name) = merge_partner_name(declaration) else {
                continue;
            };
            if namespace_names.contains(&name) {
                spans.push((span.start, span.end));
                hoisted.insert(name);
            }
        }

        let captured = collect_captures(&parsed.program.body, &spans, &hoisted);
        Self {
            spans,
            hoisted,
            captured,
        }
    }

    /// Spans to add to the generator's module-scope span list.
    pub(crate) fn spans(&self) -> &[(u32, u32)] {
        &self.spans
    }

    /// Drops bridge entries whose declaration moved to module scope, and marks
    /// captured names as declared by [`Self::emit_ambient_captures`] instead.
    pub(crate) fn reconcile_exports(&self, exports: &mut Vec<PlainScriptExport>) {
        if self.spans.is_empty() {
            return;
        }
        exports.retain(|export| !self.hoisted.contains(&export.name));
        for export in exports.iter_mut() {
            if self.captures(&export.name) {
                export.bridged_value = false;
            }
        }
    }

    /// Ambient module-scope aliases for the captured setup bindings. Emitted
    /// before the hoisted declarations so the namespace bodies can read them.
    pub(crate) fn emit_ambient_captures(&self, ts: &mut VizeString, exports: &[PlainScriptExport]) {
        if self.captured.is_empty() {
            return;
        }
        ts.push_str("// Setup-scope bindings a hoisted namespace body reads\n");
        for capture in &self.captured {
            let name = &capture.name;
            let exported = exports.iter().any(|export| export.name == *name);
            let visibility = if exported { "export " } else { "" };
            append!(
                *ts,
                "{visibility}declare const {name}: ReturnType<typeof __setup>[\"{name}\"];\n"
            );
            // An exported capture already gets its type side from the bridge;
            // emitting the alias twice would be a duplicate identifier.
            if !exported && let Some(body) = capture.kind.type_alias_body(name) {
                append!(*ts, "type {name} = {body};\n");
            }
        }
        ts.push('\n');
    }

    /// Captured names must leave `__setup()` through its return object, which is
    /// what the ambient aliases index into. Exported ones are already there.
    pub(crate) fn push_captured_return_fields(
        &self,
        exports: &[PlainScriptExport],
        fields: &mut Vec<CompactString>,
    ) {
        for capture in &self.captured {
            if !exports.iter().any(|export| export.name == capture.name) {
                fields.push(capture.name.clone());
            }
        }
    }

    fn captures(&self, name: &CompactString) -> bool {
        self.captured.iter().any(|capture| capture.name == *name)
    }
}

/// The declaration a top-level statement introduces, plus the span that has to
/// move as a unit — the `export` keyword included, so the relocated namespace
/// keeps exporting itself.
fn top_level_declaration<'a>(statement: &'a Statement<'a>) -> Option<(&'a Declaration<'a>, Span)> {
    match statement {
        Statement::ExportNamedDeclaration(export) => {
            if export.source.is_some() || export.export_kind.is_type() {
                return None;
            }
            Some((export.declaration.as_ref()?, export.span))
        }
        statement => statement
            .as_declaration()
            .map(|declaration| (declaration, statement.span())),
    }
}

/// The name a namespace can merge with. `const`/`let` are absent on purpose:
/// TypeScript does not merge a namespace into a variable, so hoisting one would
/// not repair the collision.
fn merge_partner_name(declaration: &Declaration<'_>) -> Option<CompactString> {
    let name = match declaration {
        Declaration::FunctionDeclaration(function) => function.id.as_ref()?.name.as_str(),
        Declaration::ClassDeclaration(class) => class.id.as_ref()?.name.as_str(),
        Declaration::TSEnumDeclaration(enumeration) => enumeration.id.name.as_str(),
        _ => return None,
    };
    Some(CompactString::new(name))
}

/// Setup-scope value bindings referenced from inside the hoisted spans.
///
/// Type aliases and interfaces are excluded because the generator already emits
/// those at module scope, so a hoisted body resolves them without help.
fn collect_captures(
    body: &[Statement<'_>],
    spans: &[(u32, u32)],
    hoisted: &FxHashSet<CompactString>,
) -> Vec<PlainScriptExport> {
    let is_hoisted = |span: Span| {
        spans
            .iter()
            .any(|&(start, end)| span.start >= start && span.end <= end)
    };

    let mut references = ReferencedNames::default();
    let mut seen = FxHashSet::default();
    let mut setup_bindings = Vec::new();
    for statement in body {
        let Some((declaration, span)) = top_level_declaration(statement) else {
            continue;
        };
        if is_hoisted(span) {
            references.visit_statement(statement);
            continue;
        }
        if merge_partner_name(declaration).is_some_and(|name| hoisted.contains(&name)) {
            continue;
        }
        collect_declaration_exports(declaration, &mut seen, &mut setup_bindings);
    }

    setup_bindings
        .into_iter()
        .filter(|binding| references.names.contains(&binding.name))
        .collect()
}

/// Every identifier read inside a subtree, value and type positions alike:
/// `typeof x` in a namespace body needs the same module-scope alias that `x + 1`
/// does. Member property names are `IdentifierName`, so `obj.total` never
/// reports `total`.
#[derive(Default)]
struct ReferencedNames {
    names: FxHashSet<CompactString>,
}

impl<'a> Visit<'a> for ReferencedNames {
    fn visit_identifier_reference(&mut self, reference: &IdentifierReference<'a>) {
        self.names
            .insert(CompactString::new(reference.name.as_str()));
    }
}

#[cfg(test)]
mod tests;

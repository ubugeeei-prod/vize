//! Owned, inferred `useTemplateRef('name')` calls and their template keys.
//!
//! The Vue toolchain types a call that is not nested in a function by indexing
//! the template's ref registry with its literal key, so a name the template
//! never registers is an error on that key. A call inside a function keeps
//! Vue's own signature and is left alone.
//!
//! Only inferred callees use a private alias. The index access is placed in
//! front of the statement that holds it, on the generated line the statement
//! starts on: an authored `@ts-expect-error` above the statement covers it, and
//! the key maps back to its authored literal. The call itself may span lines
//! (`useTemplateRef(\n'key')`), so the line of the key is not a place a
//! statement can be inserted at; a statement that shares its first line with
//! other code is left unchecked for the same reason.

use oxc_ast::ast::{
    Argument, CallExpression, Expression, ImportDeclarationSpecifier, ImportOrExportKind, Program,
    Statement,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::Scoping;
use oxc_span::GetSpan;
use oxc_syntax::scope::ScopeFlags;
use oxc_syntax::symbol::SymbolId;
use vize_carton::{FxHashSet, String, append, cstr};

use crate::virtual_ts::VizeMapping;
use crate::virtual_ts::helpers::push_ts_string_literal;

mod globals;
mod lines;

#[derive(Default)]
pub(super) struct TemplateRefUsage {
    calls: Vec<InferredCall>,
    helper: String,
}

pub(super) struct TemplateRefKeyChecks<'a> {
    calls: &'a [InferredCall],
    helper: &'a str,
    next: usize,
}

struct InferredCall {
    callee: std::ops::Range<usize>,
    name: String,
    imported: bool,
    /// Where the top-level statement holding the call starts.
    statement_start: usize,
    /// A key can be nonliteral; its call still needs the inferred signature.
    literal: Option<(std::ops::Range<usize>, String)>,
}

impl TemplateRefUsage {
    /// Reuse setup-helper parsing and resolved symbols. Typed calls and calls
    /// inside functions retain Vue's own signature and never read the registry.
    pub(super) fn collect(program: &Program<'_>, scoping: &Scoping, script: &str) -> Self {
        let mut collector = Collector {
            script,
            scoping,
            imports: vue_imports(program),
            calls: Vec::new(),
            function_depth: 0,
            statement_depth: 0,
            statement_start: 0,
        };
        collector.visit_program(program);
        collector.calls.sort_by_key(|call| call.callee.start);
        Self {
            helper: if collector.calls.is_empty() {
                String::default()
            } else {
                helper_name(scoping)
            },
            calls: collector.calls,
        }
    }

    pub(super) fn needs_registry(&self) -> bool {
        !self.calls.is_empty()
    }

    pub(super) fn helper_name(&self) -> &str {
        self.helper.as_str()
    }

    /// A rewritten call still uses its authored import. Keep only those
    /// imported bindings live for `noUnusedLocals`, never unrelated imports.
    pub(super) fn emit_import_anchors(&self, ts: &mut String) {
        let mut emitted = FxHashSet::default();
        for call in &self.calls {
            if call.imported && emitted.insert(call.name.as_str()) {
                append!(*ts, "void {};\n", call.name);
            }
        }
    }

    pub(super) fn key_checks(&self, registered: bool) -> TemplateRefKeyChecks<'_> {
        TemplateRefKeyChecks {
            calls: if registered { &self.calls } else { &[] },
            helper: self.helper.as_str(),
            next: 0,
        }
    }
}

impl TemplateRefKeyChecks<'_> {
    /// Emit the checks of every statement that starts on the line `[start, end)`.
    pub(super) fn emit_for_line(
        &mut self,
        ts: &mut String,
        mappings: &mut Vec<VizeMapping>,
        (start, end): (usize, usize),
        source_offset: &dyn Fn(usize) -> usize,
    ) {
        while let Some(call) = self.calls.get(self.next)
            && call.statement_start < end
        {
            self.next += 1;
            let Some((literal, key)) = &call.literal else {
                continue;
            };
            if call.statement_start < start {
                continue;
            }
            if call.imported {
                ts.push_str("void (null! as __VizeTemplateRefs[");
            } else {
                // A missing global produces the checker's error-type `any`.
                // Keep that on the object side of the conditional: using it
                // as the index would invent TS2538 beside the real TS2304.
                append!(
                    *ts,
                    "void (null! as ({}IsVue extends true ? __VizeTemplateRefs : Record<string, never>)[",
                    self.helper
                );
            }
            let literal_start = ts.len();
            push_ts_string_literal(ts, key.as_str());
            mappings.push(VizeMapping {
                gen_range: literal_start..ts.len(),
                src_range: source_offset(literal.start)..source_offset(literal.end),
                sub_spans: Vec::new(),
            });
            ts.push_str("]); ");
        }
    }
}

fn vue_imports(program: &Program<'_>) -> Vec<SymbolId> {
    program
        .body
        .iter()
        .filter_map(|statement| {
            let Statement::ImportDeclaration(import) = statement else {
                return None;
            };
            (import.source.value == "vue" && import.import_kind != ImportOrExportKind::Type)
                .then_some(import.specifiers.as_ref())
                .flatten()
        })
        .flat_map(|specifiers| specifiers.iter())
        .filter_map(|specifier| {
            let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier else {
                return None;
            };
            (named.import_kind != ImportOrExportKind::Type
                && named.imported.name() == "useTemplateRef")
                .then(|| named.local.symbol_id.get())
                .flatten()
        })
        .collect()
}

/// Reserve lexical bindings and unresolved globals so the helper cannot capture
/// authored code. Every candidate has the same byte length as `useTemplateRef`.
fn helper_name(scoping: &Scoping) -> String {
    let names = || {
        scoping
            .symbol_ids()
            .map(|symbol| scoping.symbol_name(symbol))
            .chain(
                scoping
                    .root_unresolved_references()
                    .keys()
                    .map(|name| name.as_str()),
            )
    };
    if !names().any(|name| occupies_helper(name, "__vizeInferRef")) {
        return "__vizeInferRef".into();
    }
    let occupied: FxHashSet<&str> = names().collect();
    let mut index = 0_u32;
    loop {
        let candidate = cstr!("__vize{index:08x}");
        if !occupied
            .iter()
            .any(|name| occupies_helper(name, candidate.as_str()))
        {
            return candidate;
        }
        index += 1;
    }
}

fn occupies_helper(name: &str, candidate: &str) -> bool {
    ["", "Source", "Equal", "Result", "IsVue", "Global"]
        .iter()
        .any(|suffix| name.strip_prefix(candidate) == Some(suffix))
}

struct Collector<'s> {
    script: &'s str,
    scoping: &'s Scoping,
    imports: Vec<SymbolId>,
    calls: Vec<InferredCall>,
    function_depth: usize,
    statement_depth: usize,
    /// Where the top-level statement being visited starts.
    statement_start: usize,
}

impl Collector<'_> {
    /// Whether nothing but indentation precedes the current statement on its
    /// line, so that the start of the line is a statement boundary.
    fn statement_opens_its_line(&self) -> bool {
        let Some(before) = self.script.get(..self.statement_start) else {
            return false;
        };
        let line = before.rsplit_once('\n').map_or(before, |(_, line)| line);
        line.trim().is_empty()
    }
}

impl<'a> Visit<'a> for Collector<'_> {
    fn visit_statement(&mut self, statement: &Statement<'a>) {
        if self.statement_depth == 0 {
            self.statement_start = statement.span().start as usize;
        }
        self.statement_depth += 1;
        walk::walk_statement(self, statement);
        self.statement_depth -= 1;
    }

    fn visit_function(&mut self, function: &oxc_ast::ast::Function<'a>, flags: ScopeFlags) {
        self.function_depth += 1;
        walk::walk_function(self, function, flags);
        self.function_depth -= 1;
    }

    fn visit_arrow_function_expression(
        &mut self,
        function: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        self.function_depth += 1;
        walk::walk_arrow_function_expression(self, function);
        self.function_depth -= 1;
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.function_depth == 0
            && call.type_arguments.is_none()
            && let Expression::Identifier(callee) = &call.callee
            && callee.reference_id.get().is_some_and(|reference| {
                self.scoping
                    .get_reference(reference)
                    .symbol_id()
                    .map_or(callee.name == "useTemplateRef", |symbol| {
                        self.imports.contains(&symbol)
                    })
            })
        {
            let literal = self
                .statement_opens_its_line()
                .then(|| {
                    let Argument::StringLiteral(key) = call.arguments.first()? else {
                        return None;
                    };
                    Some((
                        key.span.start as usize..key.span.end as usize,
                        String::from(key.value.as_str()),
                    ))
                })
                .flatten();
            self.calls.push(InferredCall {
                callee: callee.span.start as usize..callee.span.end as usize,
                name: String::from(
                    self.script
                        .get(callee.span.start as usize..callee.span.end as usize)
                        .unwrap_or(callee.name.as_str()),
                ),
                imported: callee.reference_id.get().is_some_and(|reference| {
                    self.scoping.get_reference(reference).symbol_id().is_some()
                }),
                statement_start: self.statement_start,
                literal,
            });
        }
        walk::walk_call_expression(self, call);
    }
}

#[cfg(test)]
mod tests;

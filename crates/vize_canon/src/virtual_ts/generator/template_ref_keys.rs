//! Owned, inferred `useTemplateRef('name')` calls and their template keys.
//!
//! An owned call is typed by indexing the template's ref registry with its
//! literal key, so a name the template never registers is an error on that
//! key. Vue's language tools do this only outside functions; Vize also types a
//! call inside a function of the component's script (a local composable, a
//! lifecycle hook), because it reads the same instance's refs. A missing key is
//! an error there only when setup runs that function (see `owners`). Only a call
//! inside an exported declaration keeps Vue's own signature: another module
//! can call it from a component with a different template.
//!
//! Only inferred callees use a private alias. The index access is placed in
//! front of the statement that holds it, on the generated line the statement
//! starts on: an authored `@ts-expect-error` above the statement covers it, and
//! the key maps back to its authored literal. The call itself may span lines
//! (`useTemplateRef(\n'key')`), so the line of the key is not a place a
//! statement can be inserted at; a statement that shares its first line with
//! other code is left unchecked for the same reason.

use oxc_ast::ast::{
    Argument, CallExpression, ExportNamedDeclaration, Expression, ImportDeclarationSpecifier,
    ImportOrExportKind, Program, Statement,
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
mod owners;

use owners::{Owner, Owners};

#[derive(Default)]
pub(super) struct TemplateRefUsage {
    calls: Vec<InferredCall>,
    helper: String,
}

#[derive(Default)]
pub(super) struct TemplateRefKeyChecks<'a> {
    calls: &'a [InferredCall],
    helper: &'a str,
    next: usize,
    tsx_arrows: &'a [super::script_module::tsx_arrows::ArrowComma],
}

struct InferredCall {
    callee: std::ops::Range<usize>,
    name: String,
    imported: bool,
    /// Where the top-level statement holding the call starts.
    statement_start: usize,
    owner: Owner,
    /// A key can be nonliteral; its call still needs the inferred signature.
    literal: Option<(std::ops::Range<usize>, String)>,
}

impl TemplateRefUsage {
    /// Reuse setup-helper parsing and resolved symbols. Typed calls and calls
    /// inside exported functions retain Vue's own signature and never read the
    /// registry.
    pub(super) fn collect(
        program: &Program<'_>,
        scoping: &Scoping,
        script: &str,
        ownership: crate::virtual_ts::TemplateRefOwnership,
    ) -> Self {
        let (imports, namespaces) = vue_imports(program, ownership.imports_alias);
        let mut collector = Collector {
            script,
            scoping,
            imports,
            namespaces,
            exported: exported_local_symbols(program, scoping),
            calls: Vec::new(),
            exported_function_depth: 0,
            export_depth: 0,
            statement_depth: 0,
            statement_start: 0,
            owners: Owners::default(),
            default_export_depth: 0,
        };
        collector.visit_program(program);
        let reached = collector.owners.reached_from_setup();
        for call in &mut collector.calls {
            if !call.owner.runs_in_instance(&reached) {
                call.literal = None;
            }
        }
        // An unresolved name can belong to an authored ambient declaration or
        // be missing entirely. Only explicit Vue-global ownership admits its
        // specialization; otherwise retain the original call and diagnostics.
        collector
            .calls
            .retain(|call| call.imported || ownership.global);
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

    /// A parse repaired around a syntax error can miss a call or a function
    /// setup runs. Its calls stay inferred; none of their keys is checked.
    pub(super) fn without_key_checks(mut self) -> Self {
        for call in &mut self.calls {
            call.literal = None;
        }
        self
    }

    pub(super) fn needs_registry(&self) -> bool {
        !self.calls.is_empty()
    }

    pub(super) fn helper_name(&self) -> &str {
        self.helper.as_str()
    }

    /// A rewritten call still uses its authored import. Keep only those
    /// imported bindings live for `noUnusedLocals`, never unrelated imports.
    pub(super) fn emit_import_anchors(&self, ts: &mut crate::virtual_ts::TextBuf) {
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
            tsx_arrows: &[],
        }
    }
}

impl TemplateRefKeyChecks<'_> {
    /// Emit the checks of every statement that starts on the line `[start, end)`.
    pub(super) fn emit_for_line(
        &mut self,
        ts: &mut crate::virtual_ts::TextBuf,
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
                ts.push_str("void (null! as $TemplateRefs[");
            } else {
                // A missing global produces the checker's error-type `any`.
                // Keep that on the object side of the conditional: using it
                // as the index would invent TS2538 beside the real TS2304.
                let record = crate::virtual_ts::generated_names::lib("Record");
                append!(
                    *ts,
                    "void (null! as ({}IsVue extends true ? $TemplateRefs : {record}<string, never>)[",
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

/// Value imports of Vue's own `useTemplateRef`, and namespace imports whose
/// `useTemplateRef` member is Vue's: from `vue`, or from `#imports` when the
/// project resolved that alias to a re-export of Vue's.
fn vue_imports(program: &Program<'_>, imports_alias: bool) -> (Vec<SymbolId>, Vec<SymbolId>) {
    let specifiers =
        || vue_import_specifiers(program, imports_alias).flat_map(|specifiers| specifiers.iter());
    let named = specifiers()
        .filter_map(|specifier| {
            let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier else {
                return None;
            };
            (named.import_kind != ImportOrExportKind::Type
                && named.imported.name() == "useTemplateRef")
                .then(|| named.local.symbol_id.get())
                .flatten()
        })
        .collect();
    let namespaces = specifiers()
        .filter_map(|specifier| {
            let ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace) = specifier else {
                return None;
            };
            namespace.local.symbol_id.get()
        })
        .collect();
    (named, namespaces)
}

/// Top-level declarations published by `export { name }` or
/// `export default name`, whose bodies other modules can call.
fn exported_local_symbols(program: &Program<'_>, scoping: &Scoping) -> Vec<SymbolId> {
    let referenced = |reference: &oxc_ast::ast::IdentifierReference<'_>| {
        reference
            .reference_id
            .get()
            .and_then(|reference| scoping.get_reference(reference).symbol_id())
    };
    let mut exported = Vec::new();
    for statement in &program.body {
        match statement {
            Statement::ExportNamedDeclaration(export) if export.source.is_none() => {
                for specifier in &export.specifiers {
                    if let oxc_ast::ast::ModuleExportName::IdentifierReference(local) =
                        &specifier.local
                        && let Some(symbol) = referenced(local)
                    {
                        exported.push(symbol);
                    }
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                if let oxc_ast::ast::ExportDefaultDeclarationKind::Identifier(local) =
                    &export.declaration
                    && let Some(symbol) = referenced(local)
                {
                    exported.push(symbol);
                }
            }
            _ => {}
        }
    }
    exported
}

fn vue_import_specifiers<'p, 'a>(
    program: &'p Program<'a>,
    imports_alias: bool,
) -> impl Iterator<Item = &'p oxc_allocator::Vec<'a, ImportDeclarationSpecifier<'a>>> {
    program.body.iter().filter_map(move |statement| {
        let Statement::ImportDeclaration(import) = statement else {
            return None;
        };
        (import.import_kind != ImportOrExportKind::Type
            && (import.source.value == "vue"
                || (imports_alias
                    && import.source.value
                        == crate::virtual_ts::TemplateRefOwnership::IMPORTS_ALIAS)))
            .then_some(import.specifiers.as_ref())
            .flatten()
    })
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
    ["", "Source", "IsVue", "Global"]
        .iter()
        .any(|suffix| name.strip_prefix(candidate) == Some(suffix))
}

struct Collector<'s> {
    script: &'s str,
    scoping: &'s Scoping,
    imports: Vec<SymbolId>,
    /// `import * as Vue from 'vue'`: `Vue.useTemplateRef` is Vue's.
    namespaces: Vec<SymbolId>,
    /// Local declarations an export list or `export default` publishes.
    exported: Vec<SymbolId>,
    calls: Vec<InferredCall>,
    /// Functions inside an exported declaration: other modules call them.
    exported_function_depth: usize,
    export_depth: usize,
    statement_depth: usize,
    /// Where the top-level statement being visited starts.
    statement_start: usize,
    owners: Owners,
    default_export_depth: usize,
}

impl Collector<'_> {
    fn publishes(&self, symbol: Option<SymbolId>) -> bool {
        symbol.is_some_and(|symbol| self.exported.contains(&symbol))
    }

    fn enter_function(&mut self) {
        if self.export_depth > 0 {
            self.exported_function_depth += 1;
        }
    }

    fn leave_function(&mut self) {
        if self.export_depth > 0 {
            self.exported_function_depth -= 1;
        }
    }

    /// The span of a callee that is Vue's `useTemplateRef`, and whether it
    /// resolves to an import: a named import, a member of a namespace import,
    /// or (not imported) the unbound global.
    fn owned_callee(&self, callee: &Expression<'_>) -> Option<(oxc_span::Span, bool)> {
        let symbol = |reference: &oxc_ast::ast::IdentifierReference<'_>| {
            reference
                .reference_id
                .get()
                .map(|reference| self.scoping.get_reference(reference).symbol_id())
        };
        match callee {
            Expression::Identifier(callee) => match symbol(callee)? {
                Some(symbol) => self
                    .imports
                    .contains(&symbol)
                    .then_some((callee.span, true)),
                None => (callee.name == "useTemplateRef").then_some((callee.span, false)),
            },
            Expression::StaticMemberExpression(member)
                if member.property.name == "useTemplateRef" && !member.optional =>
            {
                let Expression::Identifier(object) = &member.object else {
                    return None;
                };
                let namespace = symbol(object)??;
                self.namespaces
                    .contains(&namespace)
                    .then_some((member.span, true))
            }
            _ => None,
        }
    }

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
    fn visit_export_named_declaration(&mut self, export: &ExportNamedDeclaration<'a>) {
        self.export_depth += 1;
        walk::walk_export_named_declaration(self, export);
        self.export_depth -= 1;
    }

    fn visit_export_default_declaration(
        &mut self,
        export: &oxc_ast::ast::ExportDefaultDeclaration<'a>,
    ) {
        self.default_export_depth += 1;
        walk::walk_export_default_declaration(self, export);
        self.default_export_depth -= 1;
    }

    fn visit_object_property(&mut self, property: &oxc_ast::ast::ObjectProperty<'a>) {
        if self.default_export_depth > 0 {
            self.owners.observe_default_export_property(property);
        }
        walk::walk_object_property(self, property);
    }

    fn visit_statement(&mut self, statement: &Statement<'a>) {
        if self.statement_depth == 0 {
            self.statement_start = statement.span().start as usize;
        }
        self.statement_depth += 1;
        walk::walk_statement(self, statement);
        self.statement_depth -= 1;
    }

    fn visit_function(&mut self, function: &oxc_ast::ast::Function<'a>, flags: ScopeFlags) {
        let published = self.publishes(function.id.as_ref().and_then(|id| id.symbol_id.get()));
        self.export_depth += usize::from(published);
        self.enter_function();
        let id = function.id.as_ref().and_then(|id| id.symbol_id.get());
        self.owners.enter(function.span.start, id);
        walk::walk_function(self, function, flags);
        self.owners.leave();
        self.leave_function();
        self.export_depth -= usize::from(published);
    }

    fn visit_variable_declarator(&mut self, declarator: &oxc_ast::ast::VariableDeclarator<'a>) {
        let published = self.publishes(match &declarator.id {
            oxc_ast::ast::BindingPattern::BindingIdentifier(id) => id.symbol_id.get(),
            _ => None,
        });
        self.export_depth += usize::from(published);
        self.owners.name_initializer(declarator);
        walk::walk_variable_declarator(self, declarator);
        self.export_depth -= usize::from(published);
    }

    fn visit_arrow_function_expression(
        &mut self,
        function: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        self.enter_function();
        self.owners.enter(function.span.start, None);
        walk::walk_arrow_function_expression(self, function);
        self.owners.leave();
        self.leave_function();
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.owners.observe_call(call, self.scoping);
        if self.exported_function_depth == 0
            && call.type_arguments.is_none()
            && let Some((callee, imported)) = self.owned_callee(&call.callee)
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
            let range = callee.start as usize..callee.end as usize;
            let Some(name) = self.script.get(range.clone()) else {
                walk::walk_call_expression(self, call);
                return;
            };
            self.calls.push(InferredCall {
                callee: range,
                name: String::from(name),
                imported,
                statement_start: self.statement_start,
                owner: self.owners.current(),
                literal,
            });
        }
        walk::walk_call_expression(self, call);
    }
}

#[cfg(test)]
mod tests;

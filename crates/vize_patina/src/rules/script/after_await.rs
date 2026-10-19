//! Shared engine for the `*-after-await` rules.
//!
//! Vue binds lifecycle hooks, watchers and `expose` to the component instance
//! that is current while `setup()` runs. After an `await`, `setup()` resumes in
//! a microtask with no current instance, so these calls silently attach to
//! nothing. Like eslint-plugin-vue, the rules check the `setup()` option of a
//! component (`export default {}` in an SFC, or any `defineComponent({})`
//! call). Only `await` expressions and calls made by the setup function itself
//! count: nested functions run later and are skipped. `<script setup>` restores
//! the instance after `await` for every call except `defineExpose`, which the
//! expose rule checks as a top-level statement.

use oxc_ast::ast::{
    Argument, ArrayExpression, ArrayExpressionElement, AssignmentExpression, AwaitExpression,
    BindingPattern, CallExpression, ChainElement, Class, ExportDefaultDeclarationKind, Expression,
    FormalParameters, Function, FunctionBody, ImportDeclarationSpecifier, ObjectExpression,
    ObjectProperty, ObjectPropertyKind, Program, PropertyKey, Statement, VariableDeclarator,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_array_expression, walk_assignment_expression, walk_await_expression,
        walk_call_expression, walk_object_property, walk_variable_declarator,
    },
};
use oxc_span::{GetSpan, Span};
use oxc_syntax::scope::ScopeFlags;
use vize_l0::{CompactString, FxHashSet};

use super::SfcScriptContext;

/// Modules whose Composition API exports the rules track, as in
/// eslint-plugin-vue.
const VUE_MODULES: [&str; 3] = ["vue", "@vue/composition-api", "@vue/runtime-core"];

/// Lifecycle hooks that must be registered synchronously.
pub(super) const LIFECYCLE_HOOKS: [&str; 11] = [
    "onBeforeMount",
    "onBeforeUnmount",
    "onBeforeUpdate",
    "onErrorCaptured",
    "onMounted",
    "onRenderTracked",
    "onRenderTriggered",
    "onUnmounted",
    "onUpdated",
    "onActivated",
    "onDeactivated",
];

/// Watch APIs whose stop handle is lost when registered after `await`.
pub(super) const WATCH_APIS: [&str; 2] = ["watch", "watchEffect"];

/// What a rule looks for after `await`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Target {
    Lifecycle,
    Watch,
    Expose,
}

/// A call made after `await`, with the name the message uses.
pub(super) struct Finding {
    pub(super) span: Span,
    pub(super) name: CompactString,
}

/// Every call `target` forbids after an `await` in `program`.
pub(super) fn find(
    program: &Program<'_>,
    sfc: SfcScriptContext<'_>,
    target: Target,
) -> Vec<Finding> {
    let imports = Imports::collect(program, target);
    let mut findings = Vec::new();
    if target == Target::Expose && sfc.is_script_setup {
        script_setup_expose(program, &mut findings);
    }
    if target != Target::Expose && imports.is_empty() {
        return findings;
    }
    let mut components = Components {
        target,
        imports: &imports,
        seen: FxHashSet::default(),
        findings,
    };
    if sfc.is_sfc && !sfc.is_script_setup {
        components.collect_export_default(program);
    }
    components.visit_program(program);
    let mut findings = components.findings;
    findings.sort_by_key(|finding| finding.span.start);
    findings
}

/// Local names bound to the tracked Vue exports.
#[derive(Default)]
struct Imports {
    /// Local name of a tracked named import.
    named: FxHashSet<CompactString>,
    /// Local names of `import * as Vue from "vue"`.
    namespaces: FxHashSet<CompactString>,
    /// The export names `named` and namespace members resolve against.
    exports: &'static [&'static str],
}

impl Imports {
    fn collect(program: &Program<'_>, target: Target) -> Self {
        let exports: &'static [&'static str] = match target {
            Target::Lifecycle => &LIFECYCLE_HOOKS,
            Target::Watch => &WATCH_APIS,
            Target::Expose => &[],
        };
        let mut imports = Self {
            exports,
            ..Self::default()
        };
        if exports.is_empty() {
            return imports;
        }
        for statement in &program.body {
            let Statement::ImportDeclaration(declaration) = statement else {
                continue;
            };
            if declaration.import_kind.is_type()
                || !VUE_MODULES.contains(&declaration.source.value.as_str())
            {
                continue;
            }
            for specifier in declaration.specifiers.iter().flatten() {
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier)
                        if !specifier.import_kind.is_type()
                            && exports.contains(&specifier.imported.name().as_str()) =>
                    {
                        imports
                            .named
                            .insert(CompactString::new(specifier.local.name.as_str()));
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        imports
                            .namespaces
                            .insert(CompactString::new(specifier.local.name.as_str()));
                    }
                    _ => {}
                }
            }
        }
        imports
    }

    fn is_empty(&self) -> bool {
        self.named.is_empty() && self.namespaces.is_empty()
    }

    /// Whether `callee` names a tracked export: `onMounted` or `Vue.onMounted`.
    fn resolves(&self, callee: &Expression<'_>) -> bool {
        match callee {
            Expression::Identifier(identifier) => self.named.contains(identifier.name.as_str()),
            Expression::StaticMemberExpression(member) => {
                matches!(&member.object, Expression::Identifier(object)
                    if self.namespaces.contains(object.name.as_str()))
                    && self.exports.contains(&member.property.name.as_str())
            }
            _ => false,
        }
    }
}

/// The ways `setup(props, context)` can reach `expose`.
#[derive(Default)]
struct ExposeBindings {
    /// `context` in `context.expose(...)`.
    context: Option<CompactString>,
    /// `e` in `setup(props, { expose: e })`.
    expose: Option<CompactString>,
}

fn expose_bindings(params: &FormalParameters<'_>) -> ExposeBindings {
    let mut bindings = ExposeBindings::default();
    let Some(context) = params.items.get(1) else {
        return bindings;
    };
    match &context.pattern {
        BindingPattern::BindingIdentifier(identifier) => {
            bindings.context = Some(CompactString::new(identifier.name.as_str()));
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                if property.computed || property_key_name(&property.key) != Some("expose") {
                    continue;
                }
                if let BindingPattern::BindingIdentifier(identifier) = &property.value {
                    bindings.expose = Some(CompactString::new(identifier.name.as_str()));
                }
            }
        }
        BindingPattern::AssignmentPattern(assignment) => {
            if let BindingPattern::BindingIdentifier(identifier) = &assignment.left {
                bindings.context = Some(CompactString::new(identifier.name.as_str()));
            }
        }
        BindingPattern::ArrayPattern(_) => {}
    }
    bindings
}

/// Finds the `setup` option of every component in a program and scans it.
struct Components<'i> {
    target: Target,
    imports: &'i Imports,
    seen: FxHashSet<u32>,
    findings: Vec<Finding>,
}

impl Components<'_> {
    /// `export default {}` in an SFC.
    fn collect_export_default(&mut self, program: &Program<'_>) {
        for statement in &program.body {
            let Statement::ExportDefaultDeclaration(export) = statement else {
                continue;
            };
            let options = match &export.declaration {
                ExportDefaultDeclarationKind::ObjectExpression(object) => Some(&**object),
                ExportDefaultDeclarationKind::TSAsExpression(expression) => {
                    object_of(&expression.expression)
                }
                ExportDefaultDeclarationKind::TSSatisfiesExpression(expression) => {
                    object_of(&expression.expression)
                }
                ExportDefaultDeclarationKind::ParenthesizedExpression(expression) => {
                    object_of(&expression.expression)
                }
                _ => None,
            };
            if let Some(options) = options {
                self.scan_options(options);
            }
        }
    }

    fn scan_options(&mut self, options: &ObjectExpression<'_>) {
        if !self.seen.insert(options.span.start) {
            return;
        }
        for property in &options.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            if property.computed || property_key_name(&property.key) != Some("setup") {
                continue;
            }
            match &property.value {
                Expression::FunctionExpression(function) => {
                    if let Some(body) = &function.body {
                        self.scan_setup(&function.params, body);
                    }
                }
                Expression::ArrowFunctionExpression(arrow) => {
                    self.scan_setup(&arrow.params, &arrow.body);
                }
                _ => {}
            }
        }
    }

    fn scan_setup(&mut self, params: &FormalParameters<'_>, body: &FunctionBody<'_>) {
        let mut scan = Scan::new(self.target, self.imports, expose_bindings(params));
        for statement in &body.statements {
            scan.visit_statement(statement);
        }
        self.findings.append(&mut scan.findings);
    }
}

impl<'a> Visit<'a> for Components<'_> {
    // `defineComponent({ setup })` anywhere, including in plain modules.
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &it.callee
            && matches!(
                callee.name.as_str(),
                "defineComponent" | "defineNuxtComponent"
            )
            && let Some(Argument::ObjectExpression(options)) = it.arguments.first()
        {
            self.scan_options(options);
        }
        walk_call_expression(self, it);
    }
}

/// Reports forbidden calls made by one setup scope after its first `await`.
struct Scan<'i> {
    target: Target,
    imports: &'i Imports,
    expose: ExposeBindings,
    after_await: bool,
    /// Calls whose return value is kept (a watch stop handle that may be used).
    kept: FxHashSet<u32>,
    findings: Vec<Finding>,
}

impl<'i> Scan<'i> {
    fn new(target: Target, imports: &'i Imports, expose: ExposeBindings) -> Self {
        Self {
            target,
            imports,
            expose,
            after_await: false,
            kept: FxHashSet::default(),
            findings: Vec::new(),
        }
    }

    fn keep(&mut self, expression: &Expression<'_>) {
        match expression {
            Expression::CallExpression(call) => {
                self.kept.insert(call.span.start);
            }
            Expression::ChainExpression(chain) => {
                if let ChainElement::CallExpression(call) = &chain.expression {
                    self.kept.insert(call.span.start);
                }
            }
            _ => {}
        }
    }

    fn check(&mut self, call: &CallExpression<'_>) {
        let name = match self.target {
            Target::Lifecycle => (self.imports.resolves(&call.callee) && call.arguments.len() < 2)
                .then(|| CompactString::new("lifecycle")),
            Target::Watch => (self.imports.resolves(&call.callee)
                && !self.kept.contains(&call.span.start))
            .then(|| CompactString::new("watch")),
            Target::Expose => self.expose_name(&call.callee),
        };
        if let Some(name) = name {
            self.findings.push(Finding {
                span: call.span,
                name,
            });
        }
    }

    fn expose_name(&self, callee: &Expression<'_>) -> Option<CompactString> {
        match callee {
            Expression::Identifier(identifier)
                if self.expose.expose.as_deref() == Some(identifier.name.as_str()) =>
            {
                Some(CompactString::new(identifier.name.as_str()))
            }
            Expression::StaticMemberExpression(member)
                if member.property.name == "expose"
                    && matches!(&member.object, Expression::Identifier(object)
                        if self.expose.context.as_deref() == Some(object.name.as_str())) =>
            {
                Some(CompactString::new("expose"))
            }
            _ => None,
        }
    }
}

impl<'a> Visit<'a> for Scan<'_> {
    // Nested functions and classes run later, outside this setup scope.
    fn visit_function(&mut self, _it: &Function<'a>, _flags: ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _it: &oxc_ast::ast::ArrowFunctionExpression<'a>) {
    }

    fn visit_class(&mut self, _it: &Class<'a>) {}

    fn visit_await_expression(&mut self, it: &AwaitExpression<'a>) {
        self.after_await = true;
        walk_await_expression(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if self.after_await {
            self.check(it);
        }
        for argument in &it.arguments {
            if let Some(expression) = argument.as_expression() {
                self.keep(expression);
            }
        }
        walk_call_expression(self, it);
    }

    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        if let Some(init) = &it.init {
            self.keep(init);
        }
        walk_variable_declarator(self, it);
    }

    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        self.keep(&it.right);
        walk_assignment_expression(self, it);
    }

    fn visit_object_property(&mut self, it: &ObjectProperty<'a>) {
        self.keep(&it.value);
        walk_object_property(self, it);
    }

    fn visit_array_expression(&mut self, it: &ArrayExpression<'a>) {
        for element in &it.elements {
            if let ArrayExpressionElement::SpreadElement(_) | ArrayExpressionElement::Elision(_) =
                element
            {
                continue;
            }
            if let Some(expression) = element.as_expression() {
                self.keep(expression);
            }
        }
        walk_array_expression(self, it);
    }
}

/// `<script setup>`: a top-level `defineExpose(...)` statement after a
/// top-level `await` (awaits in nested functions do not count).
fn script_setup_expose(program: &Program<'_>, findings: &mut Vec<Finding>) {
    let imports = Imports::default();
    let mut scan = Scan::new(Target::Expose, &imports, ExposeBindings::default());
    for statement in &program.body {
        if scan.after_await
            && let Statement::ExpressionStatement(statement) = statement
            && let Expression::CallExpression(call) = &statement.expression
            && matches!(&call.callee, Expression::Identifier(callee) if callee.name == "defineExpose")
        {
            findings.push(Finding {
                span: call.span(),
                name: CompactString::new("defineExpose"),
            });
        }
        scan.visit_statement(statement);
    }
}

fn object_of<'s, 'a>(expression: &'s Expression<'a>) -> Option<&'s ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::ParenthesizedExpression(expression) => object_of(&expression.expression),
        Expression::TSAsExpression(expression) => object_of(&expression.expression),
        Expression::TSSatisfiesExpression(expression) => object_of(&expression.expression),
        _ => None,
    }
}

fn property_key_name<'k>(key: &'k PropertyKey<'_>) -> Option<&'k str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

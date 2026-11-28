//! Which code a component instance runs, for `useTemplateRef` calls.
//!
//! The ownership model follows the virtual TypeScript generator's template
//! ref owners: the setup body runs in the component's instance, and so does a
//! callback it hands straight to a call (`onMounted(() => ...)`) and a local
//! function it calls, directly or through other local functions. The rule
//! adds the owners that provably run without one:
//!
//! - the top level of a module (a plain `<script>`, a `.ts` file);
//! - a callback passed straight to an API that always calls it later: a timer,
//!   a `watch` callback, `nextTick`, a promise reaction, a DOM listener;
//! - a local function setup never calls, whose every use is one of those
//!   callbacks, a call from one, or a template event handler.
//!
//! Anything else (a function setup passes to an unknown call, returns,
//! stores or exports, an object method, a class) has an owner the script
//! cannot pin down, and a call there is never reported.

use oxc_ast::ast::{
    Argument, ArrowFunctionExpression, BindingPattern, CallExpression, Class,
    ExportDefaultDeclarationKind, Expression, Function, IdentifierReference,
    ImportDeclarationSpecifier, ObjectExpression, ObjectPropertyKind, Program, Statement,
    VariableDeclarator,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_arrow_function_expression, walk_call_expression, walk_function,
        walk_variable_declarator,
    },
};
use oxc_semantic::Scoping;
use oxc_span::Span;
use oxc_syntax::{reference::ReferenceId, scope::ScopeFlags, symbol::SymbolId};
use vize_l0::{FxHashMap, FxHashSet};

/// The API a deferred callback is passed to, as the label names it.
pub(super) type DeferredApi = &'static str;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Owner {
    /// Module evaluation: no component is being set up.
    Module,
    /// The setup body, a callback it passes straight to a call, or a
    /// component's `setup` function.
    Setup,
    /// A callback an API always runs after setup has returned.
    Deferred(DeferredApi),
    /// The local function setup declares under this binding.
    Local(SymbolId),
    /// Code whose caller the script cannot determine.
    Unknown,
}

/// A `useTemplateRef(...)` call and the code that runs it.
pub(super) struct Site {
    pub(super) span: Span,
    pub(super) owner: Owner,
}

/// Global timers whose callback always runs in a later task.
const TIMERS: [&str; 6] = [
    "setTimeout",
    "setInterval",
    "setImmediate",
    "requestAnimationFrame",
    "requestIdleCallback",
    "queueMicrotask",
];

/// Receivers a timer is commonly called through.
const GLOBAL_OBJECTS: [&str; 3] = ["window", "globalThis", "self"];

/// Vue exports the walk resolves.
const TEMPLATE_REF: &str = "useTemplateRef";
const VUE_EXPORTS: [&str; 3] = [TEMPLATE_REF, "watch", "nextTick"];

pub(super) struct Walk<'s> {
    scoping: &'s Scoping,
    /// Local Vue imports: the symbol and the export it names.
    vue_named: FxHashMap<SymbolId, &'static str>,
    /// `import * as Vue from 'vue'` symbols.
    vue_namespaces: FxHashSet<SymbolId>,
    /// Unbound names that do not reach the global Vue API, because the other
    /// script block of the SFC declares them.
    foreign_globals: FxHashSet<&'static str>,
    current: Owner,
    stack: Vec<Owner>,
    /// Owners decided by the enclosing call or object for the function that
    /// starts at this offset.
    planned: FxHashMap<u32, Owner>,
    /// `const name = () => ...` in setup: the function start and the binding.
    named: Option<(u32, SymbolId)>,
    /// References already classified by their parent node.
    classified: FxHashSet<ReferenceId>,
    pub(super) sites: Vec<Site>,
    /// Local functions setup declares.
    pub(super) locals: Vec<SymbolId>,
    /// Calls of a local function, with the owner that makes them.
    pub(super) edges: Vec<(Owner, SymbolId)>,
    /// Local functions passed straight to a deferred API.
    pub(super) deferred_passes: FxHashSet<SymbolId>,
    /// Symbols read in any other way: returned, stored, passed on.
    pub(super) escapes: FxHashSet<SymbolId>,
}

impl<'s> Walk<'s> {
    pub(super) fn new(scoping: &'s Scoping, top: Owner) -> Self {
        Self {
            scoping,
            vue_named: FxHashMap::default(),
            vue_namespaces: FxHashSet::default(),
            foreign_globals: FxHashSet::default(),
            current: top,
            stack: Vec::new(),
            planned: FxHashMap::default(),
            named: None,
            classified: FxHashSet::default(),
            sites: Vec::new(),
            locals: Vec::new(),
            edges: Vec::new(),
            deferred_passes: FxHashSet::default(),
            escapes: FxHashSet::default(),
        }
    }

    /// Record the Vue imports of `program` and, for an SFC with two script
    /// blocks, the names the other block binds otherwise.
    pub(super) fn collect_imports(&mut self, program: &Program<'_>, sibling: Option<&Program<'_>>) {
        for statement in &program.body {
            let Statement::ImportDeclaration(declaration) = statement else {
                continue;
            };
            if declaration.import_kind.is_type() || !is_vue_module(&declaration.source.value) {
                continue;
            }
            for specifier in declaration.specifiers.iter().flatten() {
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier)
                        if !specifier.import_kind.is_type() =>
                    {
                        let imported = specifier.imported.name();
                        if let Some(export) = VUE_EXPORTS.iter().find(|name| **name == imported)
                            && let Some(symbol) = specifier.local.symbol_id.get()
                        {
                            self.vue_named.insert(symbol, export);
                        }
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        if let Some(symbol) = specifier.local.symbol_id.get() {
                            self.vue_namespaces.insert(symbol);
                        }
                    }
                    _ => {}
                }
            }
        }
        if let Some(sibling) = sibling {
            for name in VUE_EXPORTS {
                if super::sibling::binds_non_vue(sibling, name) {
                    self.foreign_globals.insert(name);
                }
            }
        }
    }

    /// Plan the `setup` of an `export default {}` component.
    pub(super) fn plan_export_default(&mut self, program: &Program<'_>) {
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
                self.plan_setup_option(options);
            }
        }
    }

    fn plan_setup_option(&mut self, options: &ObjectExpression<'_>) {
        for property in &options.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            if property.computed || property.key.static_name().as_deref() != Some("setup") {
                continue;
            }
            if let Some(start) = function_start(&property.value) {
                self.planned.insert(start, Owner::Setup);
            }
        }
    }

    pub(super) fn symbol_name(&self, symbol: SymbolId) -> &'s str {
        self.scoping.symbol_name(symbol)
    }

    fn symbol_of(&self, identifier: &IdentifierReference<'_>) -> Option<SymbolId> {
        let reference = identifier.reference_id.get()?;
        self.scoping.get_reference(reference).symbol_id()
    }

    fn is_unbound(&self, identifier: &IdentifierReference<'_>) -> bool {
        identifier
            .reference_id
            .get()
            .is_some_and(|reference| self.scoping.get_reference(reference).symbol_id().is_none())
    }

    /// The Vue export `callee` names: a Vue import under any local name, a
    /// member of a Vue namespace, or the unbound auto-imported global.
    fn vue_export(&self, callee: &Expression<'_>) -> Option<&'static str> {
        match callee {
            Expression::Identifier(identifier) => match self.symbol_of(identifier) {
                Some(symbol) => self.vue_named.get(&symbol).copied(),
                None if self.is_unbound(identifier) => VUE_EXPORTS
                    .iter()
                    .find(|name| **name == identifier.name.as_str())
                    .copied()
                    .filter(|name| !self.foreign_globals.contains(name)),
                None => None,
            },
            Expression::StaticMemberExpression(member) => match &member.object {
                Expression::Identifier(object)
                    if self
                        .symbol_of(object)
                        .is_some_and(|symbol| self.vue_namespaces.contains(&symbol)) =>
                {
                    VUE_EXPORTS
                        .iter()
                        .find(|name| **name == member.property.name.as_str())
                        .copied()
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// The API and the argument positions of a call that always runs its
    /// callbacks after the current task: `None` for any other call.
    fn deferred_api(&self, call: &CallExpression<'_>) -> Option<(DeferredApi, ArgumentSlots)> {
        match &call.callee {
            Expression::Identifier(identifier) => {
                if self.is_unbound(identifier)
                    && let Some(timer) = TIMERS.iter().find(|name| **name == identifier.name)
                {
                    return Some((timer, ArgumentSlots::First));
                }
            }
            Expression::StaticMemberExpression(member) => {
                let property = member.property.name.as_str();
                if let Expression::Identifier(object) = &member.object
                    && self.is_unbound(object)
                    && GLOBAL_OBJECTS.contains(&object.name.as_str())
                    && let Some(timer) = TIMERS.iter().find(|name| **name == property)
                {
                    return Some((timer, ArgumentSlots::First));
                }
                match property {
                    "then" => return Some(("then", ArgumentSlots::All)),
                    "catch" => return Some(("catch", ArgumentSlots::All)),
                    "finally" => return Some(("finally", ArgumentSlots::All)),
                    "addEventListener" => {
                        return Some(("addEventListener", ArgumentSlots::Second));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        match self.vue_export(&call.callee)? {
            "watch" if runs_callback_later(call) => Some(("watch", ArgumentSlots::Second)),
            "nextTick" => Some(("nextTick", ArgumentSlots::First)),
            _ => None,
        }
    }

    fn enter(&mut self, start: u32, declared: Option<SymbolId>) {
        let owner = if let Some(planned) = self.planned.get(&start) {
            *planned
        } else {
            match self.current {
                Owner::Setup => {
                    match declared.or_else(|| {
                        self.named
                            .filter(|(at, _)| *at == start)
                            .map(|(_, symbol)| symbol)
                    }) {
                        Some(symbol) => {
                            self.locals.push(symbol);
                            Owner::Local(symbol)
                        }
                        None => Owner::Unknown,
                    }
                }
                Owner::Module | Owner::Unknown => Owner::Unknown,
                inherited @ (Owner::Deferred(_) | Owner::Local(_)) => inherited,
            }
        };
        self.stack.push(self.current);
        self.current = owner;
    }

    fn leave(&mut self) {
        self.current = self.stack.pop().unwrap_or(Owner::Unknown);
    }

    fn classify_call(&mut self, call: &CallExpression<'_>) {
        if self.vue_export(&call.callee) == Some(TEMPLATE_REF) {
            self.sites.push(Site {
                span: call.span,
                owner: self.current,
            });
        }
        if let Expression::Identifier(callee) = &call.callee
            && let Some(symbol) = self.symbol_of(callee)
            && let Some(reference) = callee.reference_id.get()
        {
            self.classified.insert(reference);
            self.edges.push((self.current, symbol));
        }
        if defines_component(&call.callee) {
            match call.arguments.first() {
                Some(Argument::ObjectExpression(options)) => self.plan_setup_option(options),
                Some(argument) => {
                    if let Some(start) = argument.as_expression().and_then(function_start) {
                        self.planned.insert(start, Owner::Setup);
                    }
                }
                None => {}
            }
        }
        let deferred = self.deferred_api(call);
        for (index, argument) in call.arguments.iter().enumerate() {
            let Some(expression) = argument.as_expression() else {
                continue;
            };
            let deferred = deferred
                .filter(|(_, slots)| slots.contains(index))
                .map(|(api, _)| api);
            if let Some(start) = function_start(expression) {
                if self.planned.contains_key(&start) {
                    continue;
                }
                if let Some(api) = deferred {
                    self.planned.insert(start, Owner::Deferred(api));
                } else if self.current == Owner::Setup {
                    self.planned.insert(start, Owner::Setup);
                }
            } else if deferred.is_some()
                && let Expression::Identifier(identifier) = expression.without_parentheses()
                && let Some(symbol) = self.symbol_of(identifier)
                && let Some(reference) = identifier.reference_id.get()
            {
                self.classified.insert(reference);
                self.deferred_passes.insert(symbol);
            }
        }
    }
}

/// Which arguments of a deferred API are its callbacks.
#[derive(Clone, Copy)]
enum ArgumentSlots {
    First,
    Second,
    All,
}

impl ArgumentSlots {
    fn contains(self, index: usize) -> bool {
        match self {
            Self::First => index == 0,
            Self::Second => index == 1,
            Self::All => true,
        }
    }
}

impl<'a> Visit<'a> for Walk<'_> {
    fn visit_function(&mut self, it: &Function<'a>, flags: ScopeFlags) {
        let declared = it
            .is_declaration()
            .then(|| it.id.as_ref().and_then(|id| id.symbol_id.get()))
            .flatten();
        self.enter(it.span.start, declared);
        walk_function(self, it, flags);
        self.leave();
    }

    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'a>) {
        self.enter(it.span.start, None);
        walk_arrow_function_expression(self, it);
        self.leave();
    }

    // Class members run whenever their instance or class is used.
    fn visit_class(&mut self, _it: &Class<'a>) {}

    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        if self.current == Owner::Setup
            && let BindingPattern::BindingIdentifier(id) = &it.id
            && let Some(start) = it.init.as_ref().and_then(function_start)
            && let Some(symbol) = id.symbol_id.get()
        {
            self.named = Some((start, symbol));
        }
        walk_variable_declarator(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        self.classify_call(it);
        walk_call_expression(self, it);
    }

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if let Some(reference) = it.reference_id.get()
            && !self.classified.contains(&reference)
            && let Some(symbol) = self.scoping.get_reference(reference).symbol_id()
        {
            self.escapes.insert(symbol);
        }
    }
}

pub(super) fn is_vue_module(source: &str) -> bool {
    source == "@vue/runtime-core"
        || super::super::composition_api::is_composition_api_module(source)
}

/// Whether a `watch(source, callback, options)` call first runs its callback
/// after setup. An `immediate` option, or options the script cannot read,
/// can run it synchronously while setup still owns the instance.
fn runs_callback_later(call: &CallExpression<'_>) -> bool {
    match call.arguments.get(2) {
        None => true,
        Some(Argument::ObjectExpression(options)) => options.properties.iter().all(|property| {
            matches!(property, ObjectPropertyKind::ObjectProperty(property)
                if property.key.static_name().is_some_and(|key| key != "immediate"))
        }),
        Some(_) => false,
    }
}

/// `defineComponent(...)`, `defineNuxtComponent(...)` or
/// `Vue.defineComponent(...)`: the argument's `setup`, or the function form
/// itself, runs in the new component's instance.
fn defines_component(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::Identifier(callee) => {
            matches!(
                callee.name.as_str(),
                "defineComponent" | "defineNuxtComponent"
            )
        }
        Expression::StaticMemberExpression(member) => member.property.name == "defineComponent",
        _ => false,
    }
}

fn object_of<'e, 'a>(expression: &'e Expression<'a>) -> Option<&'e ObjectExpression<'a>> {
    match expression.without_parentheses() {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

/// The start of a function expression, the key the walk plans owners by.
fn function_start(expression: &Expression<'_>) -> Option<u32> {
    match expression.without_parentheses() {
        Expression::ArrowFunctionExpression(function) => Some(function.span.start),
        Expression::FunctionExpression(function) => Some(function.span.start),
        _ => None,
    }
}

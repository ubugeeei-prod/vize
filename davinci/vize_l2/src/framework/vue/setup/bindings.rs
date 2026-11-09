//! How a template reads each top-level `<script setup>` declaration.
//!
//! The classification is the compiler's current one: the template lanes
//! receive it as binding metadata, and the compiler snapshots pin it. It
//! differs from Vue's `walkDeclaration` (#12404): named imports from `vue`
//! may hold refs here, the ref family needs no `vue` import, `let x =
//! reactive()` stays reactive, `readonly`/`toRefs` results are classified,
//! every literal `const` is a literal constant, `declare const` counts, and a
//! destructure of a TypeScript-wrapped `defineProps` call is an ordinary one.

use super::super::{MacroKind, lookup};
use super::props::{self, PropsDestructure};
use alloc::vec::Vec;
use oxc_ast::ast::{
    BindingPattern, CallExpression, Expression, ImportDeclaration, ImportDeclarationSpecifier,
    Statement, VariableDeclarationKind, VariableDeclarator,
};
use vize_l0::Span;

/// How a template reads a `<script setup>` binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupBindingKind {
    /// Never a ref: functions, classes, enums, objects, arrays, macros.
    SetupConst,
    /// `let`/`var`: may be reassigned.
    SetupLet,
    /// A ref, read through `.value`.
    SetupRef,
    /// Possibly a ref, read through `unref`.
    SetupMaybeRef,
    /// A reactive object; never reassigned.
    SetupReactiveConst,
    /// A `const` initialized by a literal.
    LiteralConst,
    /// A destructured prop whose local has the prop's name.
    Props,
    /// A destructured prop read through a renamed local.
    PropsAliased,
}

/// One binding introduced by a top-level identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetupBinding {
    /// The declared identifier.
    pub name: Span,
    pub kind: SetupBindingKind,
}

/// Calls whose result kind the compiler knows, by Vue API name.
const REACTIVE_CALLS: &[(&str, SetupBindingKind)] = &[
    ("ref", SetupBindingKind::SetupRef),
    ("shallowRef", SetupBindingKind::SetupRef),
    ("computed", SetupBindingKind::SetupRef),
    ("toRef", SetupBindingKind::SetupRef),
    ("toRefs", SetupBindingKind::SetupRef),
    ("customRef", SetupBindingKind::SetupRef),
    ("useTemplateRef", SetupBindingKind::SetupRef),
    ("reactive", SetupBindingKind::SetupReactiveConst),
    ("shallowReactive", SetupBindingKind::SetupReactiveConst),
    ("readonly", SetupBindingKind::SetupReactiveConst),
    ("shallowReadonly", SetupBindingKind::SetupReactiveConst),
];

fn reactive_call(name: &str) -> Option<(&'static str, SetupBindingKind)> {
    REACTIVE_CALLS
        .iter()
        .find(|(api, _)| *api == name)
        .map(|(api, kind)| (*api, *kind))
}

/// Binding rows plus the local names that alias a Vue reactivity API.
#[derive(Debug, Default)]
pub(super) struct Bindings<'a> {
    pub(super) rows: Vec<SetupBinding>,
    pub(super) props: Option<PropsDestructure>,
    /// `defineProps` destructures the facts do not cover.
    pub(super) unclassified: Vec<Span>,
    aliases: Vec<(&'a str, &'static str)>,
    pub(super) overflow: bool,
}

impl<'a> Bindings<'a> {
    fn authored(&mut self, base: u32, span: oxc_span::Span) -> Span {
        match (base.checked_add(span.start), base.checked_add(span.end)) {
            (Some(start), Some(end)) => Span::new(start, end),
            _ => {
                self.overflow = true;
                Span::new(0, 0)
            }
        }
    }

    fn push(&mut self, base: u32, name: oxc_span::Span, kind: SetupBindingKind) {
        let name = self.authored(base, name);
        self.rows.push(SetupBinding { name, kind });
    }

    pub(super) fn statement(&mut self, base: u32, statement: &Statement<'a>) {
        match statement {
            Statement::ImportDeclaration(import) if !import.import_kind.is_type() => {
                self.import(base, import);
            }
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    self.declarator(base, declaration.kind, declarator);
                }
            }
            Statement::FunctionDeclaration(function) => {
                if let Some(id) = &function.id {
                    self.push(base, id.span, SetupBindingKind::SetupConst);
                }
            }
            Statement::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    self.push(base, id.span, SetupBindingKind::SetupConst);
                }
            }
            Statement::TSEnumDeclaration(enumeration) if !enumeration.declare => {
                self.push(base, enumeration.id.span, SetupBindingKind::SetupConst);
            }
            _ => {}
        }
    }

    /// A named import may hold a ref; a default or namespace import does not.
    fn import(&mut self, base: u32, import: &ImportDeclaration<'a>) {
        let from_vue = import.source.value == "vue";
        for specifier in import.specifiers.iter().flatten() {
            match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(named) => {
                    if named.import_kind.is_type() {
                        continue;
                    }
                    let local = named.local.name.as_str();
                    let imported = named.imported.name();
                    if from_vue
                        && imported != local
                        && let Some((api, _)) = reactive_call(imported.as_str())
                    {
                        self.aliases.push((local, api));
                    }
                    self.push(base, named.local.span, SetupBindingKind::SetupMaybeRef);
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                    self.push(base, default.local.span, SetupBindingKind::SetupConst);
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace) => {
                    self.push(base, namespace.local.span, SetupBindingKind::SetupConst);
                }
            }
        }
    }

    fn declarator(
        &mut self,
        base: u32,
        kind: VariableDeclarationKind,
        declarator: &VariableDeclarator<'a>,
    ) {
        let BindingPattern::BindingIdentifier(id) = &declarator.id else {
            self.pattern(base, kind, declarator);
            return;
        };
        let name = id.name.as_str();
        let by_kind = match kind {
            VariableDeclarationKind::Let | VariableDeclarationKind::Var => {
                SetupBindingKind::SetupLet
            }
            _ => SetupBindingKind::SetupConst,
        };
        let mut alias = None;
        let class = match declarator.init.as_ref() {
            None => by_kind,
            Some(init) => match call(init) {
                Some(call) => self.call_kind(call, by_kind, kind, init),
                None => {
                    if let Expression::Identifier(api) = init {
                        alias = reactive_call(api.name.as_str()).map(|(api, _)| api);
                    }
                    by_initializer(kind, init)
                }
            },
        };
        self.aliases.retain(|(local, _)| *local != name);
        if let Some(api) = alias {
            self.aliases.push((name, api));
        }
        self.push(base, id.span, class);
    }

    /// Every identifier of a destructuring pattern shares one kind, except
    /// in a `defineProps` destructure.
    fn pattern(
        &mut self,
        base: u32,
        kind: VariableDeclarationKind,
        declarator: &VariableDeclarator<'a>,
    ) {
        let init = declarator.init.as_ref();
        let class = match &declarator.id {
            BindingPattern::ObjectPattern(pattern) if init.is_some_and(destructures_props) => {
                self.props_destructure(base, pattern);
                return;
            }
            BindingPattern::ArrayPattern(_)
                if init
                    .and_then(call)
                    .is_some_and(|call| is_callee(call, super::super::DEFINE_MODEL)) =>
            {
                SetupBindingKind::SetupMaybeRef
            }
            _ => by_destructure(kind, init),
        };
        declarator.id.all_binding_identifiers(&mut |id| {
            self.push(base, id.span, class);
            true
        });
    }

    /// A second destructure, like a pattern Vue rejects, stays unclassified.
    fn props_destructure(&mut self, base: u32, pattern: &oxc_ast::ast::ObjectPattern<'a>) {
        let mut overflow = false;
        let observed = props::destructure(pattern, |span| {
            match (base.checked_add(span.start), base.checked_add(span.end)) {
                (Some(start), Some(end)) => Span::new(start, end),
                _ => {
                    overflow = true;
                    Span::new(0, 0)
                }
            }
        });
        self.overflow |= overflow;
        match observed {
            Some((destructure, rows)) if self.props.is_none() => {
                self.props = Some(destructure);
                self.rows.extend(rows);
            }
            _ => {
                let pattern = self.authored(base, pattern.span);
                self.unclassified.push(pattern);
            }
        }
    }

    fn call_kind(
        &self,
        call: &CallExpression<'a>,
        by_kind: SetupBindingKind,
        kind: VariableDeclarationKind,
        init: &Expression<'a>,
    ) -> SetupBindingKind {
        let Expression::Identifier(callee) = &call.callee else {
            return by_initializer(kind, init);
        };
        let callee = callee.name.as_str();
        // The compiler's macro set: Vue's macros and `defineArt`.
        if let Some(row) =
            lookup(callee).filter(|row| row.builtin || row.kind == MacroKind::DefineArt)
        {
            return match row.kind {
                MacroKind::DefineProps | MacroKind::WithDefaults => {
                    SetupBindingKind::SetupReactiveConst
                }
                MacroKind::DefineModel => SetupBindingKind::SetupRef,
                _ => by_kind,
            };
        }
        let api = self
            .aliases
            .iter()
            .rev()
            .find(|(local, _)| *local == callee)
            .map_or(callee, |(_, api)| *api);
        reactive_call(api).map_or_else(|| by_initializer(kind, init), |(_, class)| class)
    }
}

fn is_callee(call: &CallExpression<'_>, name: &str) -> bool {
    matches!(&call.callee, Expression::Identifier(id) if id.name == name)
}

/// `defineProps()` or `withDefaults(defineProps(), ...)`, unwrapped by nothing.
fn destructures_props(init: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = init else {
        return false;
    };
    is_callee(call, super::super::DEFINE_PROPS)
        || (is_callee(call, super::super::WITH_DEFAULTS)
            && matches!(
                call.arguments.first(),
                Some(oxc_ast::ast::Argument::CallExpression(inner))
                    if is_callee(inner, super::super::DEFINE_PROPS)
            ))
}

/// A destructured `const` holds functions only when the initializer is one.
fn by_destructure(
    kind: VariableDeclarationKind,
    init: Option<&Expression<'_>>,
) -> SetupBindingKind {
    match (kind, init) {
        (VariableDeclarationKind::Let | VariableDeclarationKind::Var, _) => {
            SetupBindingKind::SetupLet
        }
        (
            VariableDeclarationKind::Const,
            Some(Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)),
        ) => SetupBindingKind::SetupConst,
        (VariableDeclarationKind::Const, Some(_)) => SetupBindingKind::SetupMaybeRef,
        _ => SetupBindingKind::SetupConst,
    }
}

/// The call an initializer evaluates to, through TS wrappers and parentheses.
fn call<'e, 'a>(expression: &'e Expression<'a>) -> Option<&'e CallExpression<'a>> {
    match transparent(expression) {
        Expression::CallExpression(call) => Some(call),
        _ => None,
    }
}

fn transparent<'e, 'a>(expression: &'e Expression<'a>) -> &'e Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(inner) => transparent(&inner.expression),
        Expression::TSAsExpression(inner) => transparent(&inner.expression),
        Expression::TSSatisfiesExpression(inner) => transparent(&inner.expression),
        Expression::TSNonNullExpression(inner) => transparent(&inner.expression),
        other => other,
    }
}

/// `const` by initializer shape; `let`/`var` stay reassignable.
fn by_initializer(kind: VariableDeclarationKind, init: &Expression<'_>) -> SetupBindingKind {
    match kind {
        VariableDeclarationKind::Const => {}
        VariableDeclarationKind::Let | VariableDeclarationKind::Var => {
            return SetupBindingKind::SetupLet;
        }
        VariableDeclarationKind::Using | VariableDeclarationKind::AwaitUsing => {
            return SetupBindingKind::SetupConst;
        }
    }
    match transparent(init) {
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_) => SetupBindingKind::LiteralConst,
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            SetupBindingKind::LiteralConst
        }
        Expression::UnaryExpression(unary)
            if unary.operator == oxc_ast::ast::UnaryOperator::UnaryNegation
                && matches!(unary.argument, Expression::NumericLiteral(_)) =>
        {
            SetupBindingKind::LiteralConst
        }
        Expression::ArrowFunctionExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ObjectExpression(_)
        | Expression::ArrayExpression(_) => SetupBindingKind::SetupConst,
        _ => SetupBindingKind::SetupMaybeRef,
    }
}

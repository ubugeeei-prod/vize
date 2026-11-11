//! How a template reads each top-level `<script setup>` declaration.
//!
//! The classification follows Vue's `walkDeclaration`: only Vue imports make
//! the ref/reactive call families authoritative, mutable declarations stay
//! mutable, and literal constants depend on script-setup static hoisting.

mod classify;
mod pattern;

use self::classify::{by_initializer, call, is_static_node};
use super::super::{MacroKind, lookup};
use super::props::{self, PropsDestructure};
use alloc::vec::Vec;
use oxc_ast::ast::{
    BindingPattern, CallExpression, Declaration, Expression, ImportDeclaration,
    ImportDeclarationSpecifier, Statement, VariableDeclarationKind, VariableDeclarator,
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

/// The reactivity API whose result initializes a binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactiveSource {
    Ref,
    ShallowRef,
    CustomRef,
    Computed,
    ToRef,
    ToRefs,
    TemplateRef,
    Reactive,
    ShallowReactive,
    Readonly,
    ShallowReadonly,
    /// `defineProps()`, or `withDefaults()` around it.
    Props,
    /// `defineModel()`.
    Model,
}

impl ReactiveSource {
    /// Whether an inline template reads the binding through `.value`.
    #[must_use]
    pub const fn reads_value(self) -> bool {
        matches!(
            self,
            Self::Ref
                | Self::ShallowRef
                | Self::CustomRef
                | Self::Computed
                | Self::ToRef
                | Self::TemplateRef
                | Self::Model
        )
    }
}

/// One binding introduced by a top-level identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetupBinding {
    /// The declared identifier.
    pub name: Span,
    pub kind: SetupBindingKind,
    /// The reactivity API call that initializes it, if any.
    pub source: Option<ReactiveSource>,
}

/// Calls whose result kind Vue's `walkDeclaration` knows, by imported API.
const VUE_SETUP_CALLS: &[(&str, SetupBindingKind, ReactiveSource)] = &[
    ("ref", SetupBindingKind::SetupRef, ReactiveSource::Ref),
    (
        "shallowRef",
        SetupBindingKind::SetupRef,
        ReactiveSource::ShallowRef,
    ),
    (
        "computed",
        SetupBindingKind::SetupRef,
        ReactiveSource::Computed,
    ),
    ("toRef", SetupBindingKind::SetupRef, ReactiveSource::ToRef),
    (
        "customRef",
        SetupBindingKind::SetupRef,
        ReactiveSource::CustomRef,
    ),
    (
        "useTemplateRef",
        SetupBindingKind::SetupRef,
        ReactiveSource::TemplateRef,
    ),
    (
        "reactive",
        SetupBindingKind::SetupReactiveConst,
        ReactiveSource::Reactive,
    ),
];

fn vue_setup_call(name: &str) -> Option<&'static (&'static str, SetupBindingKind, ReactiveSource)> {
    VUE_SETUP_CALLS.iter().find(|(api, _, _)| *api == name)
}

/// Binding rows plus the local names that alias a Vue reactivity API.
#[derive(Debug)]
pub(in crate::framework::vue) struct Bindings<'a> {
    pub(in crate::framework::vue) rows: Vec<SetupBinding>,
    pub(super) props: Option<PropsDestructure>,
    /// `defineProps` destructures the facts do not cover.
    pub(super) unclassified: Vec<Span>,
    /// The destructure's defaults by prop key.
    pub(super) defaults: Vec<(&'a str, Span)>,
    aliases: Vec<(&'a str, &'static str)>,
    hoist_static: bool,
    script_setup: bool,
    pub(in crate::framework::vue) overflow: bool,
}

impl Default for Bindings<'_> {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            props: None,
            unclassified: Vec::new(),
            defaults: Vec::new(),
            aliases: Vec::new(),
            hoist_static: true,
            script_setup: true,
            overflow: false,
        }
    }
}

impl<'a> Bindings<'a> {
    pub(in crate::framework::vue) fn ordinary() -> Self {
        Self {
            script_setup: false,
            ..Self::default()
        }
    }

    pub(super) fn set_hoist_static(&mut self, enabled: bool) {
        self.hoist_static = enabled;
    }

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
        self.push_source(base, name, kind, None);
    }

    fn push_source(
        &mut self,
        base: u32,
        name: oxc_span::Span,
        kind: SetupBindingKind,
        source: Option<ReactiveSource>,
    ) {
        let name = self.authored(base, name);
        self.rows.push(SetupBinding { name, kind, source });
    }

    pub(in crate::framework::vue) fn statement(&mut self, base: u32, statement: &Statement<'a>) {
        match statement {
            Statement::ImportDeclaration(import) if !import.import_kind.is_type() => {
                self.import(base, import);
            }
            Statement::VariableDeclaration(declaration) if !declaration.declare => {
                for declarator in &declaration.declarations {
                    self.declarator(base, declaration.kind, declarator);
                }
            }
            Statement::FunctionDeclaration(function) if !function.declare => {
                if let Some(id) = &function.id {
                    self.push(base, id.span, SetupBindingKind::SetupConst);
                }
            }
            Statement::ClassDeclaration(class) if !class.declare => {
                if let Some(id) = &class.id {
                    self.push(base, id.span, SetupBindingKind::SetupConst);
                }
            }
            Statement::TSEnumDeclaration(enumeration) if !enumeration.declare => {
                let kind = if enumeration
                    .body
                    .members
                    .iter()
                    .all(|member| member.initializer.as_ref().is_none_or(is_static_node))
                {
                    SetupBindingKind::LiteralConst
                } else {
                    SetupBindingKind::SetupConst
                };
                self.push(base, enumeration.id.span, kind);
            }
            Statement::ExportNamedDeclaration(export) if !self.script_setup => {
                if let Some(declaration) = &export.declaration {
                    self.declaration(base, declaration);
                }
            }
            _ => {}
        }
    }

    fn declaration(&mut self, base: u32, declaration: &Declaration<'a>) {
        match declaration {
            Declaration::VariableDeclaration(declaration) if !declaration.declare => {
                for declarator in &declaration.declarations {
                    self.declarator(base, declaration.kind, declarator);
                }
            }
            Declaration::FunctionDeclaration(function) if !function.declare => {
                if let Some(id) = &function.id {
                    self.push(base, id.span, SetupBindingKind::SetupConst);
                }
            }
            Declaration::ClassDeclaration(class) if !class.declare => {
                if let Some(id) = &class.id {
                    self.push(base, id.span, SetupBindingKind::SetupConst);
                }
            }
            Declaration::TSEnumDeclaration(enumeration) if !enumeration.declare => {
                let kind = if enumeration
                    .body
                    .members
                    .iter()
                    .all(|member| member.initializer.as_ref().is_none_or(is_static_node))
                {
                    SetupBindingKind::LiteralConst
                } else {
                    SetupBindingKind::SetupConst
                };
                self.push(base, enumeration.id.span, kind);
            }
            _ => {}
        }
    }

    /// Vue imports are constants; named imports from other modules may be refs.
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
                    if from_vue && let Some((api, _, _)) = vue_setup_call(imported.as_str()) {
                        self.aliases.push((local, api));
                    }
                    let kind = if from_vue {
                        SetupBindingKind::SetupConst
                    } else {
                        SetupBindingKind::SetupMaybeRef
                    };
                    self.push(base, named.local.span, kind);
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                    let kind = if from_vue || import.source.value.ends_with(".vue") {
                        SetupBindingKind::SetupConst
                    } else {
                        SetupBindingKind::SetupMaybeRef
                    };
                    self.push(base, default.local.span, kind);
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
        let by_kind = match kind {
            VariableDeclarationKind::Const => SetupBindingKind::SetupConst,
            _ => SetupBindingKind::SetupLet,
        };
        let (class, source) = match declarator.init.as_ref() {
            None => (by_kind, None),
            Some(init) => match call(init) {
                Some(call) => self.call_kind(call, kind, init),
                None => (by_initializer(kind, init, self.hoist_static), None),
            },
        };
        self.push_source(base, id.span, class, source);
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
            Some((destructure, rows, defaults)) if self.props.is_none() => {
                self.props = Some(destructure);
                self.rows.extend(rows);
                self.defaults = defaults;
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
        kind: VariableDeclarationKind,
        init: &Expression<'a>,
    ) -> (SetupBindingKind, Option<ReactiveSource>) {
        let Expression::Identifier(callee) = &call.callee else {
            return (by_initializer(kind, init, self.hoist_static), None);
        };
        let callee = callee.name.as_str();
        let is_const = kind == VariableDeclarationKind::Const;
        if self.script_setup
            && let Some(row) = lookup(callee).filter(|row| row.builtin)
        {
            return match (is_const, row.kind) {
                (true, MacroKind::DefineProps) => (
                    SetupBindingKind::SetupReactiveConst,
                    Some(ReactiveSource::Props),
                ),
                (true, MacroKind::DefineModel) => {
                    (SetupBindingKind::SetupRef, Some(ReactiveSource::Model))
                }
                (
                    true,
                    MacroKind::WithDefaults | MacroKind::DefineEmits | MacroKind::DefineSlots,
                ) => (SetupBindingKind::SetupConst, None),
                (false, _) => (SetupBindingKind::SetupLet, None),
                _ => (by_initializer(kind, init, self.hoist_static), None),
            };
        }
        let Some(api) = self
            .aliases
            .iter()
            .rev()
            .find(|(local, _)| *local == callee)
            .map(|(_, api)| *api)
        else {
            return (by_initializer(kind, init, self.hoist_static), None);
        };
        let Some((_, class, source)) = vue_setup_call(api) else {
            return (by_initializer(kind, init, self.hoist_static), None);
        };
        if !is_const {
            return (SetupBindingKind::SetupLet, None);
        }
        (*class, Some(*source))
    }
}

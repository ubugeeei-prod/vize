//! The attached-binding half of the owned mirror, split from
//! [`owned`](super) along the op-family boundary (region ops there,
//! binding ops here) when `ui.bind`/`ui.on` grew the family past the
//! source budget. [`own_binding`]'s match stays exhaustive with no `_`
//! arm on purpose — the same staleness discipline as the parent module.

use alloc::vec::Vec;

use vize_l0::{Span, String};

use crate::dump::owned::expr::{Contract, Expr, own_expr};
use crate::dump::owned::{Attribute, Name, own_attribute, own_name};
use crate::op::BindingOp;

/// Mirror of [`BindingOp`]: one attached op.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Binding {
    /// `ui.bind`.
    Bind(Bind),
    /// `ui.on`.
    On(On),
    /// `ui.model`.
    Model(Model),
    /// `ui.slot-content`.
    SlotContent(SlotContent),
    /// `vue.directive`.
    VueDirective(VueDirective),
    /// `vue.css-bind`.
    VueCssBind(VueCssBind),
    /// `vue.sync`.
    VueSync(VueSync),
    /// `vue.slot-scope`.
    VueSlotScope(VueSlotScope),
    /// `vue.once`.
    VueOnce(VueOnce),
    /// `vue.memo`.
    VueMemo(VueMemo),
    /// `vue.show`.
    VueShow(VueShow),
    /// `vue.html`.
    VueHtml(VueHtml),
    /// `vue.text`.
    VueText(VueText),
    /// `vue.cloak`.
    VueCloak(VueCloak),
}

/// Mirror of [`crate::op::BindOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bind {
    /// The bound name; `None` for the object-spread form.
    pub name: Option<Name>,
    /// Modifier names, in order.
    pub modifiers: Vec<String>,
    /// The bound value, when present.
    pub value: Option<Expr>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::OnOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct On {
    /// The event name; `None` for the object form.
    pub name: Option<Name>,
    /// Modifier names, in order.
    pub modifiers: Vec<String>,
    /// The handler expression, when authored.
    pub handler: Option<Expr>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::ModelOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    /// The binding contract.
    pub contract: Contract,
    /// Authored model prop name, when present.
    pub argument: Option<Name>,
    /// Element kind and dialect modifiers, in order.
    pub attributes: Vec<Attribute>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::SlotContentOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotContent {
    /// The authored slot name, when present.
    pub name: Option<Name>,
    /// Modifier names, in order.
    pub modifiers: Vec<String>,
    /// The authored params position, when non-blank.
    pub params: Option<Expr>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::VueSyncOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VueSync {
    /// The bound prop name (always static).
    pub name: String,
    /// Modifiers other than `sync`, in order.
    pub modifiers: Vec<String>,
    /// The value written back into.
    pub value: Expr,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::VueSlotScopeOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VueSlotScope {
    /// The companion `slot` name; `None` is the default slot.
    pub name: Option<String>,
    /// The slot-props expression, when authored.
    pub params: Option<Expr>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::VueDirectiveOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VueDirective {
    /// Directive name without the `v-` prefix.
    pub name: String,
    /// The authored argument, when present.
    pub argument: Option<Name>,
    /// Modifier names, in order.
    pub modifiers: Vec<String>,
    /// The value expression, when authored.
    pub value: Option<Expr>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::VueCssBindOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VueCssBind {
    /// The `v-bind()` argument.
    pub value: Expr,
    /// The call's range, relative to the style block.
    pub span: Span,
}

/// Mirror of [`crate::op::VueOnceOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VueOnce {
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::VueMemoOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VueMemo {
    /// The memo dependency expression.
    pub value: Expr,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::VueShowOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VueShow {
    /// The display predicate expression.
    pub value: Expr,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::VueHtmlOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VueHtml {
    /// The raw HTML expression, when authored.
    pub value: Option<Expr>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::VueTextOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VueText {
    /// The text-content expression, when authored.
    pub value: Option<Expr>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::VueCloakOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VueCloak {
    /// Source range.
    pub span: Span,
}

pub(super) fn own_binding(binding: &BindingOp<'_>) -> Binding {
    match binding {
        BindingOp::Bind(bind) => Binding::Bind(Bind {
            name: bind.name.as_ref().map(own_name),
            modifiers: own_modifiers(&bind.modifiers),
            value: bind.value.as_ref().map(own_expr),
            span: bind.span,
        }),
        BindingOp::On(on) => Binding::On(On {
            name: on.name.as_ref().map(own_name),
            modifiers: own_modifiers(&on.modifiers),
            handler: on.handler.as_ref().map(own_expr),
            span: on.span,
        }),
        BindingOp::Model(model) => Binding::Model(Model {
            contract: Contract {
                read: own_expr(&model.contract.read),
                write: own_expr(&model.contract.write),
            },
            argument: model.argument.as_ref().map(own_name),
            attributes: model.attributes.iter().map(own_attribute).collect(),
            span: model.span,
        }),
        BindingOp::SlotContent(content) => Binding::SlotContent(SlotContent {
            name: content.name.as_ref().map(own_name),
            modifiers: own_modifiers(&content.modifiers),
            params: content.params.as_ref().map(own_expr),
            span: content.span,
        }),
        BindingOp::VueDirective(directive) => Binding::VueDirective(VueDirective {
            name: String::from(directive.name),
            argument: directive.argument.as_ref().map(own_name),
            modifiers: own_modifiers(&directive.modifiers),
            value: directive.value.as_ref().map(own_expr),
            span: directive.span,
        }),
        BindingOp::VueCssBind(bind) => Binding::VueCssBind(VueCssBind {
            value: own_expr(&bind.value),
            span: bind.span,
        }),
        BindingOp::VueSync(sync) => Binding::VueSync(VueSync {
            name: String::from(sync.name),
            modifiers: own_modifiers(&sync.modifiers),
            value: own_expr(&sync.value),
            span: sync.span,
        }),
        BindingOp::VueSlotScope(scope) => Binding::VueSlotScope(VueSlotScope {
            name: scope.name.map(String::from),
            params: scope.params.as_ref().map(own_expr),
            span: scope.span,
        }),
        BindingOp::VueOnce(once) => Binding::VueOnce(VueOnce { span: once.span }),
        BindingOp::VueMemo(memo) => Binding::VueMemo(VueMemo {
            value: own_expr(&memo.value),
            span: memo.span,
        }),
        BindingOp::VueShow(show) => Binding::VueShow(VueShow {
            value: own_expr(&show.value),
            span: show.span,
        }),
        BindingOp::VueHtml(html) => Binding::VueHtml(VueHtml {
            value: html.value.as_ref().map(own_expr),
            span: html.span,
        }),
        BindingOp::VueText(text) => Binding::VueText(VueText {
            value: text.value.as_ref().map(own_expr),
            span: text.span,
        }),
        BindingOp::VueCloak(cloak) => Binding::VueCloak(VueCloak { span: cloak.span }),
    }
}

fn own_modifiers(modifiers: &[&str]) -> Vec<String> {
    modifiers.iter().map(|text| String::from(*text)).collect()
}

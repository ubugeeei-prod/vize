//! The attached-binding half of the owned mirror, split from
//! [`owned`](super) along the op-family boundary (region ops there,
//! binding ops here) when `ui.bind`/`ui.on` grew the family past the
//! source budget. [`own_binding`]'s match stays exhaustive with no `_`
//! arm on purpose — the same staleness discipline as the parent module.

use alloc::vec::Vec;

use vize_carton::{Span, String};

use super::expr::{FolioContract, FolioExpr, own_expr};
use super::{FolioAttribute, FolioName, own_attribute, own_name};
use crate::op::BindingOp;

/// Mirror of [`BindingOp`]: one attached op.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FolioBinding {
    /// `ui.bind`.
    Bind(FolioBind),
    /// `ui.on`.
    On(FolioOn),
    /// `ui.model`.
    Model(FolioModel),
    /// `ui.slot-content`.
    SlotContent(FolioSlotContent),
    /// `vue.directive`.
    VueDirective(FolioVueDirective),
    /// `vue.sync`.
    VueSync(FolioVueSync),
    /// `vue.slot-scope`.
    VueSlotScope(FolioVueSlotScope),
}

/// Mirror of [`crate::op::BindOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioBind {
    /// The bound name; `None` for the object-spread form.
    pub name: Option<FolioName>,
    /// Modifier names, in order.
    pub modifiers: Vec<String>,
    /// The bound value, when present.
    pub value: Option<FolioExpr>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::OnOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioOn {
    /// The event name; `None` for the object form.
    pub name: Option<FolioName>,
    /// Modifier names, in order.
    pub modifiers: Vec<String>,
    /// The handler expression, when authored.
    pub handler: Option<FolioExpr>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::ModelOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioModel {
    /// The binding contract.
    pub contract: FolioContract,
    /// Element kind and dialect modifiers, in order.
    pub attributes: Vec<FolioAttribute>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::SlotContentOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioSlotContent {
    /// The authored slot name, when present.
    pub name: Option<FolioName>,
    /// Modifier names, in order.
    pub modifiers: Vec<String>,
    /// The authored params position, when non-blank.
    pub params: Option<FolioExpr>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::VueSyncOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioVueSync {
    /// The bound prop name.
    pub name: FolioName,
    /// Modifiers other than `sync`, in order.
    pub modifiers: Vec<String>,
    /// The value written back into.
    pub value: FolioExpr,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::VueSlotScopeOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioVueSlotScope {
    /// The companion `slot` name; `None` is the default slot.
    pub name: Option<String>,
    /// The slot-props expression, when authored.
    pub params: Option<FolioExpr>,
    /// Source range.
    pub span: Span,
}

/// Mirror of [`crate::op::VueDirectiveOp`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioVueDirective {
    /// Directive name without the `v-` prefix.
    pub name: String,
    /// The authored argument, when present.
    pub argument: Option<FolioName>,
    /// Modifier names, in order.
    pub modifiers: Vec<String>,
    /// The value expression, when authored.
    pub value: Option<FolioExpr>,
    /// Source range.
    pub span: Span,
}

pub(super) fn own_binding(binding: &BindingOp<'_>) -> FolioBinding {
    match binding {
        BindingOp::Bind(bind) => FolioBinding::Bind(FolioBind {
            name: bind.name.as_ref().map(own_name),
            modifiers: own_modifiers(&bind.modifiers),
            value: bind.value.as_ref().map(own_expr),
            span: bind.span,
        }),
        BindingOp::On(on) => FolioBinding::On(FolioOn {
            name: on.name.as_ref().map(own_name),
            modifiers: own_modifiers(&on.modifiers),
            handler: on.handler.as_ref().map(own_expr),
            span: on.span,
        }),
        BindingOp::Model(model) => FolioBinding::Model(FolioModel {
            contract: FolioContract {
                read: own_expr(&model.contract.read),
                write: own_expr(&model.contract.write),
            },
            attributes: model.attributes.iter().map(own_attribute).collect(),
            span: model.span,
        }),
        BindingOp::SlotContent(content) => FolioBinding::SlotContent(FolioSlotContent {
            name: content.name.as_ref().map(own_name),
            modifiers: own_modifiers(&content.modifiers),
            params: content.params.as_ref().map(own_expr),
            span: content.span,
        }),
        BindingOp::VueDirective(directive) => FolioBinding::VueDirective(FolioVueDirective {
            name: String::from(directive.name),
            argument: directive.argument.as_ref().map(own_name),
            modifiers: own_modifiers(&directive.modifiers),
            value: directive.value.as_ref().map(own_expr),
            span: directive.span,
        }),
        BindingOp::VueSync(sync) => FolioBinding::VueSync(FolioVueSync {
            name: own_name(&sync.name),
            modifiers: own_modifiers(&sync.modifiers),
            value: own_expr(&sync.value),
            span: sync.span,
        }),
        BindingOp::VueSlotScope(scope) => FolioBinding::VueSlotScope(FolioVueSlotScope {
            name: scope.name.map(String::from),
            params: scope.params.as_ref().map(own_expr),
            span: scope.span,
        }),
    }
}

fn own_modifiers(modifiers: &[&str]) -> Vec<String> {
    modifiers.iter().map(|text| String::from(*text)).collect()
}

//! Typed S2 DOM refusal reasons.
//!
//! The new DOM lane is still a strangler path. Refusing a shape is allowed
//! while P2-11 is open, but an opaque refusal is not: every exit needs a
//! stable bucket so corpus runs can drive the next installment from counts.

use core::fmt;

use vize_davinci::id::NodeId;
use vize_s0::Span;
use vize_s2::op::{BindingOp, Op};

/// Stable census bucket for an S2 DOM emission refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnsupportedReason {
    ArrayBuiltinCannotUseSlotObject,
    ArrayChildTextRun,
    BareStyleAttributeWithDynamicStyle,
    BindNameNotJs,
    BindRequiresStaticName,
    BindValueNotJs,
    /// Retired after `createSlots` started threading the slot-content proof.
    CreateSlotsMissingSlotTemplate,
    CustomDirectiveExprNotJs,
    DuplicateClassBinding,
    DuplicateStyleBinding,
    DynamicOnHasModifiers,
    EmptyCompoundText,
    EmptyTextRun,
    ForAliasNotEmittable,
    ForItemShape,
    ForSourceNotJs,
    HtmlExpressionNotJs,
    IfBranchShape,
    IfConditionNotJs,
    IfWithoutBranches,
    LoneObjectArgument,
    MemoExpressionNotJs,
    MissingTextFacts,
    ModelArgumentNotJs,
    ModelExpressionNotJs,
    ObjectBindHasModifiers,
    ObjectOnHasModifiers,
    ObjectOnHandlerNotJs,
    OnHandlerNotJs,
    OnNameNotJs,
    OnNameNotStatic,
    /// `prefix_identifiers`: a foreign or filter expression has no
    /// shipped prefix behavior to mirror yet.
    PrefixExpressionKind,
    /// `prefix_identifiers`: the shipped lane reports a non-recoverable
    /// `X_INVALID_EXPRESSION` for this text, so there is no output to match.
    PrefixExpressionRejected,
    ShowExpressionNotJs,
    SlotDefaultShape,
    SlotFactsMissingGroup,
    SlotNameUnderscore,
    SlotOutletNameNotJs,
    SlotOutletPropKind,
    SlotsSpreadShape,
    SlotsSpreadValueNotJs,
    TemplateDynamicKeyEmpty,
    TemplateUnwrapShape,
    TextDirectiveExpressionNotJs,
    TextExpressionNotEmittable,
    TextRunContainsNonText,
    /// `is_ts` was requested from a build without the `typescript`
    /// feature, whose type erasure needs `std`.
    TypeScriptLaneUnavailable,
    UnsupportedBindingKind,
    WalkIdOverflow,
}

impl UnsupportedReason {
    /// All stable buckets. Tests assert the committed census covers every one.
    pub const ALL: &'static [Self] = &[
        Self::ArrayBuiltinCannotUseSlotObject,
        Self::ArrayChildTextRun,
        Self::BareStyleAttributeWithDynamicStyle,
        Self::BindNameNotJs,
        Self::BindRequiresStaticName,
        Self::BindValueNotJs,
        Self::CreateSlotsMissingSlotTemplate,
        Self::CustomDirectiveExprNotJs,
        Self::DuplicateClassBinding,
        Self::DuplicateStyleBinding,
        Self::DynamicOnHasModifiers,
        Self::EmptyCompoundText,
        Self::EmptyTextRun,
        Self::ForAliasNotEmittable,
        Self::ForItemShape,
        Self::ForSourceNotJs,
        Self::HtmlExpressionNotJs,
        Self::IfBranchShape,
        Self::IfConditionNotJs,
        Self::IfWithoutBranches,
        Self::LoneObjectArgument,
        Self::MemoExpressionNotJs,
        Self::MissingTextFacts,
        Self::ModelArgumentNotJs,
        Self::ModelExpressionNotJs,
        Self::ObjectBindHasModifiers,
        Self::ObjectOnHandlerNotJs,
        Self::ObjectOnHasModifiers,
        Self::OnHandlerNotJs,
        Self::OnNameNotJs,
        Self::OnNameNotStatic,
        Self::PrefixExpressionKind,
        Self::PrefixExpressionRejected,
        Self::ShowExpressionNotJs,
        Self::SlotDefaultShape,
        Self::SlotFactsMissingGroup,
        Self::SlotNameUnderscore,
        Self::SlotOutletNameNotJs,
        Self::SlotOutletPropKind,
        Self::SlotsSpreadShape,
        Self::SlotsSpreadValueNotJs,
        Self::TemplateDynamicKeyEmpty,
        Self::TemplateUnwrapShape,
        Self::TextDirectiveExpressionNotJs,
        Self::TextExpressionNotEmittable,
        Self::TextRunContainsNonText,
        Self::TypeScriptLaneUnavailable,
        Self::UnsupportedBindingKind,
        Self::WalkIdOverflow,
    ];

    /// Machine-stable key used by refusal census output.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::ArrayBuiltinCannotUseSlotObject => "array_builtin_cannot_use_slot_object",
            Self::ArrayChildTextRun => "array_child_text_run",
            Self::BareStyleAttributeWithDynamicStyle => "bare_style_attr_with_dynamic_style",
            Self::BindNameNotJs => "bind_name_not_js",
            Self::BindRequiresStaticName => "bind_requires_static_name",
            Self::BindValueNotJs => "bind_value_not_js",
            Self::CreateSlotsMissingSlotTemplate => "create_slots_missing_slot_template",
            Self::CustomDirectiveExprNotJs => "custom_directive_expr_not_js",
            Self::DuplicateClassBinding => "duplicate_class_binding",
            Self::DuplicateStyleBinding => "duplicate_style_binding",
            Self::DynamicOnHasModifiers => "dynamic_on_has_modifiers",
            Self::EmptyCompoundText => "empty_compound_text",
            Self::EmptyTextRun => "empty_text_run",
            Self::ForAliasNotEmittable => "for_alias_not_emittable",
            Self::ForItemShape => "for_item_shape",
            Self::ForSourceNotJs => "for_source_not_js",
            Self::HtmlExpressionNotJs => "html_expression_not_js",
            Self::IfBranchShape => "if_branch_shape",
            Self::IfConditionNotJs => "if_condition_not_js",
            Self::IfWithoutBranches => "if_without_branches",
            Self::LoneObjectArgument => "lone_object_argument",
            Self::MemoExpressionNotJs => "memo_expression_not_js",
            Self::MissingTextFacts => "missing_text_facts",
            Self::ModelArgumentNotJs => "model_argument_not_js",
            Self::ModelExpressionNotJs => "model_expression_not_js",
            Self::ObjectBindHasModifiers => "object_bind_has_modifiers",
            Self::ObjectOnHasModifiers => "object_on_has_modifiers",
            Self::ObjectOnHandlerNotJs => "object_on_handler_not_js",
            Self::OnHandlerNotJs => "on_handler_not_js",
            Self::OnNameNotJs => "on_name_not_js",
            Self::OnNameNotStatic => "on_name_not_static",
            Self::PrefixExpressionKind => "prefix_expression_kind",
            Self::PrefixExpressionRejected => "prefix_expression_rejected",
            Self::TypeScriptLaneUnavailable => "typescript_lane_unavailable",
            Self::ShowExpressionNotJs => "show_expression_not_js",
            Self::SlotDefaultShape => "slot_default_shape",
            Self::SlotFactsMissingGroup => "slot_facts_missing_group",
            Self::SlotNameUnderscore => "slot_name_underscore",
            Self::SlotOutletNameNotJs => "slot_outlet_name_not_js",
            Self::SlotOutletPropKind => "slot_outlet_prop_kind",
            Self::SlotsSpreadShape => "slots_spread_shape",
            Self::SlotsSpreadValueNotJs => "slots_spread_value_not_js",
            Self::TemplateDynamicKeyEmpty => "template_dynamic_key_empty",
            Self::TextDirectiveExpressionNotJs => "text_directive_expression_not_js",
            Self::TemplateUnwrapShape => "template_unwrap_shape",
            Self::TextExpressionNotEmittable => "text_expression_not_emittable",
            Self::TextRunContainsNonText => "text_run_contains_non_text",
            Self::UnsupportedBindingKind => "unsupported_binding_kind",
            Self::WalkIdOverflow => "walk_id_overflow",
        }
    }
}

impl fmt::Display for UnsupportedReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// One classified refusal, without authored source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnsupportedRefusal {
    pub reason: UnsupportedReason,
    pub span: Option<Span>,
    pub node: Option<NodeId>,
}

impl UnsupportedRefusal {
    #[must_use]
    pub const fn new(reason: UnsupportedReason) -> Self {
        Self {
            reason,
            span: None,
            node: None,
        }
    }

    #[must_use]
    pub const fn at(reason: UnsupportedReason, span: Span) -> Self {
        Self {
            reason,
            span: Some(span),
            node: None,
        }
    }

    #[must_use]
    pub const fn at_node(reason: UnsupportedReason, span: Span, node: NodeId) -> Self {
        Self {
            reason,
            span: Some(span),
            node: Some(node),
        }
    }
}

/// Why emission refused. Never a panic: unsupported S2 shapes are counted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmitError {
    /// S2 carries an error diagnostic; refuse to guess a render function.
    Diagnostics,
    /// This installment does not emit this shape.
    Unsupported(UnsupportedRefusal),
}

impl EmitError {
    #[must_use]
    pub const fn unsupported(reason: UnsupportedReason) -> Self {
        Self::Unsupported(UnsupportedRefusal::new(reason))
    }

    #[must_use]
    pub const fn unsupported_at(reason: UnsupportedReason, span: Span) -> Self {
        Self::Unsupported(UnsupportedRefusal::at(reason, span))
    }

    #[must_use]
    pub const fn unsupported_at_node(reason: UnsupportedReason, span: Span, node: NodeId) -> Self {
        Self::Unsupported(UnsupportedRefusal::at_node(reason, span, node))
    }

    #[must_use]
    pub(crate) fn unsupported_binding(reason: UnsupportedReason, binding: &BindingOp<'_>) -> Self {
        Self::unsupported_at(reason, binding_span(binding))
    }

    #[must_use]
    pub(crate) fn unsupported_op(reason: UnsupportedReason, op: &Op<'_>) -> Self {
        Self::unsupported_at(reason, op_span(op))
    }

    #[must_use]
    pub const fn reason(&self) -> Option<UnsupportedReason> {
        match self {
            Self::Diagnostics => None,
            Self::Unsupported(refusal) => Some(refusal.reason),
        }
    }
}

fn binding_span(binding: &BindingOp<'_>) -> Span {
    match binding {
        BindingOp::Bind(op) => op.span,
        BindingOp::On(op) => op.span,
        BindingOp::Model(op) => op.span,
        BindingOp::SlotContent(op) => op.span,
        BindingOp::VueDirective(op) => op.span,
        BindingOp::VueCssBind(op) => op.span,
        BindingOp::VueSync(op) => op.span,
        BindingOp::VueSlotScope(op) => op.span,
        BindingOp::VueOnce(op) => op.span,
        BindingOp::VueMemo(op) => op.span,
        BindingOp::VueShow(op) => op.span,
        BindingOp::VueHtml(op) => op.span,
        BindingOp::VueText(op) => op.span,
        BindingOp::VueCloak(op) => op.span,
    }
}

fn op_span(op: &Op<'_>) -> Span {
    match op {
        Op::Element(op) => op.span,
        Op::Component(op) => op.span,
        Op::Text(op) => op.span,
        Op::Interpolation(op) => op.span,
        Op::If(op) => op.span,
        Op::For(op) => op.span,
        Op::Slot(op) => op.span,
    }
}

const _: () = assert!(!core::mem::needs_drop::<UnsupportedRefusal>());
const _: () = assert!(!core::mem::needs_drop::<EmitError>());

#[cfg(test)]
mod tests {
    use super::UnsupportedReason;

    #[test]
    fn reason_codes_are_unique_and_sorted_by_variant_list() {
        for window in UnsupportedReason::ALL.windows(2) {
            assert!(
                window[0].code() < window[1].code(),
                "{} must stay before {}",
                window[0],
                window[1]
            );
        }
    }
}

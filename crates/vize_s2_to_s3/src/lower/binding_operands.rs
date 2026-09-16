use vize_s0::Span;
use vize_s2::{expr::ExprRef, op as s2};
use vize_s3::op::OpId;
use vize_s3::operand::OperandRole as Role;

use super::{Cx, binding_span};

impl<'a> Cx<'a> {
    pub(super) fn capture_binding(&mut self, id: OpId, target: OpId, binding: &s2::BindingOp<'_>) {
        let span = binding_span(binding);
        let kind = match binding {
            s2::BindingOp::Bind(op) => {
                self.named_binding(id, target, op.name, op.value, &op.modifiers, span);
                "bind"
            }
            s2::BindingOp::On(op) => {
                self.named_binding(id, target, op.name, op.handler, &op.modifiers, span);
                "on"
            }
            s2::BindingOp::Model(op) => {
                self.add_operand(id, Role::Name, Some(target), self.name(op.argument, span));
                self.add_operand(
                    id,
                    Role::ModelRead,
                    Some(target),
                    self.expression(Some(op.contract.read), span),
                );
                self.add_operand(
                    id,
                    Role::ModelWrite,
                    Some(target),
                    self.expression(Some(op.contract.write), span),
                );
                self.capture_attributes(id, Role::ModelAttribute, Some(target), &op.attributes);
                "model"
            }
            s2::BindingOp::SlotContent(op) => {
                self.named_binding(id, target, op.name, None, &op.modifiers, span);
                self.add_operand(
                    id,
                    Role::Params,
                    Some(target),
                    self.expression(op.params, span),
                );
                "slot-content"
            }
            s2::BindingOp::VueDirective(op) => {
                self.named_binding(id, target, op.argument, op.value, &op.modifiers, span);
                self.add_operand(
                    id,
                    Role::Tag,
                    Some(target),
                    self.literal(Some(op.name), span),
                );
                "vue.directive"
            }
            s2::BindingOp::VueSync(op) => {
                self.named_binding(
                    id,
                    target,
                    Some(s2::DynamicName::Static(op.name)),
                    Some(op.value),
                    &op.modifiers,
                    span,
                );
                "vue.sync"
            }
            s2::BindingOp::VueSlotScope(op) => {
                self.add_operand(id, Role::Name, Some(target), self.literal(op.name, span));
                self.add_operand(
                    id,
                    Role::Params,
                    Some(target),
                    self.expression(op.params, span),
                );
                "vue.slot-scope"
            }
            s2::BindingOp::VueOnce(_) => "vue.once",
            s2::BindingOp::VueCloak(_) => "vue.cloak",
            s2::BindingOp::VueMemo(op) => {
                self.binding_value(id, target, Some(op.value), span);
                "vue.memo"
            }
            s2::BindingOp::VueShow(op) => {
                self.binding_value(id, target, Some(op.value), span);
                "vue.show"
            }
            s2::BindingOp::VueHtml(op) => {
                self.binding_value(id, target, op.value, span);
                "vue.html"
            }
            s2::BindingOp::VueText(op) => {
                self.binding_value(id, target, op.value, span);
                "vue.text"
            }
            s2::BindingOp::VueCssBind(op) => {
                self.binding_value(id, target, Some(op.value), span);
                "vue.css-bind"
            }
        };
        self.add_operand(
            id,
            Role::BindingKind,
            Some(target),
            self.literal(Some(kind), span),
        );
    }

    fn binding_value(&mut self, id: OpId, target: OpId, value: Option<ExprRef<'_>>, span: Span) {
        self.add_operand(id, Role::Value, Some(target), self.expression(value, span));
    }

    fn named_binding(
        &mut self,
        id: OpId,
        target: OpId,
        name: Option<s2::DynamicName<'_>>,
        value: Option<ExprRef<'_>>,
        modifiers: &[&str],
        span: Span,
    ) {
        self.add_operand(id, Role::Name, Some(target), self.name(name, span));
        self.binding_value(id, target, value, span);
        for modifier in modifiers {
            self.add_operand(
                id,
                Role::Modifier,
                Some(target),
                self.literal(Some(modifier), span),
            );
        }
    }
}

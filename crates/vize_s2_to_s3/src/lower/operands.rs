use vize_s0::Span;
use vize_s2::{expr::ExprRef, op as s2};
use vize_s3::op::{OpId, RegionId};
use vize_s3::operand::{Operand, OperandRole as Role, OperandValue, ValueKind};

use super::Cx;

impl<'a> Cx<'a> {
    pub(super) fn literal(&self, text: Option<&str>, span: Span) -> OperandValue<'a> {
        OperandValue {
            kind: if text.is_some() {
                ValueKind::Literal
            } else {
                ValueKind::Absent
            },
            text: self.allocator.alloc_str(text.unwrap_or("")),
            qualifier: "",
            span,
        }
    }

    pub(super) fn expression(
        &self,
        expression: Option<ExprRef<'_>>,
        span: Span,
    ) -> OperandValue<'a> {
        let Some(expression) = expression else {
            return self.literal(None, span);
        };
        let (kind, qualifier) = match expression {
            ExprRef::Js(_) => (ValueKind::Js, ""),
            ExprRef::Opaque(value) => (ValueKind::Opaque, value.reason.mnemonic()),
            ExprRef::Foreign(value) => (ValueKind::Foreign, value.dialect),
            ExprRef::Filter(_) => (ValueKind::Filter, ""),
        };
        OperandValue {
            kind,
            text: self.allocator.alloc_str(expression.source()),
            qualifier: self.allocator.alloc_str(qualifier),
            span: expression.span(),
        }
    }

    pub(super) fn name(&self, name: Option<s2::DynamicName<'_>>, span: Span) -> OperandValue<'a> {
        match name {
            Some(s2::DynamicName::Static(text)) => self.literal(Some(text), span),
            Some(s2::DynamicName::Dynamic(expression)) => self.expression(Some(expression), span),
            None => self.literal(None, span),
        }
    }

    pub(super) fn add_operand(
        &mut self,
        op: OpId,
        role: Role,
        target: Option<OpId>,
        value: OperandValue<'a>,
    ) {
        self.program.operands.push(Operand {
            op,
            role,
            target,
            region: None,
            name: None,
            value,
        });
    }

    pub(super) fn capture_attributes(
        &mut self,
        op: OpId,
        role: Role,
        target: Option<OpId>,
        attributes: &[s2::Attribute<'_>],
    ) {
        for attribute in attributes {
            let value = self.literal(attribute.value, attribute.span);
            self.program.operands.push(Operand {
                op,
                role,
                target,
                region: None,
                name: Some(self.allocator.alloc_str(attribute.name)),
                value,
            });
        }
    }

    pub(super) fn capture_element(&mut self, op: OpId, element: &s2::ElementOp<'_>) {
        self.add_operand(
            op,
            Role::Tag,
            None,
            self.literal(Some(element.tag), element.span),
        );
        let namespace = match element.namespace {
            s2::Namespace::Html => "html",
            s2::Namespace::Svg => "svg",
            s2::Namespace::MathMl => "mathml",
        };
        self.add_operand(
            op,
            Role::Namespace,
            None,
            self.literal(Some(namespace), element.span),
        );
        self.capture_attributes(op, Role::Attribute, None, &element.attributes);
    }

    pub(super) fn capture_component(&mut self, op: OpId, component: &s2::ComponentOp<'_>) {
        self.add_operand(
            op,
            Role::Tag,
            None,
            self.literal(Some(component.name), component.span),
        );
        self.capture_attributes(op, Role::Attribute, None, &component.attributes);
    }

    pub(super) fn capture_text(&mut self, op: OpId, text: &str, span: Span, comment: bool) {
        let role = if comment { Role::Comment } else { Role::Text };
        self.add_operand(op, role, None, self.literal(Some(text), span));
    }

    pub(super) fn capture_interpolation(&mut self, op: OpId, expression: ExprRef<'_>) {
        self.add_operand(
            op,
            Role::Text,
            None,
            self.expression(Some(expression), expression.span()),
        );
    }

    pub(super) fn capture_condition(
        &mut self,
        op: OpId,
        region: RegionId,
        condition: Option<ExprRef<'_>>,
        span: Span,
    ) {
        let value = self.expression(condition, span);
        self.program.operands.push(Operand {
            op,
            role: Role::Condition,
            target: None,
            region: Some(region),
            name: None,
            value,
        });
    }

    pub(super) fn capture_for(&mut self, op: OpId, binding: &s2::ForBinding<'_>) {
        for (role, expression) in [
            (Role::ForSource, Some(binding.source)),
            (Role::ForValue, Some(binding.value)),
            (Role::ForKey, binding.key),
            (Role::ForIndex, binding.index),
        ] {
            self.add_operand(
                op,
                role,
                None,
                self.expression(expression, binding.source.span()),
            );
        }
    }

    pub(super) fn capture_slot(&mut self, op: OpId, slot: &s2::SlotOp<'_>) {
        self.add_operand(op, Role::Name, None, self.name(Some(slot.name), slot.span));
        self.capture_attributes(op, Role::Attribute, None, &slot.attributes);
    }
}

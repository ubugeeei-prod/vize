//! Native element spelling from checked semantic L3 facts.

use vize_l0::id::NodeId;
use vize_l2::expr::ExprRef;
use vize_l2::op::{BindingOp, DynamicName, ElementOp};
use vize_l3::decision::dom::{DomChildren, DomNode, PropertyRole};

use super::write::{number, property, quoted, spell};
use super::{DomError, DomErrorKind, Emitter, ExpressionWriter, LinkSink};

impl<E: ExpressionWriter, L: LinkSink> Emitter<'_, '_, '_, E, L> {
    pub(super) fn element(
        &mut self,
        node: NodeId,
        element: &ElementOp<'_>,
        fact: &DomNode<'_, '_>,
        branch_key: Option<u32>,
    ) -> Result<(), DomError> {
        self.writer.anchor(element.span.start);
        if fact.block_eligible {
            self.writer.push("(");
            spell(&mut self.writer, self.vocabulary, self.helpers.open_block);
            self.writer.push("(), ");
            spell(
                &mut self.writer,
                self.vocabulary,
                self.helpers.element_block,
            );
        } else {
            spell(&mut self.writer, self.vocabulary, self.helpers.element);
        }
        self.writer.push("(");
        quoted(&mut self.writer, element.tag, element.span);
        let row = self
            .tables
            .nodes
            .get(node)
            .ok_or_else(|| self.error(node, DomErrorKind::MissingNode))?;
        let has_props = branch_key.is_some()
            || !element.attributes.is_empty()
            || !row.dynamic_bindings.is_empty();
        let has_children = !matches!(fact.children, DomChildren::Empty);
        let changes = fact.changes;
        let flag = u32::from(changes.text)
            | (u32::from(changes.class) << 1)
            | (u32::from(changes.style) << 2)
            | (u32::from(changes.properties) << 3);
        let normalizers = if has_props {
            self.writer.push(", ");
            self.props(node, element, &row.dynamic_bindings, branch_key)?
        } else {
            if has_children || flag != 0 {
                self.writer.push(", null");
            }
            0
        };
        match &fact.children {
            DomChildren::Empty if flag != 0 => self.writer.push(", null"),
            DomChildren::Empty => {}
            DomChildren::Text(text) => {
                self.writer.push(", ");
                self.text(text)?;
            }
            DomChildren::Array(children) => {
                self.writer.push(", ");
                self.array(children)?;
            }
        }
        if flag != 0 {
            self.patch_flag(flag);
        }
        if !fact.dynamic_property_bindings.is_empty() {
            self.writer.push(", [");
            for (index, &binding) in fact.dynamic_property_bindings.iter().enumerate() {
                if index != 0 {
                    self.writer.push(", ");
                }
                let fact = self
                    .facts
                    .binding(binding)
                    .ok_or_else(|| self.error(binding, DomErrorKind::MissingBinding))?;
                let BindingOp::Bind(bind) = fact.binding() else {
                    return Err(self.error(binding, DomErrorKind::InvalidGrouping));
                };
                let Some(DynamicName::Static(name)) = bind.name else {
                    return Err(self.error(binding, DomErrorKind::InvalidGrouping));
                };
                quoted(&mut self.writer, name, bind.span);
            }
            self.writer.push("]");
        }
        self.writer
            .push(if fact.block_eligible { "))" } else { ")" });
        // Normalization and vnode creation close after this element's children.
        // Their syntax was already appended; imports follow actual Vue use order.
        if normalizers & 1 != 0 {
            self.writer.use_helper(self.helpers.class);
        }
        if normalizers & 2 != 0 {
            self.writer.use_helper(self.helpers.style);
        }
        if fact.block_eligible {
            self.writer.use_helper(self.helpers.open_block);
            self.writer.use_helper(self.helpers.element_block);
        } else {
            self.writer.use_helper(self.helpers.element);
        }
        Ok(())
    }

    fn props(
        &mut self,
        owner: NodeId,
        element: &ElementOp<'_>,
        bindings: &[NodeId],
        branch_key: Option<u32>,
    ) -> Result<u8, DomError> {
        let complex_value = bindings.iter().any(|&id| {
            self.facts.binding(id).is_some_and(|fact| {
                fact.role == PropertyRole::Class
                    || (fact.role == PropertyRole::Style && fact.value.is_dynamic())
                    || matches!(fact.binding(), BindingOp::Bind(bind)
                        if bind.value.is_some_and(|value| !inline_value(value)))
            })
        });
        let multiline =
            element.attributes.len() + bindings.len() + usize::from(branch_key.is_some()) > 1
                || complex_value;
        self.writer.push(if multiline { "{" } else { "{ " });
        if multiline {
            self.writer.indent();
        }
        let mut index = 0;
        let mut normalizers = 0;
        if let Some(key) = branch_key {
            self.property_start(index, multiline);
            self.writer.push("key: ");
            number(&mut self.writer, key);
            index += 1;
        }
        for attribute in &element.attributes {
            self.property_start(index, multiline);
            property(&mut self.writer, attribute.name, attribute.span);
            self.writer.push(": ");
            quoted(
                &mut self.writer,
                attribute.value.unwrap_or(""),
                attribute.span,
            );
            index += 1;
        }
        for &id in bindings {
            let fact = self
                .facts
                .binding(id)
                .ok_or_else(|| self.error(id, DomErrorKind::MissingBinding))?;
            let BindingOp::Bind(bind) = fact.binding() else {
                return Err(self.error(id, DomErrorKind::InvalidGrouping));
            };
            let Some(DynamicName::Static(name)) = bind.name else {
                return Err(self.error(id, DomErrorKind::InvalidGrouping));
            };
            let value = bind
                .value
                .ok_or_else(|| self.error(id, DomErrorKind::InvalidGrouping))?;
            self.property_start(index, multiline);
            property(&mut self.writer, name, bind.span);
            self.writer.push(": ");
            let normalizer = match (fact.role, fact.value) {
                (PropertyRole::Class, _) => Some(self.helpers.class),
                (PropertyRole::Style, value) if value.is_dynamic() => Some(self.helpers.style),
                _ => None,
            };
            if let Some(helper_id) = normalizer {
                spell(&mut self.writer, self.vocabulary, helper_id);
                normalizers |= if helper_id == self.helpers.class {
                    1
                } else {
                    2
                };
                self.writer.push("(");
            }
            self.expression(id, value)?;
            if normalizer.is_some() {
                self.writer.push(")");
            }
            index += 1;
        }
        if index == 0 {
            return Err(self.error(owner, DomErrorKind::InvalidGrouping));
        }
        if multiline {
            self.writer.deindent();
            self.writer.newline();
            self.writer.push("}");
        } else {
            self.writer.push(" }");
        }
        Ok(normalizers)
    }

    fn property_start(&mut self, index: usize, multiline: bool) {
        if index != 0 {
            self.writer.push(if multiline { "," } else { ", " });
        }
        if multiline {
            self.writer.newline();
        }
    }

    fn patch_flag(&mut self, flag: u32) {
        self.writer.push(", ");
        number(&mut self.writer, flag);
        self.writer.push(" /* ");
        let mut first = true;
        for (bit, name) in [(1, "TEXT"), (2, "CLASS"), (4, "STYLE"), (8, "PROPS")] {
            if flag & bit == 0 {
                continue;
            }
            if !first {
                self.writer.push(", ");
            }
            first = false;
            self.writer.push(name);
        }
        self.writer.push(" */");
    }
}

// This is output shape only. Eligibility, binding access and patch demands
// remain L3 facts. Inspect only the retained root: no trimming or AST walk.
fn inline_value(value: ExprRef<'_>) -> bool {
    let ExprRef::Js(expression) = value else {
        return false;
    };
    expression.ast.is_literal()
        || (expression.ast.is_identifier_reference()
            && expression
                .ast
                .get_identifier_reference()
                .is_some_and(|identifier| expression.source == identifier.name.as_str()))
}

//! L2 indexes consumed while projecting the lossless L1 structure.

use vize_l2::expr::ExprRef;
use vize_l2::op::{BindingOp, DynamicName, Op, Region};
use vize_relief::Namespace;

#[derive(Clone, Copy)]
pub(super) struct ElementIdentity {
    pub(super) start: u32,
    pub(super) namespace: Namespace,
}

pub(super) fn collect_region<'a>(
    region: &Region<'a>,
    expressions: &mut std::vec::Vec<ExprRef<'a>>,
    elements: &mut std::vec::Vec<ElementIdentity>,
) {
    for op in &region.ops {
        match op {
            Op::Element(element) => {
                elements.push(ElementIdentity {
                    start: element.span.start,
                    namespace: match element.namespace {
                        vize_l2::op::Namespace::Html => Namespace::Html,
                        vize_l2::op::Namespace::Svg => Namespace::Svg,
                        vize_l2::op::Namespace::MathMl => Namespace::MathMl,
                    },
                });
                collect_bindings(&element.bindings, expressions);
                collect_region(&element.children, expressions, elements);
            }
            Op::Component(component) => {
                elements.push(ElementIdentity {
                    start: component.span.start,
                    namespace: Namespace::Html,
                });
                collect_bindings(&component.bindings, expressions);
                collect_region(&component.children, expressions, elements);
            }
            Op::Interpolation(interpolation) | Op::JsxChildValue(interpolation) => {
                expressions.push(interpolation.expression);
            }
            Op::If(if_op) => {
                for branch in &if_op.branches {
                    expressions.extend(branch.condition);
                    collect_region(&branch.region, expressions, elements);
                }
            }
            Op::For(for_op) => {
                expressions.extend([for_op.binding.source, for_op.binding.value]);
                expressions.extend(for_op.binding.key);
                expressions.extend(for_op.binding.index);
                collect_region(&for_op.region, expressions, elements);
            }
            Op::OriginalFor(for_op) => collect_region(&for_op.region, expressions, elements),
            Op::Slot(slot) => {
                collect_dynamic_name(slot.name, expressions);
                collect_bindings(&slot.bindings, expressions);
                collect_region(&slot.fallback, expressions, elements);
            }
            Op::Text(_) | Op::Comment(_) => {}
        }
    }
}

fn collect_bindings<'a>(bindings: &[BindingOp<'a>], expressions: &mut std::vec::Vec<ExprRef<'a>>) {
    for binding in bindings {
        match binding {
            BindingOp::Bind(binding) => {
                binding
                    .name
                    .into_iter()
                    .for_each(|name| collect_dynamic_name(name, expressions));
                expressions.extend(binding.value);
            }
            BindingOp::On(binding) => {
                binding
                    .name
                    .into_iter()
                    .for_each(|name| collect_dynamic_name(name, expressions));
                expressions.extend(binding.expression());
            }
            BindingOp::Model(model) => {
                expressions.extend([model.contract.read, model.contract.write]);
                model
                    .argument
                    .into_iter()
                    .for_each(|name| collect_dynamic_name(name, expressions));
            }
            BindingOp::SlotContent(slot) => {
                slot.name
                    .into_iter()
                    .for_each(|name| collect_dynamic_name(name, expressions));
                expressions.extend(slot.params);
            }
            BindingOp::SlotsObject(slots) => expressions.push(slots.value),
            BindingOp::VueDirective(directive) => {
                directive
                    .argument
                    .into_iter()
                    .for_each(|name| collect_dynamic_name(name, expressions));
                expressions.extend(directive.value);
            }
            BindingOp::VueCssBind(binding) => expressions.push(binding.value),
            BindingOp::VueSync(binding) => expressions.push(binding.value),
            BindingOp::VueSlotScope(binding) => expressions.extend(binding.params),
            BindingOp::VueMemo(binding) => expressions.push(binding.value),
            BindingOp::VueShow(binding) => expressions.push(binding.value),
            BindingOp::VueHtml(binding) => expressions.extend(binding.value),
            BindingOp::VueText(binding) => expressions.extend(binding.value),
            BindingOp::VueOnce(_) | BindingOp::VueCloak(_) => {}
        }
    }
}

fn collect_dynamic_name<'a>(name: DynamicName<'a>, expressions: &mut std::vec::Vec<ExprRef<'a>>) {
    if let DynamicName::Dynamic(expression) = name {
        expressions.push(expression);
    }
}

//! The post-order static lattice and inherited complexity depth over one PageWalk.

use super::{
    ChildLevel, Contribution, RegionSummary, StaticFacts, children_ns, component_facts,
    element_contribution, element_facts, slot_props_static,
};
use crate::analysis::{static_facts::remarks, template_complexity as complexity};
use crate::op::{DynamicName, Namespace, Op, Region};
use crate::provenance::ProvenanceSink;
use crate::walk::{PageWalk, assert_accounting};
use vize_l0::ensure_sufficient_stack;
use vize_l0::pass::RemarkSink;
use vize_l0::{Span, id::NodeId};
mod owners;
pub use owners::OwnerFacts;

/// Compiler and tooling observations from a single immutable L2 artifact.
pub struct TemplateFacts {
    /// Owner facts indexed by the artifact's original page ids.
    pub static_facts: OwnerFacts,
    /// The same ordered complexity breakdown that the lint and Doctor tools read.
    pub complexity: complexity::ComplexityFacts,
    /// Number of region operations visited, excluding their attached bindings.
    pub op_visits: u32,
}

/// Analyze one artifact, retaining its static provenance and remark order.
///
/// A host loop's empty arm lives outside the repeated body, at the enclosing depth.
///
/// # Panics
///
/// A mismatched node count is a producer error under the L2 page-id contract.
#[must_use]
pub fn run(
    region: &Region<'_>,
    node_count: u32,
    provenance: &mut impl ProvenanceSink,
    remarks: &mut impl RemarkSink,
) -> TemplateFacts {
    let mut pass = Walk {
        ids: PageWalk::new(),
        static_facts: OwnerFacts::default(),
        complexity: complexity::ComplexityFacts::empty(),
        provenance,
        remarks,
    };
    pass.region(&region.ops, Namespace::Html, 0);
    assert_accounting(&pass.ids, node_count, "hoist-static+template-complexity");
    pass.complexity
        .contributions
        .sort_unstable_by_key(complexity::Contribution::sort_key);
    TemplateFacts {
        static_facts: pass.static_facts,
        complexity: pass.complexity,
        op_visits: pass.ids.visits(),
    }
}

struct Walk<'s, P, R> {
    ids: PageWalk,
    static_facts: OwnerFacts,
    complexity: complexity::ComplexityFacts,
    provenance: &'s mut P,
    remarks: &'s mut R,
}

impl<P: ProvenanceSink, R: RemarkSink> Walk<'_, P, R> {
    fn publish(&mut self, id: Option<NodeId>, owner: &str, span: Span, fact: StaticFacts) {
        if let Some(id) = id {
            let provenance = &mut self.provenance;
            super::publication::record!(provenance, id, owner, span, fact);
            self.static_facts.insert(id, fact);
        }
    }

    fn region(&mut self, ops: &[Op<'_>], ns: Namespace, depth: u32) -> RegionSummary {
        let mut summary = RegionSummary::empty();
        if ops.is_empty() {
            return summary;
        }
        self.complexity.max_nesting = self.complexity.max_nesting.max(depth);
        ensure_sufficient_stack(|| {
            for op in ops {
                let contribution = self.op(op, ns, depth);
                if R::ENABLED {
                    summary.note_blocker(&contribution, op.mnemonic());
                }
                summary.absorb(&contribution);
            }
        });
        summary
    }

    fn op(&mut self, op: &Op<'_>, ns: Namespace, depth: u32) -> Contribution {
        let id = self.ids.mint();
        let dynamic = Contribution {
            level: ChildLevel::Dynamic,
            nested: false,
            native: false,
        };
        match op {
            Op::Element(element) => {
                let inner = complexity::owner_bindings(
                    &mut self.ids,
                    &element.bindings,
                    depth,
                    &mut self.complexity,
                );
                let children = self.region(
                    &element.children.ops,
                    children_ns(element.namespace, element.tag),
                    inner,
                );
                let fact = element_facts(element, &children);
                self.publish(id, "ui.element", element.span, fact);
                if R::ENABLED {
                    remarks::element(self.remarks, element, fact, &children);
                }
                element_contribution(element, fact)
            }
            Op::Component(component) => {
                let inner = complexity::owner_bindings(
                    &mut self.ids,
                    &component.bindings,
                    depth,
                    &mut self.complexity,
                );
                let children = self.region(&component.children.ops, ns, inner);
                let fact = component_facts(component, ns, &children);
                self.publish(id, "ui.component", component.span, fact);
                if R::ENABLED {
                    remarks::component(self.remarks, component, fact);
                }
                dynamic
            }
            Op::Text(_) => Contribution {
                level: ChildLevel::Static,
                nested: true,
                native: true,
            },
            Op::Comment(_) => Contribution {
                native: true,
                ..dynamic
            },
            Op::Interpolation(value) => {
                complexity::score(value.expression, id, depth, &mut self.complexity);
                Contribution {
                    level: ChildLevel::DynamicText,
                    nested: true,
                    native: true,
                }
            }
            Op::ChildValue(value) => {
                complexity::score(value.expression, id, depth, &mut self.complexity);
                dynamic
            }
            Op::If(test) => {
                for (index, branch) in test.branches.iter().enumerate() {
                    complexity::score_branch(branch, index, id, depth, &mut self.complexity);
                    self.region(&branch.region.ops, ns, depth + 1);
                }
                dynamic
            }
            Op::And(and) => {
                complexity::score_structure(
                    and.left.span(),
                    complexity::DecisionKind::If,
                    id,
                    depth,
                    &mut self.complexity,
                );
                complexity::score(and.left, id, depth, &mut self.complexity);
                self.region(&and.region.ops, ns, depth + 1);
                dynamic
            }
            Op::For(iteration) => {
                let source = iteration.binding.source;
                complexity::score_structure(
                    source.span(),
                    complexity::DecisionKind::For,
                    id,
                    depth,
                    &mut self.complexity,
                );
                complexity::score(source, id, depth, &mut self.complexity);
                self.region(&iteration.region.ops, ns, depth + 1);
                if let Some(empty) = iteration.empty_region() {
                    self.region(&empty.ops, ns, depth);
                }
                dynamic
            }
            Op::OriginalFor(iteration) => {
                complexity::score_structure(
                    iteration.span,
                    complexity::DecisionKind::For,
                    id,
                    depth,
                    &mut self.complexity,
                );
                self.region(&iteration.region.ops, ns, depth + 1);
                dynamic
            }
            Op::Slot(slot) => {
                if let DynamicName::Dynamic(name) = slot.name {
                    complexity::score(name, id, depth, &mut self.complexity);
                }
                let _ = complexity::owner_bindings(
                    &mut self.ids,
                    &slot.bindings,
                    depth,
                    &mut self.complexity,
                );
                self.region(&slot.fallback.ops, ns, depth);
                Contribution {
                    nested: slot_props_static(slot),
                    ..dynamic
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        analysis::{static_facts, template, template_complexity},
        artifact::Builder,
        expr::{ExprRef, JsExpr},
        provenance::NoProvenance,
    };
    use vize_l0::{Allocator, Span, pass::NoRemarks};

    #[test]
    fn jsx_and_and_child_value_keep_their_original_decisions_and_ids() {
        let arena = Allocator::default();
        let source = "a && (b ? c : d)";
        let left = ExprRef::Js(JsExpr::parse_in(&arena, "a", Span::new(0, 1)).unwrap());
        let value = ExprRef::Js(JsExpr::parse_in(&arena, "b ? c : d", Span::new(6, 15)).unwrap());
        let mut builder = Builder::new(&arena, source).unwrap();
        builder
            .region()
            .and(left, Span::new(0, 16), |region, _| {
                region.child_value(value, Span::new(6, 15)).unwrap();
            })
            .unwrap();
        let artifact = builder.finish().unwrap();
        let joint = template::run(
            artifact.root(),
            artifact.node_count(),
            &mut NoProvenance,
            &mut NoRemarks,
        );
        assert_eq!(joint.op_visits, 2);
        assert_eq!(
            joint.complexity,
            template_complexity::run(artifact.root(), artifact.node_count())
        );
        let original = static_facts::run(artifact.root(), artifact.node_count(), &mut NoProvenance);
        assert_eq!(
            joint.static_facts.sorted_entries(),
            original.sorted_entries()
        );
        let rows = &joint.complexity.contributions;
        assert_eq!((rows[0].op.unwrap().index(), rows[0].nesting), (0, 0));
        assert_eq!((rows[1].op.unwrap().index(), rows[1].nesting), (1, 1));
    }
}

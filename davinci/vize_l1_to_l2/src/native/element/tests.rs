use crate::native::{NativeHole, NativeHoleKind, lower_component_native};
use alloc::vec::Vec;
use vize_l0::{Allocator, Span, String, cstr};
use vize_l1::embed::Lang;
use vize_l2::op::Op;

fn ignored_head() -> String {
    cstr!("v-bind:[{}{}]", "([".repeat(33), "])".repeat(33))
}

fn raw_owner(attributes: &str) -> String {
    cstr!("<div {attributes}><Child :x=\"bad + /* raw */\">{{{{raw}}}}</Child></div>")
}

fn check_refused_owner(attributes: &str) -> usize {
    let owner = raw_owner(attributes);
    let source = cstr!("{owner}<p>ok</p>");
    let allocator = Allocator::default();
    let lowered = lower_component_native(&allocator, &source, Lang::Js).unwrap();
    assert_eq!(lowered.component.errors.len(), 0);
    assert_eq!(lowered.component.unsupported.len(), 0);
    assert_eq!(vize_l1::check_fidelity(&lowered.component.tree), Ok(()));
    let mut rendered = String::default();
    vize_l1::render(&lowered.component.tree, &mut |piece| {
        rendered.push_str(piece)
    });
    assert_eq!(rendered, source);
    let Some(vize_l1::SurfaceChild::Element(carrier)) = lowered.component.tree.children.first()
    else {
        panic!("original verbatim owner");
    };
    assert!(carrier.open.is_verbatim());
    assert_eq!(lowered.artifact.source(), source);
    assert_eq!(
        lowered.holes.as_slice(),
        &[NativeHole {
            span: Span::new(0, owner.len() as u32),
            kind: NativeHoleKind::PreCarrier,
        }]
    );
    assert_eq!(lowered.diagnostics.len(), 1);
    assert_eq!(lowered.embeds.len(), 0);
    assert_eq!(lowered.rejected_syntax.len(), 0);
    assert!(!lowered.is_supported());
    assert_eq!(lowered.artifact.node_count(), 2);
    let Some(Op::Element(sibling)) = lowered.artifact.root().ops.first() else {
        panic!("unaffected sibling");
    };
    assert_eq!(sibling.tag, "p");
    let mut visited = 0;
    lowered
        .artifact
        .visit_nodes(&mut |id, _| {
            assert!(lowered.artifact.contains_node(id));
            visited += 1;
        })
        .unwrap();
    assert_eq!(visited, 2);
    let records: Vec<_> = lowered
        .artifact
        .provenance()
        .iter()
        .filter(|record| record.rule.as_str() == "native.unsupported")
        .collect();
    let [record] = records.as_slice() else {
        panic!("one source refusal record");
    };
    assert_eq!(record.node, None);
    assert_eq!(record.before.as_str(), owner);
    assert_eq!(record.span, lowered.holes.first().unwrap().span);
    allocator.allocated_bytes()
}

#[test]
fn ignored_overbudget_head_before_pre_never_creates_false_hole_diagnostic_or_record() {
    check_refused_owner(&cstr!("{} v-pre", ignored_head()));
}

#[test]
fn ignored_overbudget_head_after_pre_never_creates_false_hole_diagnostic_or_record() {
    check_refused_owner(&cstr!("v-pre {}", ignored_head()));
}

#[test]
fn ignored_static_value_before_pre_never_allocates_discarded_entity_payload() {
    let literal = check_refused_owner("title=\"xxxxx\" v-pre");
    let encoded = check_refused_owner("title=\"&amp;\" v-pre");
    // Equal-length authored values already require identical native L1 storage.
    // This is an arena-only law, not a whole heap-allocation budget claim.
    assert_eq!(encoded, literal);
}

#[test]
fn ignored_static_value_after_pre_never_allocates_discarded_entity_payload() {
    let literal = check_refused_owner("v-pre title=\"xxxxx\"");
    let encoded = check_refused_owner("v-pre title=\"&amp;\"");
    assert_eq!(encoded, literal);
}

#[test]
fn prior_tag_admission_facts_diagnostics_and_records_survive_later_pre() {
    let earlier = cstr!("<p {}></p>", ignored_head());
    let owner = raw_owner("v-pre");
    let source = cstr!("{earlier}{owner}<p>ok</p>");
    let allocator = Allocator::default();
    let lowered = lower_component_native(&allocator, &source, Lang::Js).unwrap();
    assert_eq!(lowered.component.unsupported.len(), 1);
    let facts: Vec<_> = lowered.holes.iter().map(|hole| hole.kind).collect();
    assert_eq!(
        facts.as_slice(),
        &[
            NativeHoleKind::DirectiveAdmission(vize_l1::markup::DirectiveNameError::NestingLimit),
            NativeHoleKind::DirectiveSyntax,
            NativeHoleKind::PreCarrier,
        ]
    );
    assert_eq!(lowered.diagnostics.len(), 3);
    assert_eq!(
        lowered
            .artifact
            .provenance()
            .iter()
            .filter(|record| record.rule.as_str() == "native.unsupported")
            .count(),
        3
    );
    assert_eq!(lowered.artifact.node_count(), 3);
    assert_eq!(lowered.embeds.len(), 0);
    assert_eq!(lowered.rejected_syntax.len(), 0);
    assert_eq!(
        source.get(
            lowered.holes.last().unwrap().span.start as usize
                ..lowered.holes.last().unwrap().span.end as usize
        ),
        Some(owner.as_str())
    );
}

#[test]
fn upstream_admitted_full_pre_arguments_and_modifiers_use_the_original_control() {
    for control in ["v-pre.foo", "v-pre:arg", "v-pre:[key]"] {
        check_refused_owner(control);
    }
}

#[test]
fn shorthand_pre_and_refused_dynamic_head_never_invent_a_verbatim_owner() {
    let malformed = cstr!("v-pre:[{}{}]", "([".repeat(33), "])".repeat(33));
    for head in ["@pre", malformed.as_str(), "v-pre[broken"] {
        let source = cstr!("<div {head}>{{{{value}}}}</div>");
        let allocator = Allocator::default();
        let lowered = lower_component_native(&allocator, &source, Lang::Js).unwrap();
        let Some(vize_l1::SurfaceChild::Element(carrier)) = lowered.component.tree.children.first()
        else {
            panic!("non-control source owner");
        };
        assert!(!carrier.open.is_verbatim());
        assert_eq!(
            lowered
                .holes
                .iter()
                .filter(|hole| hole.kind == NativeHoleKind::PreCarrier)
                .count(),
            0
        );
        assert_eq!(lowered.embeds.len(), 1);
        assert_eq!(lowered.rejected_syntax.len(), 0);
        assert_eq!(lowered.artifact.node_count(), 2);
    }
}

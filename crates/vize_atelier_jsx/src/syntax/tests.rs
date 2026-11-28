use super::{SyntaxLowerOutput, SyntaxRoot, lower_with_options};
use crate::lower::BabelLoweringOptions;
use crate::{JsxLang, LowerOutput, LoweringOptions};
use vize_l0::{Allocator, Span};
use vize_l2::op::{ComponentOp, ElementOp, Op};

fn options(native: bool) -> LoweringOptions<'static> {
    LoweringOptions {
        babel: BabelLoweringOptions {
            vdom_lane: native,
            ..BabelLoweringOptions::default()
        },
        ..LoweringOptions::default()
    }
}

fn produced<'a>(arena: &'a Allocator, source: &'a str, native: bool) -> SyntaxLowerOutput<'a> {
    lower_with_options(arena, arena.as_oxc(), source, JsxLang::Tsx, options(native))
}

fn component<'o, 'a>(root: SyntaxRoot<'o, 'a>) -> &'o ComponentOp<'a> {
    let [Op::Component(component)] = root.lowered().l2.as_ref().unwrap().root.ops.as_slice() else {
        panic!("expected one component operation");
    };
    component
}

fn slot<'o, 'a>(root: SyntaxRoot<'o, 'a>) -> &'o ElementOp<'a> {
    let [Op::Element(slot)] = component(root).children.ops.as_slice() else {
        panic!("expected one synthesized slot template");
    };
    slot
}

const SOURCE: &str = concat!(
    "const Bound = () => null;\n",
    "const A = () => <Comp v-slots={{ head: () => <div><span/></div> }} />;\n",
    "const B = () => <component is={tag}/>;\n",
    "const C = () => <Bound/>;\n",
    "const D = () => <Lib.Member/>;\n",
);

#[test]
fn both_actual_producers_retain_original_slot_range_and_intrinsic_classification() {
    for native in [false, true] {
        let arena = Allocator::new();
        let output = produced(&arena, SOURCE, native);
        let roots: Vec<_> = output.roots().collect();
        assert_eq!(roots.len(), 4);
        assert!(roots.iter().all(|root| root.has_syntax()));
        assert!(
            roots
                .iter()
                .all(|root| root.lowered().l2.as_ref().unwrap().native == native)
        );
        let slot_root = roots[0];
        let slot = slot(slot_root);
        let start = SOURCE.find("() => <div>").unwrap() as u32;
        let end = (SOURCE.find("</div>").unwrap() + "</div>".len()) as u32;
        assert_eq!(
            slot_root.slot_diagnostic_range(slot),
            Some(Span::new(start, end))
        );
        assert!(
            slot.span.start < start,
            "the semantic range still includes the slot key"
        );
        assert!(!slot_root.component_was_intrinsic(component(slot_root)));
        assert!(roots[1].component_was_intrinsic(component(roots[1])));
        assert_eq!(component(roots[1]).name, "component");
        for root in &roots[2..] {
            assert_eq!(component(*root).name, "component");
            assert!(!root.component_was_intrinsic(component(*root)));
        }
    }
}

#[test]
fn equal_source_buffers_and_equal_spans_do_not_transfer_fact_ownership() {
    for native in [false, true] {
        let arena = Allocator::new();
        let original = produced(&arena, SOURCE, native);
        let foreign = produced(&arena, SOURCE, native);
        let original: Vec<_> = original.roots().collect();
        let foreign: Vec<_> = foreign.roots().collect();
        assert!(core::ptr::eq(original[0].source(), foreign[0].source()));
        assert_eq!(slot(original[0]).span, slot(foreign[0]).span);
        assert_eq!(component(original[1]).span, component(foreign[1]).span);
        assert_eq!(original[0].slot_diagnostic_range(slot(foreign[0])), None);
        assert!(!original[1].component_was_intrinsic(component(foreign[1])));
        assert_eq!(foreign[0].slot_diagnostic_range(slot(original[0])), None);
        assert!(!foreign[1].component_was_intrinsic(component(original[1])));
    }
}

#[test]
fn whole_root_refusal_discards_partial_facts_without_shifting_the_next_root() {
    let source = "const A = () => <Comp><component is={tag}/><div v-if={ok}/></Comp>; const B = () => <component is={other}/>;";
    for native in [false, true] {
        let arena = Allocator::new();
        let output = produced(&arena, source, native);
        let roots: Vec<_> = output.roots().collect();
        assert_eq!(roots.len(), 2);
        assert!(roots[0].lowered().l2.is_err());
        assert!(!roots[0].has_syntax());
        assert!(roots[1].has_syntax());
        assert!(roots[1].component_was_intrinsic(component(roots[1])));
    }
}

#[test]
fn retained_typecheck_roots_keep_their_own_fact_packet_and_production_order() {
    let source = "const A = () => <div>{fn(<component is={tag}/>)}</div>; const B = () => <component is={other}/>;";
    let arena = Allocator::new();
    let output = lower_with_options(
        &arena,
        arena.as_oxc(),
        source,
        JsxLang::Tsx,
        LoweringOptions {
            preserve_slot_parameter_types: true,
            ..options(true)
        },
    );
    let roots: Vec<_> = output.roots().collect();
    assert_eq!(roots.len(), 3);
    for root in &roots[1..] {
        assert!(root.has_syntax());
        assert!(root.component_was_intrinsic(component(*root)));
    }
    assert!(component(roots[1]).span.start < component(roots[2]).span.start);
    drop(roots);
    let original = crate::lower_source_with_options(
        &arena,
        arena.as_oxc(),
        source,
        JsxLang::Tsx,
        LoweringOptions {
            preserve_slot_parameter_types: true,
            ..options(true)
        },
    )
    .0;
    assert_eq!(complete(&output.into_output()), complete(&original));
}

fn complete(output: &LowerOutput<'_>) -> String {
    let roots: Vec<_> = output
        .roots
        .iter()
        .map(|root| {
            format!(
                "{:?}",
                (
                    &root.root,
                    &root.l2,
                    root.vdom_value_children,
                    root.mode,
                    &root.component_name,
                    &root.component_setup,
                    &root.scoped_css,
                    &root.scoped_style_exprs,
                )
            )
        })
        .collect();
    format!(
        "{roots:?} {:?} {:?} {:?}",
        output.diagnostics,
        output.analysis.bindings,
        output.analysis.semantic_snapshot(),
    )
}

fn assert_output_parity(source: &str, lang: JsxLang) {
    for native in [false, true] {
        let arena = Allocator::new();
        let original =
            crate::lower_source_with_options(&arena, arena.as_oxc(), source, lang, options(native))
                .0;
        let observed = lower_with_options(&arena, arena.as_oxc(), source, lang, options(native));
        assert_eq!(observed.roots().len(), original.roots.len());
        let observed = observed.into_output();
        assert_eq!(
            complete(&observed),
            complete(&original),
            "native={native}: {source}"
        );
    }
}

#[test]
fn syntax_observation_preserves_every_original_root_and_full_lowering_packet() {
    for source in [
        SOURCE,
        "const A = () => <Comp render={<component is={tag}/>}/>; const B = () => <div/>;",
        "const A = () => <><div v-if={ok}/><span v-else/></>; const B = () => <component is={tag}/>;",
        "const A = () => <Comp>{({ value }) => <div>{value}</div>}</Comp>;",
        "const A = () => <div>{/* comment */}</div>; const B = () => <span/>;",
    ] {
        assert_output_parity(source, JsxLang::Tsx);
    }
    let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/jsx");
    let mut paths: Vec<_> = std::fs::read_dir(corpus)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "jsx" || extension == "tsx")
        })
        .collect();
    paths.sort();
    assert!(paths.len() >= 15);
    for path in paths {
        let lang = if path.extension().unwrap() == "tsx" {
            JsxLang::Tsx
        } else {
            JsxLang::Jsx
        };
        assert_output_parity(&std::fs::read_to_string(path).unwrap(), lang);
    }
}

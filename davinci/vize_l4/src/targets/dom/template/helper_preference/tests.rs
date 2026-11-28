//! The fused asset collection against the walks it replaced.
//!
//! `prefer_helpers` collects the preamble's component and directive names
//! while it walks for helpers. Before that, each list had a walk of its own.
//! Those two walks are kept here as oracles, and the tests pin that the fused
//! lists equal theirs, order included, over a generated nesting corpus and
//! over the JSX `Op::And` region the component walk never entered.

use alloc::vec::Vec as StdVec;
use core::cell::Cell;

use vize_l0::pass::NoObserver;
use vize_l0::{Allocator, ensure_sufficient_stack};
use vize_l1::markup::parse;
use vize_l1_to_l2::lower::{Lowered, lower};
use vize_l1_to_l2::pass::{TransformProfile, run_dom_transform_with_profile};
use vize_l2::analysis::pass_analysis::L2Analyses;
use vize_l2::op::{AndOp, Op, Region};
use vize_l2::walk::PageWalk;

use super::super::buf::Buf;
use super::super::{component, directive};
use super::{PreferCx, prefer_helpers};

/// The component walk the emitter ran before the fusion: post-order, and
/// blind to `Op::And` regions.
fn component_oracle<'a>(region: &Region<'a>, source: &str, names: &mut StdVec<&'a str>) {
    ensure_sufficient_stack(|| {
        for op in region.ops.iter() {
            match op {
                Op::Element(element) => component_oracle(&element.children, source, names),
                Op::Component(owner) => {
                    component_oracle(&owner.children, source, names);
                    component::note_name(owner, source, names);
                }
                Op::Slot(slot) => component_oracle(&slot.fallback, source, names),
                Op::If(if_op) => {
                    for branch in if_op.branches.iter() {
                        component_oracle(&branch.region, source, names);
                    }
                }
                Op::For(for_op) => component_oracle(&for_op.region, source, names),
                Op::OriginalFor(for_op) => component_oracle(&for_op.region, source, names),
                Op::And(_)
                | Op::Text(_)
                | Op::Interpolation(_)
                | Op::ChildValue(_)
                | Op::Comment(_) => {}
            }
        }
    });
}

/// The directive walk the emitter ran before the fusion: pre-order, bindings
/// before children, through every region.
fn directive_oracle<'a>(region: &Region<'a>, names: &mut StdVec<&'a str>) {
    ensure_sufficient_stack(|| {
        for op in region.ops.iter() {
            match op {
                Op::Element(element) => {
                    directive::note_names(&element.bindings, names);
                    directive_oracle(&element.children, names);
                }
                Op::Component(owner) => {
                    directive::note_names(&owner.bindings, names);
                    directive_oracle(&owner.children, names);
                }
                Op::Slot(slot) => {
                    directive::note_names(&slot.bindings, names);
                    directive_oracle(&slot.fallback, names);
                }
                Op::If(if_op) => {
                    for branch in if_op.branches.iter() {
                        directive_oracle(&branch.region, names);
                    }
                }
                Op::And(and) => directive_oracle(&and.region, names),
                Op::For(for_op) => directive_oracle(&for_op.region, names),
                Op::OriginalFor(for_op) => directive_oracle(&for_op.region, names),
                Op::Text(_) | Op::Interpolation(_) | Op::ChildValue(_) | Op::Comment(_) => {}
            }
        }
    });
}

struct Names<'a> {
    components: StdVec<&'a str>,
    directives: StdVec<&'a str>,
}

fn oracle_names<'a>(root: &Region<'a>, source: &str) -> Names<'a> {
    let mut names = Names {
        components: StdVec::new(),
        directives: StdVec::new(),
    };
    component_oracle(root, source, &mut names.components);
    directive_oracle(root, &mut names.directives);
    names
}

fn fused_names<'a>(lowered: &Lowered<'a>, facts: &L2Analyses) -> Names<'a> {
    let mut buf = Buf::new(false);
    let cx = PreferCx {
        source: lowered.artifact.source,
        facts,
        texts: &lowered.artifact.texts,
        for_wrappers: &lowered.artifact.loop_fragments,
        bindings: None,
        inline: false,
        authored_eager_unref: Cell::new(u32::MAX),
        authored_for: Cell::new(u32::MAX),
        first_for_source_unref: Cell::new(false),
        early_transition_slot_for: Cell::new(false),
    };
    let names = prefer_helpers(&mut buf, &cx, &mut PageWalk::new(), &lowered.artifact.root);
    Names {
        components: names.components,
        directives: names.directives,
    }
}

/// Lower `source` as the DOM emitter does and return (fused, oracle) names.
fn both<'a>(allocator: &'a Allocator, source: &'a str) -> (Names<'a>, Names<'a>) {
    let (tree, errors) = parse(allocator, source);
    let mut lowered = lower(allocator, &tree, &errors);
    let facts =
        run_dom_transform_with_profile(&mut lowered, &mut NoObserver, TransformProfile::DEFAULT);
    let fused = fused_names(&lowered, &facts);
    let oracle = oracle_names(&lowered.artifact.root, source);
    (fused, oracle)
}

/// Wrappers each nesting level draws from; `{}` is the content slot.
const WRAPPERS: &[&str] = &[
    "<div>{}</div>",
    "<Comp>{}</Comp>",
    "<template v-if=\"a\">{}</template>",
    "<p v-if=\"a\"></p><span v-else>{}</span>",
    "<div v-for=\"i in xs\">{}</div>",
    "<Comp v-slot=\"{ x }\">{}</Comp>",
    "<Comp><template #named>{}</template></Comp>",
    "<slot v-focus>{}</slot>",
    "<KeepAlive>{}</KeepAlive>",
    "<component :is=\"c\">{}</component>",
    "<Transition>{}</Transition>",
];

/// Leaves carrying assets, reserved tags, repeated names and `v-slots`.
const LEAVES: &[&str] = &[
    "<Comp/>",
    "<Other v-focus/>",
    "<div v-focus v-tooltip:x.y=\"v\"/>",
    "<Comp v-focus/>",
    "<slot v-tooltip/>",
    "text {{ t }}",
    "<component :is=\"x\" v-focus/>",
    "<my-comp v-slots=\"s\"/>",
    "<MyComp/>",
    "<Suspense><Other/></Suspense>",
];

#[test]
fn fused_assets_match_the_replaced_walks_over_two_nesting_levels() {
    let mut cases = 0_usize;
    let mut with_components = 0_usize;
    let mut with_directives = 0_usize;
    for outer in WRAPPERS {
        for inner in WRAPPERS {
            for first in LEAVES {
                for second in LEAVES {
                    let body = inner.replace("{}", first) + second;
                    let source = outer.replace("{}", &body);
                    let allocator = Allocator::new();
                    let (fused, oracle) = both(&allocator, &source);
                    assert_eq!(fused.components, oracle.components, "components: {source}");
                    assert_eq!(fused.directives, oracle.directives, "directives: {source}");
                    cases += 1;
                    with_components += usize::from(!oracle.components.is_empty());
                    with_directives += usize::from(!oracle.directives.is_empty());
                }
            }
        }
    }
    assert_eq!(cases, WRAPPERS.len().pow(2) * LEAVES.len().pow(2));
    // The corpus is not vacuous: most cases name an asset of each kind.
    assert!(with_components * 2 > cases, "{with_components} of {cases}");
    assert!(with_directives * 2 > cases, "{with_directives} of {cases}");
}

#[test]
fn fused_assets_keep_first_occurrence_order() {
    let source = "<B v-y/><A v-x><B v-y/><C/></A><A/><div v-x v-z/>";
    let allocator = Allocator::new();
    let (fused, oracle) = both(&allocator, source);
    // Components resolve children first (post-order); directives resolve in
    // pre-order, an owner's bindings before its children.
    assert_eq!(fused.components, ["B", "C", "A"]);
    assert_eq!(fused.directives, ["y", "x", "z"]);
    assert_eq!(fused.components, oracle.components);
    assert_eq!(fused.directives, oracle.directives);
}

/// JSX `{cond && <A/>}` lowers to `Op::And`. Directives inside it resolve in
/// the preamble. Components inside it do not: the replaced component walk
/// never entered an `And` region, and the emitted bytes must not change.
#[test]
fn fused_assets_skip_components_only_inside_and_regions() {
    let source = "{{ ok }}<Inner v-b><Deep v-c/></Inner>";
    let allocator = Allocator::new();
    let (tree, errors) = parse(&allocator, source);
    let mut lowered = lower(&allocator, &tree, &errors);
    let facts =
        run_dom_transform_with_profile(&mut lowered, &mut NoObserver, TransformProfile::DEFAULT);

    // Before the rewrite, both components resolve.
    let plain = fused_names(&lowered, &facts);
    assert_eq!(plain.components, ["Deep", "Inner"]);
    assert_eq!(plain.directives, ["b", "c"]);

    // Move the whole root under one `And`, tested on the interpolation.
    let Some(Op::Interpolation(interpolation)) = lowered.artifact.root.ops.first() else {
        panic!("the interpolation lowers first");
    };
    let (left, span) = (interpolation.expression, interpolation.span);
    // The arena constructors take the handle (`&Allocator`) by reference.
    let arena = &allocator;
    let empty = Region {
        ops: vize_l0::Vec::new_in(&arena),
    };
    let inner = core::mem::replace(&mut lowered.artifact.root, empty);
    lowered.artifact.root.ops.push(Op::And(vize_l0::Box::new_in(
        AndOp {
            left,
            region: inner,
            span,
        },
        &arena,
    )));

    let fused = fused_names(&lowered, &L2Analyses::default());
    let oracle = oracle_names(&lowered.artifact.root, source);
    assert_eq!(fused.components, oracle.components);
    assert_eq!(fused.directives, oracle.directives);
    assert!(fused.components.is_empty(), "{:?}", fused.components);
    assert_eq!(fused.directives, ["b", "c"]);
}

//! The per-pass artifact hook (`run_transform_with_pass_hook`): it sees
//! every executed pass of the artifact-selected plan, in order, with the
//! post-pass S2 tree, and it only observes - the facts, diagnostics and
//! folio equal the hook-free `run_transform` run exactly.

use vize_davinci::folio::{Folio, FolioMode};
use vize_davinci::pass::BudgetObserver;
use vize_s0::{Allocator, String};
use vize_s1_to_s2::lower;
use vize_s1_to_s2::pass::{TransformProfile, run_transform, run_transform_with_pass_hook};
use vize_s2::folio::S2Folio;

const SOURCE: &str = r#"<Comp v-model="x"><template #a>hi</template></Comp><p>{{ y }}</p>"#;

#[test]
fn the_hook_sees_every_executed_pass_in_order_with_the_post_pass_tree() {
    let allocator = Allocator::default();
    let (tree, errors) = vize_s1::parse(&allocator, SOURCE);
    let mut lowered = lower(&allocator, &tree, &errors);
    let before = S2Folio::of(&lowered.root.ops).print_to_string(FolioMode::Full);

    let mut seen: Vec<(String, usize, usize, String)> = Vec::new();
    let mut budget = BudgetObserver::new();
    run_transform_with_pass_hook(
        &mut lowered,
        &mut budget,
        TransformProfile::DEFAULT,
        |event, lowered| {
            seen.push((
                String::from(event.desc().name),
                event.group_index,
                event.pass_index,
                S2Folio::of(&lowered.root.ops).print_to_string(FolioMode::Full),
            ));
        },
    );

    // The plan the artifact selected: slot carriers and a model binding
    // keep both mandatory barriers, and the default profile keeps the
    // optional static analysis - three passes, three walks, in the
    // selectable table's order.
    let names: Vec<(&str, usize, usize)> = seen
        .iter()
        .map(|(name, group, pass, _)| (name.as_str(), *group, *pass))
        .collect();
    assert_eq!(
        names,
        vec![("v-slot", 0, 0), ("v-model", 1, 1), ("hoist-static", 2, 2)]
    );
    assert_eq!((budget.passes, budget.walks), (3, 3));
    // Every Vue 3 pass here preserves the tree (facts go to side tables),
    // so each post-pass page is the lowering's page byte for byte.
    for (_, _, _, page) in &seen {
        assert_eq!(page, &before);
    }
}

#[test]
fn the_hook_only_observes() {
    let allocator = Allocator::default();
    let (tree, errors) = vize_s1::parse(&allocator, SOURCE);
    let mut hooked = lower(&allocator, &tree, &errors);
    let mut plain = lower(&allocator, &tree, &errors);

    let mut hooked_budget = BudgetObserver::new();
    let hooked_facts = run_transform_with_pass_hook(
        &mut hooked,
        &mut hooked_budget,
        TransformProfile::DEFAULT,
        |_, _| {},
    );
    let mut plain_budget = BudgetObserver::new();
    let plain_facts = run_transform(&mut plain, &mut plain_budget);

    assert_eq!(hooked_budget, plain_budget);
    assert_eq!(hooked.diagnostics, plain.diagnostics);
    assert_eq!(hooked.provenance, plain.provenance);
    assert_eq!(
        S2Folio::of(&hooked.root.ops).print_to_string(FolioMode::Full),
        S2Folio::of(&plain.root.ops).print_to_string(FolioMode::Full)
    );
    assert_eq!(hooked_facts.slot_facts, plain_facts.slot_facts);
    assert_eq!(hooked_facts.model_faults, plain_facts.model_faults);
    assert_eq!(hooked_facts.static_facts, plain_facts.static_facts);
    assert_eq!(hooked_facts.if_facts, plain_facts.if_facts);
    assert_eq!(hooked_facts.for_facts, plain_facts.for_facts);
    assert_eq!(hooked_facts.text_facts, plain_facts.text_facts);
}

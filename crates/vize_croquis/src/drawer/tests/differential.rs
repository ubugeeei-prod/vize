//! Davinci P1-6 differential lane — plain-suite coverage witness.
//!
//! Under `cfg(test)` the migrated helpers dual-run every retained-AST read
//! against the legacy re-parse and panic on divergence (see
//! `crate::drawer::differential`), so the whole croquis unit suite is a
//! differential run. This test plants a battery of retained slow-dispatch
//! expressions and asserts the comparators actually fired for them — a
//! `cfg` regression that silently disarmed the lane fails here. Counter
//! deltas use `>=`: the counters are process-global and other unit tests
//! run concurrently. The exact-pinned lane lives in
//! `tests/davinci_differential.rs` (own process, deterministic counts).

use super::super::{Drawer, DrawerOptions, differential};
use vize_carton::Allocator;

#[test]
fn retained_reads_are_dual_run_across_the_unit_suite() {
    let template = r#"<div :class="{ active: isActive, 'is-open': open }">
  {{ items.filter(item => item.id > 0).length / total }}
  <span :title="(value as string) + suffix">{{ /* note */ ({ a: alpha }).a }}</span>
  <component :is="targetComponent"></component>
  <component :is="shapes.current"></component>
  <li v-for="{ id, label } in entries" :key="id">{{ label }}</li>
</div>"#;

    let before = differential::stats();

    let allocator = Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert_eq!(
        errors.len(),
        0,
        "battery template must parse cleanly: {errors:?}"
    );
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_template(&root);
    let croquis = drawer.finish();
    assert_eq!(
        croquis.undefined_refs.len(),
        0,
        "no script was drawn, so undefined-ref reporting must stay off"
    );

    let after = differential::stats();
    // 3 distinct comment-free retained slow-dispatch expressions (`:class`
    // object, arrow+division interpolation, `as`-cast), 2 `<component :is>`
    // references, and 1 v-for destructure shape planted above. The
    // comment-carrying interpolation is planted as fallback-class coverage —
    // it must run the legacy path, not the comparator.
    assert!(
        after.identifier_comparisons >= before.identifier_comparisons + 3,
        "identifier dual-runs did not fire for the planted battery: {before:?} -> {after:?}"
    );
    assert!(
        after.component_reference_comparisons >= before.component_reference_comparisons + 2,
        "component-reference dual-runs did not fire for the planted battery: {before:?} -> {after:?}"
    );
    assert!(
        after.v_for_shapes > before.v_for_shapes,
        "v-for shape witness did not fire for the planted battery: {before:?} -> {after:?}"
    );
}

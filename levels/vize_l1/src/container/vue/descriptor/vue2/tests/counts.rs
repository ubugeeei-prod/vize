extern crate std;
use super::*;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn exactly_one_splitter_and_component_entry_precede_readback_and_normal_drop() {
    let arena = Allocator::default();
    let measured = hooks::Armed::new(hooks::Fault::None);
    let owner = Vue.observe_vue2_descriptor(
        &arena,
        "<template>{{ value | upper }}</template>",
        options(),
    );
    for _ in 0..3 {
        let view = owner.selected().unwrap();
        let component = view.component();
        component
            .text_for(component.children().next().unwrap())
            .unwrap();
    }
    assert_eq!(
        measured.counts(),
        hooks::Counts {
            splitters: 1,
            components: 1,
            parked: 1,
            ..hooks::Counts::default()
        }
    );
    drop(owner);
    assert_eq!(measured.counts().dropped, 1);
}

#[test]
fn every_later_refused_slot_and_profile_stops_before_the_actual_parser_entry() {
    let arena = Allocator::default();
    for suffix in [
        "<script></script>",
        "<style></style>",
        "<custom/>",
        "<template>duplicate</template>",
    ] {
        let measured = hooks::Armed::new(hooks::Fault::None);
        let source = vize_l0::cstr!("<template>{{{{ value }}}}</template>{suffix}");
        let owner = Vue.observe_vue2_descriptor(&arena, &source, options());
        assert!(owner.selected().is_err());
        assert_eq!(
            measured.counts(),
            hooks::Counts {
                splitters: 1,
                ..hooks::Counts::default()
            }
        );
        assert_eq!(owner.container().blocks.len(), 2);
        drop(owner);
    }
    for version in VueVersion::ALL.into_iter().filter(|v| *v != VueVersion::V2) {
        let measured = hooks::Armed::new(hooks::Fault::None);
        let owner = Vue.observe_vue2_descriptor(
            &arena,
            "<template>{{ value }}</template>",
            DescriptorOptions {
                version,
                ..options()
            },
        );
        assert!(owner.selected().is_err());
        assert_eq!(
            (measured.counts().splitters, measured.counts().components),
            (1, 0)
        );
    }
}

#[test]
fn caught_entry_or_after_park_unwind_never_returns_or_retries_an_owner() {
    let arena = Allocator::default();
    for fault in [hooks::Fault::ComponentEntry, hooks::Fault::AfterPark] {
        let measured = hooks::Armed::new(fault);
        let mut returned = None;
        let result = catch_unwind(AssertUnwindSafe(|| {
            returned = Some(Vue.observe_vue2_descriptor(
                &arena,
                "<template>{{ x + }}</template>",
                options(),
            ));
        }));
        assert!(result.is_err() && returned.is_none());
        let counts = measured.counts();
        assert_eq!(
            (counts.splitters, counts.components, counts.dropped),
            (1, 1, 1)
        );
        assert_eq!(counts.parked, usize::from(fault == hooks::Fault::AfterPark));
        if fault == hooks::Fault::AfterPark {
            assert!(counts.parked_diagnostics > 0);
        }
        assert_eq!(measured.counts(), counts);
    }
}

#[test]
fn original_entry_measurements_are_isolated_from_other_test_threads() {
    let measured = hooks::Armed::new(hooks::Fault::None);
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            let arena = Allocator::default();
            let measured = hooks::Armed::new(hooks::Fault::None);
            let owner =
                Vue.observe_vue2_descriptor(&arena, "<template>{{ value }}</template>", options());
            assert!(owner.selected().is_ok());
            drop(owner);
            assert_eq!(
                (
                    measured.counts().splitters,
                    measured.counts().components,
                    measured.counts().dropped
                ),
                (1, 1, 1)
            );
        });
        worker.join().unwrap();
    });
    assert_eq!(measured.counts(), hooks::Counts::default());
}

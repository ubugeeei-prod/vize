use super::{Helper, HelperSet, Runtime, vocabulary, vocabulary_for};

#[test]
fn every_export_round_trips_and_all_tables_fit_the_compact_helper_set() {
    for runtime in [
        Runtime::VueDom,
        Runtime::VueServerRenderer,
        Runtime::VueVapor,
    ] {
        let vocabulary = vocabulary(runtime);
        let mut names = Vec::new();
        for module in vocabulary.modules {
            for &name in module.names {
                assert!(
                    !names.contains(&name),
                    "duplicate {runtime:?} export {name}"
                );
                let helper = vocabulary.helper(name).unwrap();
                assert_eq!(usize::from(helper.index()), names.len());
                assert_eq!(vocabulary.export(helper), Some((module.module, name)));
                names.push(name);
            }
        }
        assert!(names.len() <= Helper::LIMIT);
        assert_eq!(vocabulary.name(Helper::from_index(127).unwrap()), None);
    }
}

#[test]
fn exact_version_selection_has_no_cross_runtime_or_old_version_fallback() {
    for (runtime, version, count) in [
        (Runtime::VueDom, "3.5.35", 57),
        (Runtime::VueServerRenderer, "3.5.35", 76),
        (Runtime::VueVapor, "3.6.0-rc.9", 74),
    ] {
        let selected = vocabulary_for(runtime, version).unwrap();
        assert!(core::ptr::eq(selected, vocabulary(runtime)));
        assert_eq!(
            selected
                .modules
                .iter()
                .map(|module| module.names.len())
                .sum::<usize>(),
            count
        );
    }
    for runtime in [
        Runtime::VueDom,
        Runtime::VueServerRenderer,
        Runtime::VueVapor,
    ] {
        for version in ["0.12.16", "1.0.28", "2.7.16", "3.5", "3.6.0-rc.6", "3.6.0"] {
            assert!(vocabulary_for(runtime, version).is_none());
        }
    }
    assert!(vocabulary_for(Runtime::VueVapor, "3.5.35").is_none());
    assert!(vocabulary_for(Runtime::VueDom, "3.6.0-rc.9").is_none());
}

#[test]
fn unsupported_old_generator_helpers_are_not_admitted_as_current_exports() {
    assert!(
        vocabulary(Runtime::VueDom)
            .helper("resolveFilter")
            .is_none()
    );
    let vapor = vocabulary(Runtime::VueVapor);
    assert!(vapor.helper("prepend").is_none());
    assert!(vapor.helper("withVaporModifiers").is_some());
    assert!(vapor.helper("withVaporKeys").is_some());
    assert!(vapor.helper("ssrInterpolate").is_none());
    let server = vocabulary(Runtime::VueServerRenderer);
    assert_eq!(server.helper("ssrInterpolate").unwrap().index(), 0);
    assert_eq!(
        server.export(server.helper("unref").unwrap()),
        Some(("vue", "unref"))
    );
}

#[test]
fn helper_bits_include_the_boundary_and_union_preserves_first_use() {
    assert!(Helper::from_index(128).is_none());
    let first = Helper::from_index(127).unwrap();
    let second = Helper::from_index(0).unwrap();
    let third = Helper::from_index(65).unwrap();
    let mut body = HelperSet::default();
    assert!(body.insert(first));
    assert!(body.insert(second));
    assert!(!body.insert(first));
    let mut fragment = HelperSet::default();
    fragment.insert(second);
    fragment.insert(third);
    fragment.insert(first);
    body.extend(&fragment);
    assert_eq!(body.in_use_order(), &[first, second, third]);
    assert!(body.contains(first));
}

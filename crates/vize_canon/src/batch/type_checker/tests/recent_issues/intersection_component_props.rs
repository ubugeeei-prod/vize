//! #13689 uses real Vue and SFC intersections in both orders, without Nuxt.
use super::super::{create_project_case, resolve_test_tsgo_binary};
use crate::batch::{BatchTypeChecker, TypeChecker};

#[test]
fn intersection_props_match_the_vue_tsc_negative_controls() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/typechecker/component-intersection-props-13689");
    let names = [
        "Authored.vue",
        "Exposed.vue",
        "ExposedAny.vue",
        "ExposedUnknown.vue",
        "ExposedNever.vue",
        "StaticNever.vue",
        "StaticOpen.vue",
        "StaticLiteral.vue",
        "StaticPattern.vue",
        "OwnLiteral.vue",
        "OwnPattern.vue",
        "OwnIndex.vue",
        "Child.vue",
        "Empty.vue",
        "Emit.vue",
        "Fresh.vue",
        "Second.vue",
        "Generic.vue",
        "lazy.ts",
        "Valid.vue",
        "Invalid.vue",
        "controls.ts",
        "tsconfig.json",
    ];
    let sources: Vec<_> = names
        .iter()
        .map(|name| std::fs::read_to_string(fixture.join(format!("{name}.txt"))).unwrap())
        .collect();
    let files: Vec<_> = names
        .iter()
        .zip(&sources)
        .map(|(name, source)| (*name, source.as_str()))
        .collect();
    let root = create_project_case("component-intersection-props-13689", &files);
    let mut checker = BatchTypeChecker::new(&root).unwrap();
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    let mut actual: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            assert_eq!(diagnostic.file, root.join("Invalid.vue"));
            (diagnostic.line + 1, diagnostic.column + 1, diagnostic.code)
        })
        .collect();
    actual.sort();
    std::fs::remove_dir_all(&root).unwrap();
    assert_eq!(
        actual,
        [
            (6, 4, Some(2345)),
            (7, 11, Some(2322)),
            (8, 24, Some(2322)),
            (9, 24, Some(2353)),
            (10, 4, Some(2345)),
            (11, 33, Some(2322)),
            (12, 24, Some(2353)),
            (13, 25, Some(2322)),
            (14, 15, Some(2322)),
            (15, 15, Some(2353)),
        ],
        "{:#?}",
        result.diagnostics
    );
}

#[test]
fn component_identity_keeps_global_instance_raw_metadata() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/typechecker/component-intersection-props-13689");
    let globals = fixture.join("globals");
    let component = std::fs::read_to_string(globals.join("Augmented.vue.txt")).unwrap();
    let common = std::fs::read_to_string(globals.join("controls.ts.txt")).unwrap();
    let config = std::fs::read_to_string(fixture.join("tsconfig.json.txt")).unwrap();
    for kind in ["object", "any", "unknown", "never"] {
        let augmentation =
            std::fs::read_to_string(globals.join(format!("{kind}.d.ts.txt"))).unwrap();
        let mut controls = common.clone();
        if kind == "object" {
            controls.push_str(
                &std::fs::read_to_string(globals.join("object-controls.ts.txt")).unwrap(),
            );
        }
        let root = create_project_case(
            &format!("component-instance-raw-metadata-{kind}"),
            &[
                ("Augmented.vue", component.as_str()),
                ("globals.d.ts", augmentation.as_str()),
                ("controls.ts", controls.as_str()),
                ("tsconfig.json", config.as_str()),
            ],
        );
        let mut checker = BatchTypeChecker::new(&root).unwrap();
        checker.scan_project().unwrap();
        let result = checker.check_project().unwrap();
        std::fs::remove_dir_all(&root).unwrap();
        assert!(
            result.diagnostics.is_empty(),
            "{kind}: {:#?}",
            result.diagnostics
        );
    }
}

//! Runtime-sensitive patch-flag regressions reported in #10168.

use vize_atelier_core::options::CodegenMode;
#[cfg(feature = "legacy-differential")]
use vize_atelier_dom::compile_template_legacy_with_options;
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_l0::{Allocator, cstr};

#[test]
fn static_root_props_keep_text_patch_flags() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/runtime_patch_flags.json")).unwrap();
    let mut capture = Vec::new();

    for fixture in fixtures.as_array().unwrap() {
        let name = fixture["name"].as_str().unwrap();
        let source = fixture["template"].as_str().unwrap();
        let allocator = Allocator::new();
        let (_, errors, output) = compile_template_with_options(
            &allocator,
            source,
            DomCompilerOptions {
                mode: CodegenMode::Module,
                prefix_identifiers: true,
                ..Default::default()
            },
        );
        assert!(
            errors.iter().all(|error| error.is_compatibility_notice()),
            "{name}: {errors:?}",
        );
        if name == "scoped-slot-closure" {
            assert_eq!(
                output.code.matches("_: 2 /* DYNAMIC */").count(),
                1,
                "{name}: captured slot parameters require a dynamic slot object:\n{}",
                output.code,
            );
            assert_eq!(
                output
                    .code
                    .matches("1032 /* PROPS, DYNAMIC_SLOTS */")
                    .count(),
                1,
                "{name}: captured slot parameters require a DYNAMIC_SLOTS vnode:\n{}",
                output.code,
            );
        } else {
            assert_eq!(
                output.code.matches("1 /* TEXT */").count(),
                1,
                "{name}: dynamic text requires the TEXT patch flag:\n{}",
                output.code,
            );
        }
        capture.push(serde_json::json!({
            "name": name,
            "template": source,
            "code": cstr!("{}\n{}", output.preamble, output.code),
        }));
    }

    assert_eq!(capture.len(), 3);
    if let Some(path) = std::env::var_os("VIZE_RUNTIME_PATCH_FLAGS_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&capture).unwrap()).unwrap();
    }
}

#[test]
fn scoped_slot_detection_keeps_uncaptured_and_shadowed_slots_stable() {
    for (name, source) in [
        (
            "uncaptured",
            r#"<MyList><template #default="{ item }"><MyChip tone="red">fixed</MyChip></template></MyList>"#,
        ),
        (
            "shadowed-arrow",
            r#"<MyList><template #default="{ item }"><MyChip>{{ list.map(item => item.name).join(',') }}</MyChip></template></MyList>"#,
        ),
    ] {
        let allocator = Allocator::new();
        let (_, errors, output) = compile_template_with_options(
            &allocator,
            source,
            DomCompilerOptions {
                mode: CodegenMode::Module,
                prefix_identifiers: true,
                ..Default::default()
            },
        );
        assert!(
            errors.iter().all(|error| error.is_compatibility_notice()),
            "{name}: {errors:?}",
        );
        assert_eq!(
            output.code.matches("DYNAMIC_SLOTS").count(),
            0,
            "{name}: a slot that does not capture the outer parameter stays stable:\n{}",
            output.code,
        );
    }
}

#[test]
#[cfg(feature = "legacy-differential")]
fn legacy_scoped_slot_detection_reads_typescript_expressions() {
    let source = r#"<MyList><template #default="{ item }"><MyChip>{{ (item as Item).name }}</MyChip></template></MyList>"#;
    let allocator = Allocator::new();
    let (_, errors, output) =
        compile_template_legacy_with_options(&allocator, source, DomCompilerOptions::default());
    assert!(
        errors.iter().all(|error| error.is_compatibility_notice()),
        "{errors:?}",
    );
    assert_eq!(
        output.code.matches("DYNAMIC_SLOTS").count(),
        1,
        "a TypeScript assertion still closes over the scoped slot parameter:\n{}",
        output.code,
    );
}

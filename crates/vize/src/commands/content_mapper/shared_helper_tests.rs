use serde_json::{Value, json};
use vize_l0::cstr;

use super::helper_abi_tests::InstalledSdk;
use super::test_support::{
    exchange, frames, initialize_request, open_project_request, transform_request,
};

#[test]
fn qualified_vize_package_enables_shared_component_attribute_helpers() {
    let sdk = InstalledSdk::new();
    std::fs::remove_file(sdk.package.join("virtual-ts-checker.d.ts")).unwrap();
    let text = sdk.transform();
    assert!(
        text.contains("from \"vize/content-mapper-helpers\";"),
        "{text}"
    );
    assert!(!text.contains("type $ComponentNativeAttrNames ="), "{text}");
}

#[test]
fn mismatched_project_and_helper_vue_packages_keep_transforms_self_contained() {
    let sdk = InstalledSdk::new();
    let vue = sdk.config.parent().unwrap().join("node_modules/vue");
    std::fs::create_dir_all(&vue).unwrap();
    std::fs::write(vue.join("package.json"), "{}").unwrap();
    let text = sdk.transform();
    assert!(
        !text.contains("from \"vize/content-mapper-helpers\";"),
        "{text}"
    );
    assert!(text.contains("type $ComponentNativeAttrNames ="), "{text}");
}

#[test]
fn core_sharing_survives_without_the_optional_program_module() {
    let sdk = InstalledSdk::new();
    std::fs::remove_file(sdk.package.join("virtual-ts-checker.d.ts")).unwrap();
    let text = sdk.transform();
    assert!(
        text.contains("from \"vize/content-mapper-helpers\";"),
        "{text}"
    );
    assert!(!text.contains("type $ComponentInput<"), "{text}");
    assert!(text.contains("type $ComponentRawProps<"), "{text}");
    assert!(
        !text.contains("import type {} from \"vize/virtual-ts-checker\";"),
        "{text}"
    );
}

#[test]
fn legacy_or_incomplete_helper_packages_use_embedded_types() {
    for stage in 0..3 {
        let sdk = InstalledSdk::new();
        // An empty legacy surface, generated file alone, and a helper frontend
        // without its package export are insufficient ABI evidence.
        if stage < 2 {
            for extension in ["ts", "mts", "cts"] {
                std::fs::write(
                    sdk.package
                        .join(cstr!("content-mapper-helpers.d.{extension}").as_str()),
                    "export {};\n",
                )
                .unwrap();
            }
        }
        if stage == 0 {
            std::fs::remove_file(sdk.package.join("generated/content-mapper-types.d.ts")).unwrap();
        } else if stage == 2 {
            let manifest = sdk.package.join("package.json");
            let mut value: Value =
                serde_json::from_slice(&std::fs::read(&manifest).unwrap()).unwrap();
            value["exports"]
                .as_object_mut()
                .unwrap()
                .remove("./content-mapper-helpers");
            std::fs::write(manifest, serde_json::to_vec(&value).unwrap()).unwrap();
        }
        let text = sdk.transform();
        assert!(
            !text.contains("from \"vize/content-mapper-helpers\";"),
            "stage {stage}: {text}"
        );
        assert!(
            text.contains("type $ComponentInput<"),
            "stage {stage}: {text}"
        );
    }
}

#[test]
fn installed_virtual_module_points_helpers_at_the_package() {
    let sdk = InstalledSdk::new();
    let mut open = open_project_request(2, Value::Null, json!({}));
    open["params"]["configFileName"] = json!(sdk.config);
    let responses = exchange(&frames(&[
        initialize_request(),
        open,
        transform_request(3, "<template><main /></template>"),
    ]));
    let text = responses[2]["result"]["text"].as_str().unwrap();
    assert_eq!(
        text.matches("import type {} from \"vize/virtual-ts-checker\";")
            .count(),
        1,
        "{text}"
    );
    assert_eq!(text.matches("vize:virtual-ts").count(), 0, "{text}");
    assert_eq!(text.matches("type __EmitFn<").count(), 0, "{text}");
}

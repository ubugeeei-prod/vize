use serde_json::{Value, json};
use vize_l0::cstr;

use super::helper_abi_fixture::{InstalledSdk, Mode};

#[test]
fn qualified_vize_package_enables_shared_component_attribute_helpers() {
    let sdk = InstalledSdk::new();
    std::fs::remove_file(sdk.package.join("virtual-ts-checker.d.ts")).unwrap();
    sdk.assert_projection(Mode::Core);
}

#[test]
fn mismatched_project_and_helper_vue_packages_keep_transforms_self_contained() {
    let sdk = InstalledSdk::new();
    let vue = sdk.config.parent().unwrap().join("node_modules/vue");
    std::fs::create_dir_all(&vue).unwrap();
    std::fs::write(vue.join("package.json"), "{}").unwrap();
    sdk.assert_projection(Mode::Inline);
}

#[test]
fn core_sharing_survives_without_the_optional_program_module() {
    let sdk = InstalledSdk::new();
    std::fs::remove_file(sdk.package.join("virtual-ts-checker.d.ts")).unwrap();
    sdk.assert_projection_of(
        "<template><main data-test-id=\"shared\" /></template>",
        Mode::Core,
        json!({}),
    );
}

#[test]
fn legacy_or_incomplete_helper_packages_use_embedded_types() {
    for stage in 0..3 {
        let sdk = InstalledSdk::new();
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
        sdk.assert_projection(Mode::Inline);
    }
}

#[test]
fn installed_virtual_module_points_helpers_at_the_package() {
    let sdk = InstalledSdk::new();
    sdk.assert_projection_of("<template><main /></template>", Mode::Shared, json!({}));
}

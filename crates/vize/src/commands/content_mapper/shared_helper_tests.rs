use serde_json::{Value, json};

use super::test_support::{
    exchange, frames, initialize_request, open_project_request, transform_request,
};

#[test]
fn installed_vize_package_enables_shared_component_attribute_helpers() {
    let directory = tempfile::tempdir().unwrap();
    let project = directory.path().join("app");
    std::fs::create_dir_all(&project).unwrap();
    let config = project.join("tsconfig.json");
    std::fs::write(&config, "{}").unwrap();
    let package = directory.path().join("node_modules/vize");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(package.join("content-mapper-helpers.d.ts"), "export {};").unwrap();
    let vue = directory.path().join("node_modules/vue");
    std::fs::create_dir_all(&vue).unwrap();
    std::fs::write(vue.join("package.json"), "{}").unwrap();

    let mut open = open_project_request(2, Value::Null, json!({}));
    open["params"]["configFileName"] = json!(config);
    let responses = exchange(&frames(&[
        initialize_request(),
        open,
        transform_request(3, "<template><main data-test-id=\"shared\" /></template>"),
    ]));
    let text = responses[2]["result"]["text"].as_str().unwrap();
    assert_eq!(
        text.matches("from \"vize/content-mapper-helpers\";")
            .count(),
        1,
        "{text}"
    );
    assert_eq!(
        text.matches("type __VizeComponentNativeAttrNames =")
            .count(),
        0,
        "{text}"
    );
}

#[test]
fn mismatched_project_and_helper_vue_packages_keep_transforms_self_contained() {
    let directory = tempfile::tempdir().unwrap();
    let project = directory.path().join("app");
    std::fs::create_dir_all(&project).unwrap();
    let config = project.join("tsconfig.json");
    std::fs::write(&config, "{}").unwrap();
    let package = directory.path().join("node_modules/vize");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(package.join("content-mapper-helpers.d.ts"), "export {};").unwrap();
    for vue in [
        directory.path().join("node_modules/vue"),
        project.join("node_modules/vue"),
    ] {
        std::fs::create_dir_all(&vue).unwrap();
        std::fs::write(vue.join("package.json"), "{}").unwrap();
    }

    let mut open = open_project_request(2, Value::Null, json!({}));
    open["params"]["configFileName"] = json!(config);
    let responses = exchange(&frames(&[
        initialize_request(),
        open,
        transform_request(3, "<template><main data-test-id=\"inline\" /></template>"),
    ]));
    let text = responses[2]["result"]["text"].as_str().unwrap();
    assert_eq!(
        text.matches("from \"vize/content-mapper-helpers\";")
            .count(),
        0,
        "{text}"
    );
    assert_eq!(
        text.matches("type __VizeComponentNativeAttrNames =")
            .count(),
        1,
        "{text}"
    );
}

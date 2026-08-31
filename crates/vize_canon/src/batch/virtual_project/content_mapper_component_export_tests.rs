use std::path::Path;

use crate::batch::generate_vue_content_mapper_transform;

#[test]
fn names_the_default_export_from_the_sfc_filename() {
    let source = "<script setup lang=\"ts\"></script>";
    for (file_name, component_name) in [
        ("Child.vue", "Child"),
        ("user-card.vue", "UserCard"),
        ("user_card.vue", "UserCard"),
        ("component.name.vue", "Component_name"),
        ("123-widget.vue", "_123Widget"),
        ("日本語.vue", "VueComponent"),
    ] {
        let result =
            generate_vue_content_mapper_transform(Path::new(file_name), source).expect("transform");
        assert!(
            result.text.contains(&format!(
                "declare const {component_name}: typeof __vize_component__;\nexport default {component_name};"
            )),
            "{}",
            result.text
        );
    }
}

#[test]
fn avoids_a_filename_collision_with_an_authored_module_binding() {
    let source = r#"<script setup lang="ts">
import Child from "./dependency";
void Child;
</script>"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("Child.vue"), source).expect("transform");

    assert!(
        result.text.contains(
            "declare const ChildVueComponent: typeof __vize_component__;\nexport default ChildVueComponent;"
        ),
        "{}",
        result.text
    );

    let setup_local = r#"<script setup lang="ts">
const Child = 1;
</script>"#;
    let result = generate_vue_content_mapper_transform(Path::new("Child.vue"), setup_local)
        .expect("transform");
    assert!(
        result.text.contains("export default Child;"),
        "{}",
        result.text
    );
}

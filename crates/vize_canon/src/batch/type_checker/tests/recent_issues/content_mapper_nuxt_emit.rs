//! Content Mapper component constructors must retain literal public instance
//! members when Nuxt contributes an open `$${string}` injection surface
//! (#10195).

use std::path::Path;

use crate::batch::generate_vue_content_mapper_transform;

#[test]
fn content_mapper_preserves_emit_with_nuxt_pattern_injections() {
    let source = include_str!(
        "../../../../../../../tests/_fixtures/differential/typechecker/content-mapper-nuxt-emit-10195/src/MyItem.vue.txt"
    );
    let result = generate_vue_content_mapper_transform(Path::new("src/MyItem.vue"), source)
        .expect("transform");

    assert!(result.text.contains(
        "__VizeReplaceComponentProps<__VizeComponentInstance, __VizeAuthoredKeyWitness<__VizeAuthoredProps>"
    ));
    assert!(
        !result
            .text
            .contains(": Omit<__VizeComponentInstance, '$props'> & {")
    );
}

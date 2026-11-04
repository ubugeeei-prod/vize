use super::*;

#[test]
fn expect_directive_targets_the_next_template_line() {
    let source = "<script setup lang=\"ts\">\nconst count = 1\n</script>\n<template>\n  <!-- @vue-expect-error -->\n  {{ count.bad }}\n</template>\n";
    let result = transform(source);
    let directives = result.diagnostic_directives.unwrap();

    assert_eq!(
        directives.unused_expect_directive_diagnostics.len(),
        1,
        "{directives:?}"
    );
    assert_eq!(directives.unused_expect_directive_diagnostics[0].code, 4);
    assert_eq!(directives.directives.len(), 1, "{directives:?}");
    let [
        original_start,
        original_len,
        virtual_start,
        virtual_end,
        policy,
        unused_index,
    ] = directives.directives[0].0;
    assert_eq!(policy, DIRECTIVE_POLICY_EXPECT);
    assert_eq!(unused_index, 0);
    assert_eq!(
        &source[original_start..original_start + original_len],
        "@vue-expect-error"
    );
    let suppressed = &result.text.as_str()[virtual_start..virtual_end];
    assert_eq!(suppressed.matches("count").count(), 1, "{suppressed}");
}

/// The frozen Volar source must suppress the actual instance check, while
/// leaving later siblings and descendant checks outside its ownership.
#[test]
fn volar_expect_directive_owns_the_instance_primary_not_a_navigation_copy() {
    let source = include_str!("fixtures/volar-directives.vue");
    let result =
        generate_vue_content_mapper_transform(Path::new("/project/Directives.vue"), source)
            .unwrap();
    let text = result.text.as_str();
    let primary_at = |name: &str| {
        let needle = format!("__vize_public_instance.{name}");
        text.find(&needle).expect("instance primary") + "__vize_public_instance.".len()
    };
    let expected = primary_at("expectedMissing");
    let navigation = text
        .find("void (expectedMissing)")
        .expect("navigation copy")
        + "void (".len();
    let table = result
        .diagnostic_directives
        .as_ref()
        .expect("directive table");
    let token = source.find("@vue-expect-error").unwrap();
    let owner = table
        .directives
        .iter()
        .find(|row| row.0[0] == token && row.0[4] == DIRECTIVE_POLICY_EXPECT)
        .expect("authored expect owner");
    if let Some(output) = std::env::var_os("VIZE_DIRECTIVE_SOURCE_PACKET") {
        std::fs::write(
            output,
            serde_json::to_vec_pretty(&serde_json::json!({
                "source": source, "projection": &result,
                "expectedPrimary": expected, "navigationCopy": navigation
            }))
            .unwrap(),
        )
        .unwrap();
    }
    assert!(
        owner.0[2] <= expected && expected < owner.0[3],
        "expect must own actual TS2339 primary {expected}, got {:?}",
        owner.0
    );
    assert!(
        navigation < owner.0[2] || owner.0[3] <= navigation,
        "a diagnostic-free navigation copy cannot replace the primary"
    );
    for name in ["realMissing", "childMissing"] {
        let primary = primary_at(name);
        assert!(
            !table
                .directives
                .iter()
                .any(|row| { row.0[2] <= primary && primary < row.0[3] }),
            "independent {name} primary must remain visible"
        );
    }
}

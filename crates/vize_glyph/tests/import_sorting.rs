//! Whole-byte SFC import sorting and semantic safety contracts.
use vize_glyph::{Allocator, FormatOptions, GlyphFormatter, resolve_sort_imports};
use vize_l0::config::SortImportsSetting;

fn sorted(source: &str, setting: &str) -> vize_l0::String {
    let setting: SortImportsSetting = serde_json::from_str(setting).expect("config");
    let sorting = resolve_sort_imports(Some(&setting)).expect("valid sorting");
    let allocator = Allocator::default();
    let options = FormatOptions::default();
    GlyphFormatter::new(&options, &allocator)
        .with_sort_imports(sorting.as_ref())
        .format(source)
        .expect("formatted SFC")
        .code
}

#[test]
fn setup_imports_match_the_authored_reference_and_stabilize() {
    let source = include_str!("fixtures/sort-imports/UserCard.vue");
    let expected = include_str!("fixtures/sort-imports/UserCard.sorted.vue");
    let output = sorted(source, "{}");
    assert_eq!(output.as_str(), expected);
    assert_eq!(sorted(&output, "{}"), output);
    assert_eq!(sorted(source, "false").as_str(), source);
}

#[test]
fn ordinary_scripts_and_tsx_keep_their_block_syntax() {
    for lang in ["ts", "tsx"] {
        let source = vize_l0::cstr!(
            "<script lang=\"{lang}\">\nimport B from \"./b\";\nimport A from \"./a\";\nexport default [B, A];\n</script>\n"
        );
        let expected = vize_l0::cstr!(
            "<script lang=\"{lang}\">\nimport A from \"./a\";\nimport B from \"./b\";\nexport default [B, A];\n</script>\n"
        );
        assert_eq!(sorted(&source, "{}"), expected);
    }
}

#[test]
fn configured_groups_custom_sources_and_boundaries_apply() {
    let source = "<script setup>\nimport B from \"./b\";\nimport A from \"project/a\";\nimport C from \"vue\";\n</script>\n";
    let expected = "<script setup>\nimport C from \"vue\";\nimport A from \"project/a\";\n\nimport B from \"./b\";\n</script>\n";
    let setting = r#"{"groups":["external",{"newlinesBetween":false},"project",{"newlinesBetween":true},["sibling","parent","index"]],"newlinesBetween":false,"customGroups":[{"groupName":"project","elementNamePattern":["project/**"]}]}"#;
    assert_eq!(sorted(source, setting).as_str(), expected);
    assert_eq!(sorted(expected, setting).as_str(), expected);
}

#[test]
fn side_effects_and_comment_partitions_retain_authored_order() {
    let source = "<script setup>\nimport \"./z-init\";\nimport \"./a-init\";\n// stable boundary\nimport B from \"./b\";\nimport A from \"./a\";\n</script>\n";
    let expected = "<script setup>\nimport \"./z-init\";\nimport \"./a-init\";\n// stable boundary\nimport A from \"./a\";\nimport B from \"./b\";\n</script>\n";
    assert_eq!(
        sorted(source, r#"{"partitionByComment":true}"#).as_str(),
        expected
    );
}

#[test]
fn contradictory_settings_fail_before_formatting() {
    for source in [
        "true",
        r#"{"order":"random"}"#,
        r#"{"partitionByNewline":true}"#,
        r#"{"partitionByNewline":true,"newlinesBetween":false,"groups":["external",{"newlinesBetween":true},"sibling"]}"#,
    ] {
        let setting: SortImportsSetting = serde_json::from_str(source).expect("settings value");
        assert!(resolve_sort_imports(Some(&setting)).is_err());
    }
}

#[test]
fn internal_patterns_descending_order_and_newline_partitions_apply() {
    let source = "<script setup>\nimport A from \"project/a\";\nimport V from \"vue\";\nimport B from \"project/b\";\n</script>\n";
    let expected = "<script setup>\nimport V from \"vue\";\n\nimport B from \"project/b\";\nimport A from \"project/a\";\n</script>\n";
    assert_eq!(
        sorted(source, r#"{"internalPattern":["project/"],"order":"desc"}"#).as_str(),
        expected
    );
    let source = "<script setup>\nimport B from \"./b\";\nimport A from \"./a\";\n\nimport D from \"./d\";\nimport C from \"./c\";\n</script>\n";
    let expected = "<script setup>\nimport A from \"./a\";\nimport B from \"./b\";\n\nimport C from \"./c\";\nimport D from \"./d\";\n</script>\n";
    assert_eq!(
        sorted(
            source,
            r#"{"partitionByNewline":true,"newlinesBetween":false}"#
        )
        .as_str(),
        expected
    );
}

#[test]
fn misplaced_boundaries_fail_with_the_complete_validation_error() {
    for source in [
        r#"{"groups":[{"newlinesBetween":true},"external"]}"#,
        r#"{"groups":["external",{"newlinesBetween":true}]}"#,
        r#"{"groups":["external",{"newlinesBetween":true},{"newlinesBetween":false},"sibling"]}"#,
    ] {
        let setting: SortImportsSetting = serde_json::from_str(source).expect("setting");
        assert_eq!(
            resolve_sort_imports(Some(&setting))
                .expect_err("misplaced boundary")
                .to_string(),
            "Failed to format script: sortImports boundary must sit between two groups",
        );
    }
}

#[test]
fn unknown_sort_settings_and_custom_group_keys_are_rejected() {
    for source in [
        r#"{"newlineBetween":false}"#,
        r#"{"customGroups":[{"groupName":"project","elementNamePatern":["project/**"]}]}"#,
    ] {
        assert!(serde_json::from_str::<SortImportsSetting>(source).is_err());
    }
}

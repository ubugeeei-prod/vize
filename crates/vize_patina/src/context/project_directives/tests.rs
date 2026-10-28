use super::ProjectLintDirectives;

const EDITOR: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/cross-file-directives-10912/original/src/Editor.vue.txt"
));
const PAGE_B: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/cross-file-directives-10912/original/src/PageB.vue.txt"
));
const SPREAD: &str = "croquis/cf/spread-breaks-reactivity";
const DUPLICATE: &str = "croquis/cf/duplicate-id";

fn ranges(directives: &ProjectLintDirectives) -> Vec<(u32, u32)> {
    directives
        .unused_named(|rule| matches!(rule, SPREAD | DUPLICATE))
        .into_iter()
        .map(|diagnostic| (diagnostic.start, diagnostic.end))
        .collect()
}

#[test]
fn original_script_and_template_comments_share_suppression_and_usage() {
    for (source, filename, rule, anchor) in [
        (EDITOR, "Editor.vue", SPREAD, "...initial"),
        (PAGE_B, "PageB.vue", DUPLICATE, "id="),
    ] {
        let directives = ProjectLintDirectives::from_sfc(source, filename).unwrap();
        let expected_marker = source.find("eslint-disable-next-line").unwrap() as u32;
        let unused = ranges(&directives);
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0].0, expected_marker);
        assert!(!directives.is_disabled_at(
            "croquis/cf/unmatched-inject",
            source.find(anchor).unwrap() as u32
        ));
        assert_eq!(ranges(&directives), unused);
        assert!(directives.is_disabled_at(rule, source.find(anchor).unwrap() as u32));
        assert!(ranges(&directives).is_empty());
        // A second query uses the same authored comment and never emits unused.
        assert!(directives.is_disabled_at(rule, source.find(anchor).unwrap() as u32));
        assert!(ranges(&directives).is_empty());
    }
}

#[test]
fn disabled_or_foreign_rule_names_do_not_enter_unused_accounting() {
    for source in [EDITOR, PAGE_B] {
        let directives = ProjectLintDirectives::from_sfc(source, "Component.vue").unwrap();
        assert!(directives.unused_named(|_| false).is_empty());
    }
}

#[test]
fn oxlint_uses_the_same_canonical_rule_identity_without_display_aliases() {
    for (source, rule, anchor) in [(EDITOR, SPREAD, "...initial"), (PAGE_B, DUPLICATE, "id=")] {
        let source = source.replace("eslint-disable", "oxlint-disable");
        let directives = ProjectLintDirectives::from_sfc(&source, "Component.vue").unwrap();
        let start = source.find(anchor).unwrap() as u32;
        assert!(!directives.is_disabled_at("cross-file", start));
        assert!(!directives.is_disabled_at(&format!("vize:{rule}"), start));
        assert!(!directives.is_disabled_at(rule.rsplit('/').next().unwrap(), start));
        assert!(directives.is_disabled_at(rule, start));
    }
}

#[test]
fn template_blanket_reconciliation_is_exact_and_keeps_listed_normal_reports() {
    let source = PAGE_B.replace(" croquis/cf/duplicate-id", "");
    let directives = ProjectLintDirectives::from_sfc(&source, "PageB.vue").unwrap();
    let marker = source.find("eslint-disable-next-line").unwrap() as u32;
    let end = marker + "eslint-disable-next-line".len() as u32;
    let mut diagnostics = vec![
        crate::LintDiagnostic::error("vue/comment-directive", "unused", marker, end),
        crate::LintDiagnostic::error(
            "vue/comment-directive",
            "another comment",
            marker + 1,
            end + 1,
        ),
        crate::LintDiagnostic::error("vue/require-v-for-key", "ordinary finding", marker, end),
    ];
    directives.reconcile_template_unused(&mut diagnostics);
    assert_eq!(diagnostics.len(), 3);
    assert!(directives.is_disabled_at(DUPLICATE, source.find("id=").unwrap() as u32));
    directives.reconcile_template_unused(&mut diagnostics);
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].message.as_str(), "another comment");
    assert_eq!(diagnostics[1].rule_name, "vue/require-v-for-key");

    let source = PAGE_B.replace(DUPLICATE, &format!("vue/require-v-for-key, {DUPLICATE}"));
    let directives = ProjectLintDirectives::from_sfc(&source, "PageB.vue").unwrap();
    let marker = source.find("eslint-disable-next-line").unwrap() as u32;
    let end = source.find(" -->").unwrap() as u32;
    let mut listed = vec![crate::LintDiagnostic::error(
        "vue/comment-directive",
        "ordinary unused",
        marker,
        end,
    )];
    assert!(directives.is_disabled_at(DUPLICATE, source.find("id=").unwrap() as u32));
    directives.reconcile_template_unused(&mut listed);
    assert_eq!(listed.len(), 1);
}

#[test]
fn directives_cannot_claim_an_offset_outside_their_authored_file() {
    let directives = ProjectLintDirectives::from_sfc(PAGE_B, "PageB.vue").unwrap();
    assert!(!directives.is_disabled_at(DUPLICATE, PAGE_B.len() as u32));
    assert!(!directives.is_disabled_at(DUPLICATE, u32::MAX));
}

#[test]
fn script_file_regions_share_usage_with_their_template_scope() {
    let source = format!(
        "<script>/* eslint-disable {DUPLICATE} */\nconst value = 1;\n</script>\n{}",
        PAGE_B
            .lines()
            .filter(|line| !line.contains("eslint-disable"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let directives = ProjectLintDirectives::from_sfc(&source, "PageB.vue").unwrap();
    assert_eq!(ranges(&directives).len(), 1);
    assert!(directives.is_disabled_at(DUPLICATE, source.find("id=").unwrap() as u32));
    assert!(ranges(&directives).is_empty());

    // A next-line pragma in a script cannot claim the following template.
    let source = EDITOR.replace(SPREAD, DUPLICATE);
    let directives = ProjectLintDirectives::from_sfc(&source, "Editor.vue").unwrap();
    assert!(!directives.is_disabled_at(DUPLICATE, source.find("<p>").unwrap() as u32));
    assert_eq!(ranges(&directives).len(), 1);
}

#[test]
fn strings_attributes_and_custom_blocks_cannot_create_directive_events() {
    let script = EDITOR.replace(
        "// eslint-disable-next-line croquis/cf/spread-breaks-reactivity -- a draft copy of the prop",
        "const note = 'eslint-disable-next-line croquis/cf/spread-breaks-reactivity';"
    );
    let attribute = PAGE_B.replace(
        "<!-- eslint-disable-next-line croquis/cf/duplicate-id -->",
        "<span title=\"eslint-disable-next-line croquis/cf/duplicate-id\"></span>",
    );
    let custom = format!(
        "<docs><!-- eslint-disable croquis/cf/duplicate-id --></docs>\n{}",
        PAGE_B
            .lines()
            .filter(|line| !line.contains("eslint-disable"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    for (source, rule, anchor) in [
        (script, SPREAD, "...initial"),
        (attribute, DUPLICATE, "id="),
        (custom, DUPLICATE, "id="),
    ] {
        let directives = ProjectLintDirectives::from_sfc(&source, "Component.vue").unwrap();
        assert!(!directives.is_disabled_at(rule, source.find(anchor).unwrap() as u32));
        assert!(ranges(&directives).is_empty());
    }
}

#[test]
fn unsupported_vize_disable_alias_keeps_the_normal_lint_grammar() {
    let source = PAGE_B.replace("eslint-disable", "vize-disable");
    assert!(ProjectLintDirectives::from_sfc(&source, "PageB.vue").is_none());
}

#[test]
fn canonical_vize_comment_controls_share_the_normal_template_prescan() {
    for marker in ["@vize:level off", "@vize:level warn"] {
        let source = PAGE_B.replace("eslint-disable-next-line croquis/cf/duplicate-id", marker);
        let directives = ProjectLintDirectives::from_sfc(&source, "PageB.vue").unwrap();
        let start = source.find("id=").unwrap() as u32;
        let mut diagnostic = crate::LintDiagnostic::error(
            "cross-file",
            "unsupported syntax stays active",
            start,
            start + 2,
        );
        assert!(directives.admit(DUPLICATE, &mut diagnostic));
        assert_eq!(diagnostic.severity, crate::Severity::Error);
    }

    for marker in [
        "@vize:forget deliberate",
        "@vize:ignore-start",
        "@vize:expected deliberate",
    ] {
        let source = PAGE_B.replace("eslint-disable-next-line croquis/cf/duplicate-id", marker);
        let directives = ProjectLintDirectives::from_sfc(&source, "PageB.vue").unwrap();
        assert!(
            directives.is_disabled_at(DUPLICATE, source.find("id=").unwrap() as u32),
            "{marker}"
        );
        assert!(ranges(&directives).is_empty());
    }
    for (level, severity, admitted) in [
        ("off", crate::Severity::Error, false),
        ("warn", crate::Severity::Warning, true),
        ("error", crate::Severity::Error, true),
    ] {
        let source = PAGE_B.replace(
            "eslint-disable-next-line croquis/cf/duplicate-id",
            &format!("@vize:level({level})"),
        );
        let directives = ProjectLintDirectives::from_sfc(&source, "PageB.vue").unwrap();
        let start = source.find("id=").unwrap() as u32;
        let mut diagnostic =
            crate::LintDiagnostic::error("cross-file", "structured finding", start, start + 2);
        assert_eq!(directives.admit(DUPLICATE, &mut diagnostic), admitted);
        if admitted {
            assert_eq!(diagnostic.severity, severity);
        }
    }
}

#[test]
fn existing_plugin_rule_alias_and_authored_unicode_crlf_ranges_are_retained() {
    for original in [EDITOR, PAGE_B] {
        let source = format!("<!-- 😀 authored header -->\n{original}")
            .replace("croquis/cf/", "vize/croquis/cf/")
            .replace('\n', "\r\n");
        let (rule, anchor) = if original == EDITOR {
            (SPREAD, "...initial")
        } else {
            (DUPLICATE, "id=")
        };
        let directives = ProjectLintDirectives::from_sfc(&source, "Component.vue").unwrap();
        assert_eq!(
            ranges(&directives)[0].0,
            source.find("eslint-disable").unwrap() as u32
        );
        assert!(directives.is_disabled_at(rule, source.find(anchor).unwrap() as u32));
        assert!(ranges(&directives).is_empty());
    }
}

use globset::{GlobBuilder, GlobMatcher};
use vize_l0::{String, ToCompactString, cstr};

use super::{DoctorFilterDimension, PatternSet};
use crate::DoctorFilterSpec;

fn independent(patterns: &[String]) -> Vec<GlobMatcher> {
    patterns
        .iter()
        .filter_map(|pattern| {
            GlobBuilder::new(pattern)
                .literal_separator(true)
                .backslash_escape(false)
                .build()
                .ok()
        })
        .map(|glob| glob.compile_matcher())
        .collect()
}

#[test]
fn combined_patterns_preserve_literal_prefix_suffix_regex_and_unicode_verdicts() {
    let groups: &[&[&str]] = &[
        &["VIZE_DOCTOR_SECURITY_001", "VIZE_DOCTOR_PERFORMANCE_001"],
        &[
            "packages/account/**",
            "packages/account/sub/**",
            "packages/team/**",
        ],
        &[
            "**/account/components/Login.vue",
            "**/team/components/Panel.vue",
        ],
        &["VIZE_*_[AB]_???", "VIZE_SECURITY_???", "VIZE_*_[CD]_???"],
        &[
            "日本語/**",
            "**/组件/Panel.vue",
            "apps/[ab]/**",
            "src/{one,two}.ts",
        ],
        &["literal\\name", "case?name", "CASE*"],
    ];
    let values = [
        "",
        "VIZE_DOCTOR_SECURITY_001",
        "VIZE_DOCTOR_PERFORMANCE_001",
        "VIZE_SECURITY_002",
        "VIZE_DOCTOR_A_001",
        "VIZE_DOCTOR_D_999",
        "packages/account/src/Login.vue",
        "packages/account/sub/Deep.vue",
        "packages/accountant/src/Login.vue",
        "apps/account/components/Login.vue",
        "account/components/Login.vue",
        "otheraccount/components/Login.vue",
        "apps/team/components/Panel.vue",
        "日本語/页面.vue",
        "组件/Panel.vue",
        "目录/组件/Panel.vue",
        "apps/a/deep/Panel.vue",
        "src/one.ts",
        "src/three.ts",
        "literal\\name",
        "casexname",
        "casex/name",
        "CASEsuffix",
    ];
    for group in groups {
        let patterns: Vec<String> = group.iter().map(|pattern| (*pattern).into()).collect();
        let combined = PatternSet::compile(DoctorFilterDimension::Rule, &patterns).unwrap();
        let baseline = independent(&patterns);
        assert_eq!(baseline.len(), patterns.len());
        let expected: Vec<bool> = values
            .iter()
            .map(|value| baseline.iter().any(|matcher| matcher.is_match(value)))
            .collect();
        let actual: Vec<bool> = values.iter().map(|value| combined.matches(value)).collect();
        assert_eq!(actual, expected, "{group:?}");
        assert!(matches!(combined, PatternSet::Combined(_)));
    }
}

#[test]
fn unrestricted_single_combined_and_optional_context_keep_their_contracts() {
    for patterns in [
        vec![],
        vec![String::from("web")],
        vec!["web".into(), "mobile".into()],
    ] {
        let compiled = PatternSet::compile(DoctorFilterDimension::Target, &patterns).unwrap();
        let baseline = independent(&patterns);
        for value in [None, Some(""), Some("web"), Some("mobile"), Some("server")] {
            assert_eq!(
                compiled.matches_optional(value),
                patterns.is_empty()
                    || value.is_some_and(|value| baseline
                        .iter()
                        .any(|matcher| matcher.is_match(value)))
            );
        }
        assert_eq!(compiled.is_unrestricted(), patterns.is_empty());
    }
}

#[test]
fn path_normalization_and_component_boundaries_match_independent_patterns() {
    let patterns = vec![
        "packages/account/**".into(),
        "**/components/Panel.vue".into(),
    ];
    let compiled = PatternSet::compile(DoctorFilterDimension::Path, &patterns).unwrap();
    let values = [
        r"packages\account\src\Login.vue",
        r"packages\accountant\src\Login.vue",
        r"日本語\components\Panel.vue",
        "components/Panel.vue",
        "badcomponents/Panel.vue",
    ];
    let actual: Vec<bool> = values
        .iter()
        .map(|value| compiled.matches_path(value))
        .collect();
    assert_eq!(actual, [true, false, true, true, false]);
}

#[test]
fn invalid_pattern_priority_and_normalized_dimension_errors_are_unchanged() {
    let patterns = vec!["valid".into(), "[broken".into(), "".into()];
    let error = PatternSet::compile(DoctorFilterDimension::Route, &patterns).unwrap_err();
    let expected_reason = GlobBuilder::new("[broken")
        .literal_separator(true)
        .backslash_escape(false)
        .build()
        .unwrap_err()
        .to_compact_string();
    assert_eq!(error.dimension(), DoctorFilterDimension::Route);
    assert_eq!(error.pattern(), "[broken");
    assert_eq!(error.reason(), expected_reason);

    let error = DoctorFilterSpec {
        targets: vec!["[zzz".into(), "[aaa".into()],
        rules: vec!["".into()],
        ..DoctorFilterSpec::default()
    }
    .compile()
    .unwrap_err();
    assert_eq!(error.dimension(), DoctorFilterDimension::Target);
    assert_eq!(error.pattern(), "[aaa");
    assert_eq!(
        error.to_compact_string(),
        cstr!(
            "invalid target filter pattern {:?}: {}",
            "[aaa",
            GlobBuilder::new("[aaa")
                .literal_separator(true)
                .backslash_escape(false)
                .build()
                .unwrap_err()
        )
        .as_str()
    );
}

#[test]
fn aggregate_regex_limit_preserves_valid_independent_specifications() {
    // Each individual matcher is below globset 0.4.20's 10 MiB NFA limit,
    // while the combined set exceeds it. This exercises the actual fallback.
    let suffix = "[ab]?".repeat(2048);
    let patterns: Vec<String> = (0..64)
        .map(|index| cstr!("rule-{index:03}-{suffix}"))
        .collect();
    let compiled = PatternSet::compile(DoctorFilterDimension::Rule, &patterns).unwrap();
    assert!(matches!(compiled, PatternSet::Independent(_)));
    let positive = cstr!("rule-063-{}", "ab".repeat(2048));
    assert!(compiled.matches(positive.as_str()));
    assert!(!compiled.matches("rule-063-ab"));
}

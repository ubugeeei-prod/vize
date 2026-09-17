use super::analyze;
use crate::ScopeKind;

#[test]
fn invalid_patterns_preserve_descendant_facts_without_partial_bindings() {
    for (subject, arm) in [
        ("v-match", "v-when=\"const leaked\""),
        ("v-match=\"subject\"", ""),
        ("v-match=\"subject\"", "v-when"),
        ("v-match=\"subject\"", "v-when.mod=\"const leaked\""),
        (
            "v-match=\"subject\"",
            "v-when=\"[const leaked, const leaked]\"",
        ),
        (
            "v-match=\"subject\"",
            "v-when=\"const leaked\" v-if=\"subject\"",
        ),
        (
            "v-match=\"subject\"",
            "v-when=\"const leaked\" v-for=\"outer in items\"",
        ),
    ] {
        let source = format!(
            r#"<template {subject}>
          <Widget {arm} :value="missingProp">
            <p v-for="item in items" @click="() => missingEvent">{{{{ item }}}}{{{{ missingText }}}}{{{{ leaked }}}}</p>
            <template v-match="subject"><b v-when="const nested">{{{{ nested }}}}{{{{ missingNested }}}}</b></template>
          </Widget>
          <p v-when="_">{{{{ missingSibling }}}}</p>
        </template>"#
        );
        let result = analyze(
            &source,
            "const subject = {}; const items = []; const Widget = {};",
            true,
        );
        assert!(!result.pattern_diagnostics.is_empty(), "{subject}: {arm}");
        if subject != "v-match" {
            assert!(
                result
                    .pattern_diagnostics
                    .iter()
                    .all(|diagnostic| !diagnostic.message.starts_with("v-when must be"))
            );
        }
        let mut missing: Vec<_> = result
            .undefined_refs
            .iter()
            .map(|r| r.name.as_str())
            .collect();
        missing.sort_unstable();
        assert_eq!(
            missing,
            [
                "leaked",
                "missingEvent",
                "missingNested",
                "missingProp",
                "missingSibling",
                "missingText"
            ],
            "{subject}: {arm}"
        );
        assert!(
            result
                .component_usages
                .iter()
                .any(|usage| usage.name == "Widget")
        );
        for kind in [ScopeKind::VFor, ScopeKind::EventHandler, ScopeKind::VWhen] {
            assert!(
                result.scopes.iter().any(|scope| scope.kind == kind),
                "{kind:?}: {source}"
            );
        }
        assert!(
            result
                .scopes
                .iter()
                .all(|scope| !scope.has_binding("leaked"))
        );
        assert!(result.scopes.iter().any(|scope| {
            scope
                .get_binding("nested")
                .is_some_and(|binding| binding.is_used())
        }));
    }
}

#[test]
fn recovery_does_not_treat_indirect_arms_as_direct_children() {
    let source = r#"<template v-match="subject"><section><p v-when="const leaked">{{ leaked }}</p></section><p v-when="_" /></template>"#;
    let result = analyze(source, "const subject = {};", true);
    assert!(
        result
            .pattern_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.starts_with("v-when must be"))
    );
    assert_eq!(result.undefined_refs.len(), 1);
    assert_eq!(result.undefined_refs[0].name, "leaked");
    assert!(
        result
            .scopes
            .iter()
            .all(|scope| !scope.has_binding("leaked"))
    );
}

#[test]
fn invalid_arm_cohosts_do_not_dispatch_patterns_but_valid_descendants_do() {
    for arm in [
        "v-when=\"const outer\"",
        "v-when",
        "v-when=\"values[class]\"",
    ] {
        let source = format!(
            r#"<template v-match="subject">
          <Widget {arm} v-match="subject">
            <p v-when="const leaked">{{{{ leaked }}}}{{{{ missing }}}}{{{{ outer }}}}</p>
            <template v-match="subject"><b v-when="const valid">{{{{ valid }}}}</b></template>
          </Widget>
          <p v-when="_" />
        </template>"#
        );
        let result = analyze(&source, "const subject = {}; const Widget = {};", true);
        assert_eq!(
            result
                .scopes
                .iter()
                .filter(|scope| scope.kind == ScopeKind::VMatch)
                .count(),
            2,
            "{arm}"
        );
        assert_eq!(
            result
                .scopes
                .iter()
                .filter(|scope| scope.kind == ScopeKind::VWhen)
                .count(),
            2,
            "{arm}"
        );
        let mut missing: Vec<_> = result
            .undefined_refs
            .iter()
            .map(|reference| reference.name.as_str())
            .collect();
        missing.sort_unstable();
        assert_eq!(missing, ["leaked", "missing", "outer"], "{arm}");
        assert!(
            result
                .component_usages
                .iter()
                .any(|usage| usage.name == "Widget")
        );
        assert!(
            result
                .scopes
                .iter()
                .all(|scope| !scope.has_binding("leaked") && !scope.has_binding("outer"))
        );
        assert!(result.scopes.iter().any(|scope| {
            scope
                .get_binding("valid")
                .is_some_and(|binding| binding.is_used())
        }));
    }
}

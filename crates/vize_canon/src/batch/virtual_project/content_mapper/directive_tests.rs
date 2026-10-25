#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use std::path::Path;

use super::generate_vue_content_mapper_transform;
use super::protocol::{DIRECTIVE_POLICY_EXPECT, DIRECTIVE_POLICY_IGNORE};

fn transform(source: &str) -> super::ContentMapperTransform {
    generate_vue_content_mapper_transform(Path::new("/project/App.vue"), source).unwrap()
}

fn without_owned_input_echoes(source: &str) -> super::ContentMapperTransform {
    super::transform_with_diagnostic_ownership(
        Path::new("/project/App.vue"),
        source,
        super::ContentMapperTransformOptions::default(),
        None,
    )
    .unwrap()
}

fn static_id_echoes(
    source: &str,
    generated: &str,
    id: &str,
) -> Vec<super::protocol::ContentMapperDiagnosticDirective> {
    use super::protocol::ContentMapperDiagnosticDirective as Directive;
    let original = source.find(&format!("id=\"{id}\"")).unwrap();
    let entry = format!("\"id\": \"{id}\"");
    let positions = generated
        .match_indices(&entry)
        .map(|(start, _)| start)
        .collect::<Vec<_>>();
    assert_eq!(
        positions.len(),
        3,
        "two actual inference literals and one canonical literal"
    );
    positions
        .into_iter()
        .take(2)
        .flat_map(|generated| {
            [
                Directive([
                    original,
                    2,
                    generated,
                    generated + 4,
                    DIRECTIVE_POLICY_IGNORE,
                    0,
                ]),
                Directive([
                    original + 4,
                    id.len(),
                    generated + 6,
                    generated + entry.len(),
                    DIRECTIVE_POLICY_IGNORE,
                    0,
                ]),
            ]
        })
        .collect()
}

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
    assert!(suppressed.contains("count"), "{suppressed}");
}

#[test]
fn ignore_directive_suppresses_without_an_unused_report() {
    let source = "<script setup lang=\"ts\">\nconst count = 1\n</script>\n<template>\n  <!-- @vue-ignore -->\n  {{ count.bad }}\n</template>\n";
    let directives = transform(source).diagnostic_directives.unwrap();

    assert!(directives.unused_expect_directive_diagnostics.is_empty());
    assert!(!directives.directives.is_empty());
    for directive in &directives.directives {
        let [_, _, virtual_start, virtual_end, policy, _] = directive.0;
        assert_eq!(policy, DIRECTIVE_POLICY_IGNORE);
        assert!(virtual_start < virtual_end);
    }
}

#[test]
fn ignore_directive_owns_parent_checks_without_hiding_child_checks() {
    let source = "<script setup lang=\"ts\">const child = 1</script>\n<template>\n  <!-- @vue-ignore -->\n  <div :id=\"child.bad\">{{ child.bad }}</div>\n</template>\n";
    let result = transform(source);
    let directives = result.diagnostic_directives.unwrap();
    let parent = source.find("child.bad").unwrap();
    let child = source.find("{{ child.bad }}").unwrap() + 3;
    let parent_positions = mapped_positions(&result.mappings, parent);
    let child_positions = mapped_positions(&result.mappings, child);
    assert!(!parent_positions.is_empty(), "parent check has no mapping");
    assert!(!child_positions.is_empty(), "child check has no mapping");
    assert!(
        parent_positions
            .iter()
            .all(|position| covered(&directives.directives, *position))
    );
    assert!(
        child_positions
            .iter()
            .all(|position| !covered(&directives.directives, *position))
    );
}

#[test]
fn skip_directive_owns_parent_and_child_checks() {
    let source = "<script setup lang=\"ts\">const child = 1</script>\n<template>\n  <!-- @vue-skip -->\n  <div :id=\"child.bad\">{{ child.bad }}</div>\n</template>\n";
    let result = transform(source);
    let directives = result.diagnostic_directives.unwrap();
    let parent = source.find("child.bad").unwrap();
    let child = source.find("{{ child.bad }}").unwrap() + 3;
    let parent_positions = mapped_positions(&result.mappings, parent);
    let child_positions = mapped_positions(&result.mappings, child);
    assert!(!parent_positions.is_empty(), "parent check has no mapping");
    assert!(!child_positions.is_empty(), "child check has no mapping");
    assert!(
        parent_positions
            .iter()
            .all(|position| covered(&directives.directives, *position))
    );
    assert!(
        child_positions
            .iter()
            .all(|position| covered(&directives.directives, *position))
    );
}

fn mapped_positions(spans: &[super::protocol::ContentMapperSpan], original: usize) -> Vec<usize> {
    spans
        .iter()
        .filter_map(
            |super::protocol::ContentMapperSpan(
                [generated, length, start, original_length, _, _],
            )| {
                (*start <= original && original < start + original_length)
                    .then_some(*generated + (*length / 2))
            },
        )
        .collect()
}

fn covered(
    directives: &[super::protocol::ContentMapperDiagnosticDirective],
    position: usize,
) -> bool {
    directives.iter().any(|directive| {
        let [_, _, start, end, _, _] = directive.0;
        start <= position && position < end
    })
}

#[test]
fn expect_directive_covers_content_on_its_own_line() {
    let source = "<script setup lang=\"ts\">\nconst count = 1\n</script>\n<template>\n  <!-- @vue-expect-error -->{{ count.bad }}\n</template>\n";
    let directives = transform(source).diagnostic_directives.unwrap();

    let [_, _, virtual_start, virtual_end, policy, _] = directives.directives[0].0;
    assert_eq!(policy, DIRECTIVE_POLICY_EXPECT);
    assert!(virtual_start < virtual_end);
}

#[test]
fn unmapped_expect_directive_keeps_an_empty_virtual_range() {
    let source = "<template>\n  <div />\n  <!-- @vue-expect-error -->\n</template>\n";
    let directives = transform(source).diagnostic_directives.unwrap();

    let [_, _, virtual_start, virtual_end, policy, _] = directives.directives[0].0;
    assert_eq!(policy, DIRECTIVE_POLICY_EXPECT);
    assert_eq!((virtual_start, virtual_end), (0, 0));
}

#[test]
fn unmapped_ignore_directive_is_dropped() {
    let source = "<template>\n  <div />\n  <!-- @vue-ignore -->\n</template>\n";
    assert!(transform(source).diagnostic_directives.is_none());
}

#[test]
fn templates_without_directives_emit_none() {
    let source = "<script setup lang=\"ts\">\nconst count = 1\n</script>\n<template>\n  <!-- plain comment -->\n  {{ count }}\n</template>\n";
    assert!(transform(source).diagnostic_directives.is_none());
}

#[test]
fn component_prop_assignments_are_ignored_only_when_the_literal_owns_them() {
    let source = "<script setup lang=\"ts\">import Child from './Child.vue'; const value = 'wrong'</script>\n<template><Child :count=\"value\" /></template>\n";
    let result = transform(source);
    let directives = result
        .diagnostic_directives
        .expect("component prop echo directive");
    let check = result
        .text
        .find("const __vize_prop_check_0_count")
        .expect("per-prop check");
    let literal = result
        .text
        .find("\"count\": value")
        .expect("whole-props entry");

    assert!(covered(&directives.directives, check));
    assert!(!covered(&directives.directives, literal));
    assert!(directives.unused_expect_directive_diagnostics.is_empty());
}

#[test]
fn input_echo_policy_subtracts_only_existing_ignore_intervals() {
    use super::protocol::{
        ContentMapperDiagnosticDirective as Row, ContentMapperDiagnosticDirectives as Table,
        ContentMapperUnusedExpectDiagnostic,
    };
    for (covered, expected) in [
        (0..10, vec![10..30]),
        (30..40, vec![10..30]),
        (10..30, vec![]),
        (15..20, vec![10..15, 20..30]),
        (0..20, vec![20..30]),
        (20..40, vec![10..20]),
    ] {
        let authored = Row([1, 2, covered.start, covered.end, DIRECTIVE_POLICY_IGNORE, 0]);
        let expectation = Row([3, 4, 40, 50, DIRECTIVE_POLICY_EXPECT, 0]);
        let table = Table {
            directives: vec![authored, expectation],
            unused_expect_directive_diagnostics: vec![ContentMapperUnusedExpectDiagnostic {
                code: 4,
                message_text: "Unused '@vue-expect-error' directive".into(),
            }],
        };
        let echoes = [crate::virtual_ts::diagnostic_ownership::DiagnosticEcho {
            generated: 10..30,
            authored: 100..110,
            suppress: true,
        }];
        let result = super::directives::with_owned_input_echoes(Some(table), &echoes).unwrap();
        let mut expected = expected
            .into_iter()
            .map(|range| Row([100, 10, range.start, range.end, DIRECTIVE_POLICY_IGNORE, 0]))
            .collect::<Vec<_>>();
        expected.extend([authored, expectation]);
        assert_eq!(result.directives, expected);
        assert_eq!(
            result.unused_expect_directive_diagnostics,
            [ContentMapperUnusedExpectDiagnostic {
                code: 4,
                message_text: "Unused '@vue-expect-error' directive".into()
            }]
        );
    }
}

#[test]
fn longer_words_do_not_match_directive_tokens() {
    let source = "<template>\n  <!-- @vue-ignores nothing here -->\n  <div />\n</template>\n";
    assert!(transform(source).diagnostic_directives.is_none());
}

#[test]
fn static_component_expect_owns_one_packet_without_its_child() {
    for newline in ["\n", "\r\n"] {
        for prefix in ["", "😀日本"] {
            for (id, event) in [
                ("bad", "missing"),
                ("ok", "missing"),
                ("bad", "save"),
                ("ok", "save"),
            ] {
                let source = vize_l0::cstr!(
                    "<script setup lang=\"ts\">import Comp from './Comp.vue'; const value = 1; const noop = () => {{}};</script>{newline}<template>{prefix}<!-- @vue-expect-error -->{newline}<Comp{newline} id=\"{id}\" @{event}=\"noop\">{{{{ value.bad }}}}</Comp></template>"
                );
                let result = transform(source.as_str());
                let baseline = without_owned_input_echoes(source.as_str())
                    .diagnostic_directives
                    .unwrap();
                let directives = result.diagnostic_directives.unwrap();
                let mut expected = static_id_echoes(source.as_str(), result.text.as_str(), id);
                let mut authored = baseline.directives;
                let event_read = format!("void __vize_events_nav_0.{event};");
                let event_name_start = result.text.as_str().find(&event_read).unwrap()
                    + "void __vize_events_nav_0.".len();
                // The mapped name read is the first real diagnostic owner.
                // The two emitter-proven literals precede this packet and
                // cannot stretch its EXPECT across their IGNORE intervals.
                authored[0].0[2] = event_name_start;
                expected.extend(authored);
                assert_eq!(
                    directives.directives, expected,
                    "complete ordered inference tuples and one precise authored expectation"
                );
                assert_eq!(
                    directives.unused_expect_directive_diagnostics,
                    baseline.unused_expect_directive_diagnostics
                );
                assert_eq!(directives.unused_expect_directive_diagnostics.len(), 1);
                let [token, length, start, end, policy, unused] = directives.directives[4].0;
                assert_eq!(&source[token..token + length], "@vue-expect-error");
                assert_eq!((policy, unused), (DIRECTIVE_POLICY_EXPECT, 0));
                let parent = source.find("<Comp").unwrap();
                let parent_end = parent + source[parent..].find('>').unwrap() + 1;
                let parent_rows = result
                    .mappings
                    .iter()
                    .filter(
                        |super::protocol::ContentMapperSpan([_, _, original, length, _, _])| {
                            *original < parent_end && parent < original + length
                        },
                    )
                    .collect::<Vec<_>>();
                assert!(!parent_rows.is_empty());
                assert!(
                    parent_rows.iter().all(
                        |super::protocol::ContentMapperSpan([generated, length, _, _, _, _])| {
                            covered(&directives.directives, *generated)
                                && covered(&directives.directives, generated + length - 1)
                        }
                    ),
                    "one EXPECT plus exact inference rows cover the complete parent"
                );
                assert!(
                    expected
                        .iter()
                        .take(4)
                        .all(|row| row.0[3] <= start || end <= row.0[2]),
                    "no protocol interval overlaps EXPECT"
                );
                let child = source.find("value.bad").unwrap();
                let child_positions = mapped_positions(&result.mappings, child);
                assert!(!child_positions.is_empty());
                assert!(
                    child_positions
                        .iter()
                        .all(|position| !covered(&directives.directives, *position)),
                    "child cannot be suppressed by the parent's one expectation"
                );
            }
        }
    }
}

#[test]
fn unrelated_marker_text_never_creates_owner_directives() {
    for template in [
        "<!-- prose @vue-expect-error --><Comp id=\"bad\" @save=\"noop\">{{ value.bad }}</Comp>",
        "@vue-expect-error<Comp id=\"bad\" @save=\"noop\">{{ value.bad }}</Comp>",
        "<Comp id=\"@vue-expect-error\" @save=\"noop\">{{ value.bad }}</Comp>",
    ] {
        let source = vize_l0::cstr!(
            "<script setup lang=\"ts\">import Comp from './Comp.vue'; const value = 1; const noop = () => {{}};</script><template>{template}</template>"
        );
        let result = transform(source.as_str());
        let baseline = without_owned_input_echoes(source.as_str())
            .diagnostic_directives
            .expect("legacy component-prop echo directives");
        let directives = result
            .diagnostic_directives
            .expect("internal component-prop echo directives");
        let id = if template.contains("id=\"@vue-expect-error\"") {
            "@vue-expect-error"
        } else {
            "bad"
        };
        let mut expected = static_id_echoes(source.as_str(), result.text.as_str(), id);
        expected.extend(baseline.directives);
        assert_eq!(
            directives.directives, expected,
            "a marker literal is an ignored input value, never an authored directive"
        );
        assert_eq!(
            directives.unused_expect_directive_diagnostics,
            baseline.unused_expect_directive_diagnostics
        );
        assert!(directives.unused_expect_directive_diagnostics.is_empty());
        assert!(directives.directives.iter().all(|directive| {
            let [_, _, _, _, policy, _] = directive.0;
            policy == DIRECTIVE_POLICY_IGNORE
        }));
    }
    let source = "<script setup lang=\"ts\">import Comp from './Comp.vue'; const value = 1; const noop = () => {};</script><template><Comp id=\"bad\" @save=\"noop\">{{ value.bad }}</Comp><!-- @vue-expect-error --></template>";
    let result = transform(source);
    let directives = result.diagnostic_directives.unwrap();
    let authored = directives
        .directives
        .iter()
        .filter(|directive| {
            let [original, length, _, _, _, _] = directive.0;
            &source[original..original + length] == "@vue-expect-error"
        })
        .collect::<Vec<_>>();
    assert_eq!(authored.len(), 1, "{directives:?}");
    assert_eq!((authored[0].0[2], authored[0].0[3]), (0, 0));
}

/// #9746: a comment directly before an SFC block owns that block's opening tag.
#[test]
fn block_directives_cover_duplicate_css_module_names_only() {
    for (token, policy) in [
        ("@vue-ignore", DIRECTIVE_POLICY_IGNORE),
        ("@vue-expect-error", DIRECTIVE_POLICY_EXPECT),
    ] {
        let source = vize_l0::cstr!(
            "<template><div /></template>\n<!-- {token} -->\n<style module=\"foo\"></style>\n<!-- {token} -->\n<style module=\"foo\"></style>\n"
        );
        let source = source.as_str();
        let result = transform(source);
        let directives = result.diagnostic_directives.expect("block directives");
        let text = result.text.as_str();
        let keys: Vec<_> = text
            .match_indices("\"foo\":")
            .map(|(start, _)| start)
            .collect();
        assert_eq!(keys.len(), 2, "{text}");
        for key in keys {
            assert!(
                directives.directives.iter().any(|directive| {
                    let [_, _, start, end, kind, _] = directive.0;
                    kind == policy && start <= key && key + "\"foo\"".len() <= end
                }),
                "{token} must cover generated key at {key}: {directives:?}"
            );
        }
        for directive in &directives.directives {
            let [original, length, start, end, _, _] = directive.0;
            assert_eq!(&source[original..original + length], token);
            // Only the generated module key, never a template check.
            assert_eq!(&text[start..end], "\"foo\"");
        }
    }
}

#[test]
fn block_directives_do_not_reach_the_block_body_or_other_blocks() {
    let source = "<!-- @vue-ignore -->\n<script setup lang=\"ts\">\nconst count: number = 'x'\n</script>\n<template>{{ count }}</template>\n";
    let result = transform(source);
    let text = result.text.as_str();
    let body = text.find("const count: number = 'x'").expect("setup body");
    for directive in result
        .diagnostic_directives
        .iter()
        .flat_map(|directives| &directives.directives)
    {
        let [_, _, start, end, _, _] = directive.0;
        assert!(end <= body || body + 25 <= start, "{:?}", &text[start..end]);
    }
}

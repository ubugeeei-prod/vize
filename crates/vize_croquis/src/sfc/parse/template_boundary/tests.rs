use crate::sfc::parse_sfc;

#[test]
fn ignores_template_tokens_in_non_markup_contexts() {
    let cases = [
        (
            "opening tag in an HTML comment",
            "<template><!-- <template> --><div>after</div></template><style>.ok {}</style>",
        ),
        (
            "closing tag in an HTML comment",
            "<template><!-- </template> --><div>after</div></template><style>.ok {}</style>",
        ),
        (
            "opening tag in a quoted attribute",
            r#"<template><div title="<template>">after</div></template><style>.ok {}</style>"#,
        ),
        (
            "closing tag in a quoted attribute",
            r#"<template><div title="</template>">after</div></template><style>.ok {}</style>"#,
        ),
        (
            "opening tag in an interpolation string",
            r#"<template>{{ "<template>" }}<div>after</div></template><style>.ok {}</style>"#,
        ),
        (
            "closing tag in an interpolation string",
            r#"<template>{{ '</template>' }}<div>after</div></template><style>.ok {}</style>"#,
        ),
        (
            "delimiter and opening tag in an interpolation string",
            r#"<template>{{ "}} <template>" }}<div>after</div></template><style>.ok {}</style>"#,
        ),
        (
            "delimiter and opening tag in an interpolation comment",
            r#"<template>{{ /* }} <template> */ value }}<div>after</div></template><style>.ok {}</style>"#,
        ),
        (
            "opening tag in an interpolation template literal",
            r#"<template>{{ `<template>` }}<div>after</div></template><style>.ok {}</style>"#,
        ),
        (
            "closing tag in an interpolation template literal",
            r#"<template>{{ `</template>` }}<div>after</div></template><style>.ok {}</style>"#,
        ),
        (
            "opening tag in a malformed interpolation string",
            r#"<template>{{ "<template> }}<div>after</div></template><style>.ok {}</style>"#,
        ),
        (
            "closing tag in a malformed interpolation string",
            r#"<template>{{ '</template> }}<div>after</div></template><style>.ok {}</style>"#,
        ),
        (
            "tags in a regular expression",
            r#"<template>{{ /<template>|<\/template>/.test(value) }}<div>after</div></template><style>.ok {}</style>"#,
        ),
        (
            "delimiter and tags in a regex after a newline",
            r#"<template>{{ (
  /}} <template> <\/template>/.test(value)
) }}<div>after</div></template><style>.ok {}</style>"#,
        ),
        (
            "tags in a nested interpolation expression",
            r#"<template>{{ ({ open: "<template>", close: '</template>' }) }}<div>after</div></template><style>.ok {}</style>"#,
        ),
        (
            "tags in script raw text",
            r#"<template><script>const open = "<template>"; const close = "</template>"</script><div>after</div></template><style>.ok {}</style>"#,
        ),
        (
            "tags in textarea raw text",
            r#"<template><textarea><template> and </template></textarea><div>after</div></template><style>.ok {}</style>"#,
        ),
        (
            "template-prefixed custom elements",
            r#"<template><template-card></template-card><templateSlot></templateSlot><div>after</div></template><style>.ok {}</style>"#,
        ),
        (
            "tags in CDATA-like text",
            r#"<template><![CDATA[<template> </template>]]><div>after</div></template><style>.ok {}</style>"#,
        ),
    ];

    for (name, source) in cases {
        let result = parse_sfc(source, Default::default())
            .unwrap_or_else(|error| panic!("{name} must parse: {error:?}"));
        let template = result
            .template
            .unwrap_or_else(|| panic!("{name} must preserve the root template"));

        assert!(
            template.content.contains("after</div>"),
            "{name} truncated the template: {}",
            template.content
        );
        assert_eq!(result.styles.len(), 1, "{name} swallowed the style block");
    }
}

#[test]
fn balances_real_nested_templates_and_custom_elements() {
    let source = r#"<template>
  <template
    v-if="show"
    data-marker="> <template> </template>"
  >
    <template-card data-close="</template>">
      <x-template>custom element</x-template>
    </template-card>
    <template data-marker="/>">
      nested template
    </template>
  </template>
  <template data-marker="</template>" />
  <div>after</div>
</template>

<docs>{"marker":"<template>"}</docs>"#;

    let result = parse_sfc(source, Default::default()).expect("adversarial SFC must parse");
    let template = result.template.expect("root template must be preserved");

    assert!(template.content.contains("<template-card"));
    assert!(template.content.contains("nested template"));
    assert!(template.content.contains("<div>after</div>"));
    assert_eq!(result.custom_blocks.len(), 1);
    assert_eq!(result.custom_blocks[0].block_type, "docs");
    assert_eq!(
        result.custom_blocks[0].content,
        r#"{"marker":"<template>"}"#
    );
}

#[test]
fn tracks_lines_across_a_multiline_nested_closing_tag() {
    let source =
        "<template>\n  <template>nested</template\n  >\n</template>\n<style>.ok {}</style>";
    let result = parse_sfc(source, Default::default()).expect("nested template must parse");

    let template = result.template.expect("root template must be preserved");
    assert_eq!(template.loc.end_line, 4);
    assert_eq!(result.styles.len(), 1);
    assert_eq!(result.styles[0].loc.start_line, 5);
}

#[test]
fn unclosed_interpolation_keeps_the_root_boundary_visible() {
    let source = "<template>\n  <div>{{ unclosed\n</template>\n<style>.ok {}</style>";
    let result = parse_sfc(source, Default::default())
        .expect("template diagnostics must not become an SFC boundary error");

    let template = result.template.expect("root template must be preserved");
    assert!(template.content.contains("{{ unclosed"));
    assert_eq!(result.styles.len(), 1);
}

#[test]
fn later_interpolations_survive_an_unclosed_scan_that_consumed_delimiters() {
    // The first `{{` never closes, but its JS scan walks over a `}}` that
    // brace depth consumes, so raw delimiters do exist ahead. The linear-scan
    // shortcut for exhausted delimiters (#3275) must not latch here: the later
    // `{{ '</template>' }}` still has to be recognized as an interpolation, or
    // the quoted closing tag would end the root template early.
    let source = concat!(
        "<template><p>{{ { }} {{ '</template>' }}</p><i>after</i></template>",
        "<style>.ok {}</style>"
    );
    let result = parse_sfc(source, Default::default())
        .expect("unclosed interpolation must not become a boundary error");

    let template = result.template.expect("root template must be preserved");
    assert!(
        template.content.contains("'</template>'"),
        "content: {}",
        template.content
    );
    assert!(template.content.contains("after"));
    assert_eq!(result.styles.len(), 1);
}

#[test]
fn valid_interpolations_survive_a_spent_failed_scan_budget() {
    // Every `{{ { }}` group leaves a brace-consumed `}}` behind, so its scan
    // walks to EOF and fails while raw delimiters still exist ahead. Enough
    // groups spend the failed-scan budget (#3275), which must bound later scans
    // rather than drop JS awareness for the rest of the block: the trailing
    // `{{ '</template>' }}` is well-formed, so its quoted closing tag has to
    // stay opaque instead of ending the root template early.
    let mut source = String::from("<template><p>");
    for _ in 0..8 {
        source.push_str("{{ { }} ");
    }
    source.push_str("{{ '</template>' }}</p><i>after</i></template><style>.ok {}</style>");

    let result = parse_sfc(&source, Default::default())
        .expect("unclosed interpolations must not become a boundary error");

    let template = result.template.expect("root template must be preserved");
    assert!(
        template.content.contains("'</template>'"),
        "content: {}",
        template.content
    );
    assert!(
        template.content.contains("<i>after</i>"),
        "content: {}",
        template.content
    );
    assert_eq!(result.styles.len(), 1);
}

#[test]
fn unclosed_interpolation_runs_scan_linearly() {
    // Fuzz slow units #3275 (runs 30073026409): 45KB with 1162 `{{` against
    // 44 `}}` made every unclosed `{{` re-walk the rest of the source through
    // the JS string/regex machinery, one full pass per occurrence. Once no
    // literal `}}` remains ahead the scan is provably unable to close, so the
    // boundary scanner skips the JS loop for every later `{{`. This canary is
    // quadratic-in-the-loop without that shortcut (tens of seconds in a debug
    // build) and linear with it; the generous bound only guards the class.
    let mut source = String::from("<template><div>");
    for _ in 0..6000 {
        source.push_str("{{a ");
    }
    source.push_str("</div></template>\n<style>.ok {}</style>");

    let started = std::time::Instant::now();
    let result = parse_sfc(&source, Default::default())
        .expect("an interpolation-dense template must still parse");
    let template = result.template.expect("root template must be preserved");
    assert!(template.content.contains("{{a"));
    assert_eq!(result.styles.len(), 1);
    assert!(
        started.elapsed() < std::time::Duration::from_secs(5),
        "boundary scan took {:?}; the unclosed-interpolation shortcut regressed",
        started.elapsed()
    );
}

#[test]
fn unclosed_interpolation_run_with_a_hidden_delimiter_stays_bounded() {
    // Variant of the #3275 shape where a late raw `}}` survives (here inside
    // the style block), so the no-delimiter-ahead shortcut never fires. Every
    // unclosed `{{` would re-enter the JS scanner and walk the tail; the
    // failed-scan budget caps the walks while the root boundary and the style
    // block still parse exactly as before.
    let mut source = String::from("<template><div>");
    for _ in 0..6000 {
        source.push_str("{{a ");
    }
    source.push_str("</div></template>\n<style>.ok {}}</style>");

    let started = std::time::Instant::now();
    let result = parse_sfc(&source, Default::default())
        .expect("an interpolation-dense template must still parse");
    let template = result.template.expect("root template must be preserved");
    assert!(template.content.contains("{{a"));
    assert_eq!(result.styles.len(), 1);
    assert!(result.styles[0].content.contains(".ok"));
    assert!(
        started.elapsed() < std::time::Duration::from_secs(5),
        "boundary scan took {:?}; the failed-interpolation-scan budget regressed",
        started.elapsed()
    );
}

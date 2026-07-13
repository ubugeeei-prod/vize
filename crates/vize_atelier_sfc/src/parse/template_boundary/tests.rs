use crate::parse_sfc;

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

//! Multiline template-literal content inside interpolations must be preserved
//! verbatim so its semantically significant value is not corrupted and
//! formatting stays a fixed point. (#3247)

use vize_glyph::{FormatOptions, format_sfc, format_template};

#[test]
fn multiline_template_literal_in_interpolation_is_idempotent() {
    // A multiline template literal inside an interpolation carries
    // semantically significant string content. Re-indenting its inner lines
    // corrupts the value and, because each pass prepends indentation again, is
    // not a fixed point. Quasi lines must be preserved verbatim. (#3247)
    let source = concat!(
        "<div>\n",
        "  {{\n",
        "    items\n",
        "      .map((p) => `\n",
        "${p} [data-chart] {\n",
        "  color: red;\n",
        "}\n",
        "`)\n",
        "      .join(\"\\n\")\n",
        "  }}\n",
        "</div>",
    );
    let options = FormatOptions::default();
    let first = format_template(source, &options).unwrap();
    let second = format_template(&first, &options).unwrap();
    assert_eq!(
        first, second,
        "multiline template-literal interpolation must be idempotent;\nfirst:\n{first}\nsecond:\n{second}"
    );
    assert!(
        first.contains("\n${p} [data-chart] {\n  color: red;\n}\n"),
        "template-literal quasi must be preserved verbatim; got:\n{first}"
    );
}

#[test]
fn nested_multiline_template_literal_interpolation_is_idempotent() {
    // The shadcn-vue `ChartStyle` shape: an interpolation whose expression
    // builds CSS with an outer multiline template literal that itself contains
    // a multiline `${ … }` holding a nested template literal. oxc canonicalizes
    // the `${ … }` code each pass while the quasi text stays verbatim, so
    // re-indenting only non-quasi lines keeps the whole thing a fixed point.
    // (#3247)
    let source = concat!(
        "<Primitive v-if=\"colorConfig.length\" as=\"style\">\n",
        "  {{ Object.entries(THEMES)\n",
        "    .map(\n",
        "      ([theme, prefix]) => `\n",
        "${prefix} [data-chart=${id}] {\n",
        "${colorConfig\n",
        "  .map(([key, itemConfig]) => {\n",
        "    const color = itemConfig.color\n",
        "    return color ? `  --color-${key}: ${color};` : null\n",
        "  })\n",
        "      .join(\"\\n\")}\n",
        "}\n",
        "`,\n",
        "    )\n",
        "    .join(\"\\n\") }}\n",
        "</Primitive>\n",
    );
    let options = FormatOptions::default();
    let first = format_template(source, &options).unwrap();
    let second = format_template(&first, &options).unwrap();
    assert_eq!(
        first, second,
        "nested multiline template literal must be idempotent;\nfirst:\n{first}\nsecond:\n{second}"
    );
    assert!(
        first.contains("\n${prefix} [data-chart=${id}] {\n"),
        "outer template-literal quasi must be preserved verbatim; got:\n{first}"
    );
}

#[test]
fn plain_multiline_interpolation_still_indents() {
    // Regression guard: an interpolation with no template literal is still
    // indented under `{{` at depth+1, and each wrapped line keeps oxc's
    // relative indentation, staying idempotent. (#3247)
    let source = concat!(
        "<span>\n",
        "  {{\n",
        "    reallyQuiteLongConditionName\n",
        "      ? firstBranchValueHere\n",
        "      : secondBranchValueHere\n",
        "  }}\n",
        "</span>",
    );
    let options = FormatOptions::default();
    let first = format_template(source, &options).unwrap();
    // The expression body is indented under `{{` at depth+1 (4 spaces),
    // regardless of whether oxc keeps it broken or collapses it.
    assert!(
        first.contains("\n    reallyQuiteLongConditionName"),
        "plain multiline interpolation body must be indented under {{; got:\n{first}"
    );
    assert_eq!(
        format_template(&first, &options).unwrap(),
        first,
        "plain multiline interpolation must be idempotent"
    );
}

#[test]
fn literal_open_delimiters_keep_their_complete_authored_values() {
    let options = FormatOptions::default();
    for (source, expected) in [
        (r#"{{'{{'}}"#, r#"{{ "{{" }}"#),
        (r#"<p>{{'{{<i>'}}</p>"#, r#"<p>{{ "{{<i>" }}</p>"#),
        (r#"{{'{{'}}{{ value }}"#, r#"{{ "{{" }}{{ value }}"#),
        (r#"{{ `{{<i>` }}"#, r#"{{ `{{<i>` }}"#),
        ("{{ value", "{{ value"),
    ] {
        let first = format_template(source, &options).unwrap();
        assert_eq!(first, expected, "complete source: {source}");
        assert_eq!(format_template(&first, &options).unwrap(), expected);
    }
}

#[test]
fn multiline_literal_open_delimiters_preserve_crlf_and_quote_policy() {
    let options = FormatOptions {
        end_of_line: vize_glyph::EndOfLine::Crlf,
        single_quote: true,
        ..FormatOptions::default()
    };
    let source = "<p>\r\n  {{\r\n    \"{{<i>\"\r\n  }}\r\n</p>";
    let expected = "<p>\r\n  {{\r\n    '{{<i>'\r\n  }}\r\n</p>";
    let first = format_template(source, &options).unwrap();
    assert_eq!(first, expected);
    assert_eq!(format_template(&first, &options).unwrap(), expected);
}

#[test]
fn sfc_literal_open_delimiters_stay_inside_the_authored_interpolation() {
    let source = r#"<template><p>{{'{{<i>'}}</p></template>"#;
    let expected = "<template>\n  <p>{{ \"{{<i>\" }}</p>\n</template>\n";
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    assert_eq!(first.code.as_str(), expected);
    assert!(first.changed);
    let second = format_sfc(expected, &options).unwrap();
    assert_eq!(second.code.as_str(), expected);
    assert!(!second.changed);
}

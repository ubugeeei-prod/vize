//! A multi-line template literal in a directive value must survive `vize fmt`
//! byte-for-byte.
//!
//! Every byte between the backticks is part of the string's runtime value. The
//! attribute writer used to strip the leading whitespace off those lines and
//! re-indent them to the attribute's depth, which changed the rendered string
//! *and* moved the column at which every embedded `${…}` starts. The expression
//! formatter measures its line-break budget from that column, so the next pass
//! read back a differently-indented literal and made a different wrap decision:
//! `fmt(fmt(x)) != fmt(x)`. (#3379)

#![expect(clippy::unwrap_used, reason = "tests assert by panicking")]

use vize_glyph::{EndOfLine, FormatOptions, format_sfc};

/// Format `source` three times: pass 1 must equal `expected`, and passes 2 and
/// 3 must reproduce it byte-for-byte.
#[track_caller]
fn assert_stable(source: &str, expected: &str) {
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    assert_eq!(first.code.as_str(), expected, "pass 1 output");
    let second = format_sfc(&first.code, &options).unwrap();
    assert_eq!(second.code, first.code, "fmt; fmt must be a no-op");
    let third = format_sfc(&second.code, &options).unwrap();
    assert_eq!(third.code, second.code, "fmt must stay at its fixed point");
}

#[test]
fn an_and_chain_in_a_ternary_arm_inside_a_template_literal_is_stable() {
    // Minimized from vue-data-ui's `vue-ui-flow.vue`, the file this issue
    // pinned. The literal's own indentation is deep enough that the `&&` chain
    // does not fit on one line; flattening the quasi lines moved `${` sixteen
    // columns left, so pass 2 joined what pass 1 had split.
    let source = r#"<template>
  <path
    :style="`
                        opacity:${
                            selectedNodes
                                ? selectedNodes.includes(path.source) &&
                                  selectedNodes.includes(path.target)
                                    ? 1
                                    : 0.3
                                : 1
                        }
                    `"
  />
</template>
"#;
    assert_stable(
        source,
        r#"<template>
  <path
    :style="`
                        opacity:${
                          selectedNodes
                            ? selectedNodes.includes(path.source) &&
                              selectedNodes.includes(path.target)
                              ? 1
                              : 0.3
                            : 1
                        }
                    `"
  />
</template>
"#,
    );
}

#[test]
fn a_ternary_nested_in_a_ternary_inside_a_template_literal_is_stable() {
    let source = r#"<template>
  <i
    :class="`
            a-${
                x ? (y ? 1 : 2) : (z ? 3 : 4)
            }-b
        `"
  />
</template>
"#;
    assert_stable(
        source,
        r#"<template>
  <i
    :class="`
            a-${x ? (y ? 1 : 2) : z ? 3 : 4}-b
        `"
  />
</template>
"#,
    );
}

#[test]
fn an_or_chain_in_a_ternary_test_inside_a_template_literal_is_stable() {
    let source = r#"<template>
  <i
    :style="`
            opacity:${
                aVeryLongConditionName || anotherVeryLongConditionName || aThirdOne ? 1 : 0
            }
        `"
  />
</template>
"#;
    assert_stable(
        source,
        r#"<template>
  <i
    :style="`
            opacity:${aVeryLongConditionName || anotherVeryLongConditionName || aThirdOne ? 1 : 0}
        `"
  />
</template>
"#,
    );
}

#[test]
fn a_template_literal_without_substitutions_keeps_its_content() {
    // No `${…}` to re-measure here — this is the plain rendered-output half of
    // the same defect: the class list is a string, and its indentation used to
    // be rewritten.
    let source = r#"<template>
  <i
    :class="`
        w-full
        px5
    `"
  />
</template>
"#;
    assert_stable(source, source);
}

#[test]
fn template_literals_in_both_ternary_arms_keep_their_content() {
    // The #1965 fixture, which the flattening was originally added for. It is
    // idempotent either way; what changes is that the emitted class strings are
    // now the ones the source wrote.
    let source = r#"<template>
  <NuxtLink
    :class="isSmallScreen
      ? `
        w-full
        px5 sm:mxa
      `
      : `
        w-fit rounded-3
        px2 mx3 sm:mxa
      `"
  />
</template>
"#;
    assert_stable(source, source);
}

/// Like [`assert_stable`], for inputs already at the fixed point.
#[track_caller]
fn assert_unchanged(source: &str) {
    assert_stable(source, source);
}

#[test]
fn a_quoted_backtick_before_a_multiline_literal_keeps_every_line_in_place() {
    // #9583: the value is owned by its parsed literal spans, so the backtick
    // in 'a ` b' does not flip later lines into raw ones, inside the value or
    // after it.
    assert_unchanged(
        r#"<template>
  <div>
    <i
      :title="[
        'a ` b',
        `
          x
        `,
      ]"
    />
    <p>
      <span>after</span>
    </p>
  </div>
</template>
"#,
    );
}

#[test]
fn a_backtick_in_a_block_comment_before_a_multiline_literal_is_not_a_literal() {
    assert_unchanged(
        r#"<template>
  <i
    :class="[
      /* `not a literal */ a,
      `
        keep   this
      `,
    ]"
  />
</template>
"#,
    );
}

#[test]
fn an_entity_quoted_string_with_a_backtick_does_not_own_later_lines() {
    assert_stable(
        "<template>\n  <i\n    :title=\"cond\n      ? &quot;a  `  b&quot;\n      : `\n          x\n        `\"\n  />\n</template>\n",
        "<template>\n  <i\n    :title=\"cond\n      ? 'a  `  b'\n      : `\n          x\n        `\"\n  />\n</template>\n",
    );
}

#[test]
fn a_backtick_in_a_line_comment_does_not_open_a_literal() {
    assert_unchanged(
        r#"<template>
  <button
    @click="
      // toggle `open` state
      open = !open
    "
  />
</template>
"#,
    );
}

#[test]
fn a_regexp_with_backticks_after_an_assignment_is_not_a_literal() {
    assert_unchanged(
        r#"<template>
  <button
    @click="
      pattern = /`[a-z]+`/
      label = `
        raw   text
      `
    "
  />
</template>
"#,
    );
}

#[test]
fn an_escaped_backtick_keeps_the_literal_open() {
    assert_unchanged(
        r#"<template>
  <i
    :title="`
      a \` b
         c
    `"
  />
</template>
"#,
    );
}

#[test]
fn an_object_literal_in_a_substitution_does_not_close_the_literal_early() {
    // The `}` of `{ b: 1 }` must not end `${…}`, or the trailing quasi line
    // would be re-indented into the string's value.
    assert_stable(
        r#"<template>
  <i
    :title="`
      a ${ { b: 1 }.b }
        c
    `"
  />
</template>
"#,
        r#"<template>
  <i
    :title="`
      a ${{ b: 1 }.b}
        c
    `"
  />
</template>
"#,
    );
}

#[test]
fn a_literal_after_a_regexp_comparison_keeps_its_content() {
    assert_stable(
        r#"<template>
  <i
    :title="
      a == /x/ ? `
        k
      ` : b
    "
  />
</template>
"#,
        r#"<template>
  <i
    :title="a == /x/
      ? `
        k
      `
      : b"
  />
</template>
"#,
    );
}

#[test]
fn an_interpolated_multiline_literal_keeps_its_content() {
    assert_stable(
        "<template>\n  <p>{{ `\n      line one\n        line two\n  ` }}</p>\n</template>\n",
        "<template>\n  <p>{{\n    `\n      line one\n        line two\n  `\n  }}</p>\n</template>\n",
    );
}

#[test]
fn a_crlf_literal_keeps_its_content_and_line_endings() {
    let source = "<template>\r\n  <i\r\n    :class=\"`\r\n        w-full\r\n          px5\r\n    `\"\r\n  />\r\n</template>\r\n";
    let options = FormatOptions {
        end_of_line: EndOfLine::Crlf,
        ..FormatOptions::default()
    };
    let first = format_sfc(source, &options).unwrap();
    assert_eq!(first.code.as_str(), source);
    assert!(!first.changed);
}

#![expect(clippy::disallowed_macros, reason = "test fixtures use format!")]
//! Whitespace a hugged child prints with decides the opening-tag break the
//! same way on every pass. Shapes reduced from real-project idempotence
//! failures (nutui, vue-flow, gogocode, vue-netcore, elk).
use vize_glyph::{FormatOptions, format_sfc};

fn assert_fixed_point(input: &str, expected: &str, options: &FormatOptions) {
    let first = format_sfc(input, options).expect("first pass");
    assert_eq!(first.code.as_str(), expected, "first pass");
    let second = format_sfc(&first.code, options).expect("second pass");
    assert_eq!(second.code.as_str(), expected, "second pass");
    assert!(!second.changed, "second pass reports no change");
}

#[test]
fn whitespace_only_child_measures_like_an_empty_child() {
    let options = FormatOptions::default();
    let line = r#"<Panel ref="panelEl" position="bottom-right" :class="{ intersecting: isIntersectingWithPanel }">"#;
    let expected = format!("<template>\n  <div>\n    {line}</Panel>\n  </div>\n</template>\n");
    for child in ["", " ", "  ", "\t", " \t "] {
        let input =
            format!("<template>\n  <div>\n    {line}{child}</Panel>\n  </div>\n</template>\n");
        assert_fixed_point(&input, &expected, &options);
    }
}

#[test]
fn whitespace_only_child_after_an_inline_sibling() {
    let options = FormatOptions::default();
    let input = concat!(
        "<template>\n",
        "  <div>\n",
        "    <el-button plain size=\"small\" type=\"primary\" @click=\"preview(true)\"",
        "><i class=\"el-icon-view-ab\"> </i>Prev</el-button>\n",
        "  </div>\n",
        "</template>\n",
    );
    let expected = concat!(
        "<template>\n",
        "  <div>\n",
        "    <el-button plain size=\"small\" type=\"primary\" @click=\"preview(true)\"",
        "><i class=\"el-icon-view-ab\"></i>Prev</el-button>\n",
        "  </div>\n",
        "</template>\n",
    );
    assert_fixed_point(input, expected, &options);
}

#[test]
fn whitespace_significant_children_still_count_their_whitespace() {
    let options = FormatOptions::default();
    let textarea = concat!(
        "<template>\n",
        "  <div>\n",
        "    <textarea ref=\"panelEl\" name=\"bottom-right\" ",
        ":class=\"{ intersecting: isIntersectingWithPanelX }\"> </textarea>\n",
        "  </div>\n",
        "</template>\n",
    );
    let textarea_expected = concat!(
        "<template>\n",
        "  <div>\n",
        "    <textarea ref=\"panelEl\" name=\"bottom-right\" ",
        ":class=\"{ intersecting: isIntersectingWithPanelX }\"\n",
        "    > </textarea>\n",
        "  </div>\n",
        "</template>\n",
    );
    assert_fixed_point(textarea, textarea_expected, &options);

    let v_pre = concat!(
        "<template>\n",
        "  <div>\n",
        "    <div v-pre title=\"bottom-right\" ",
        "class=\"{ intersecting: isIntersectingWithPanelXYZ }\" lang=\"en\"> </div>\n",
        "  </div>\n",
        "</template>\n",
    );
    let v_pre_expected = concat!(
        "<template>\n",
        "  <div>\n",
        "    <div v-pre class=\"{ intersecting: isIntersectingWithPanelXYZ }\" lang=\"en\" ",
        "title=\"bottom-right\"\n",
        "    > </div>\n",
        "  </div>\n",
        "</template>\n",
    );
    assert_fixed_point(v_pre, v_pre_expected, &options);
}

#[test]
fn overflowing_text_child_still_breaks_the_opening_tag() {
    let options = FormatOptions::default();
    let title = "a".repeat(80);
    let input = format!(
        "<template>\n  <div>\n    <span title=\"{title}\"> x</span>\n  </div>\n</template>\n"
    );
    let expected = format!(
        "<template>\n  <div>\n    <span title=\"{title}\"\n    > x</span>\n  </div>\n</template>\n"
    );
    assert_fixed_point(&input, &expected, &options);
}

const LONG: &str =
    "!threadIsSending ? $t('action.publish_thread') : $t('state.publishing') + extraWords";
const LONG_PRINTED: &str =
    "!threadIsSending ? $t(\"action.publish_thread\") : $t(\"state.publishing\") + extraWords";

fn nested(body: &str) -> String {
    format!(
        "<template>\n  <div>\n    <div>\n      <div>\n{body}      </div>\n    </div>\n  </div>\n</template>\n"
    )
}

#[test]
fn sole_interpolation_with_a_whitespace_tail_wraps_inside_the_mustache() {
    let options = FormatOptions::default();
    let expected = nested(&format!(
        "        <span>{{{{\n          {LONG_PRINTED}\n        }}}} </span>\n"
    ));
    for tail in [" ", "  "] {
        let input = nested(&format!("        <span>{{{{ {LONG} }}}}{tail}</span>\n"));
        assert_fixed_point(&input, &expected, &options);
    }
}

#[test]
fn sole_interpolation_with_a_whitespace_tail_keeps_a_break_that_fits() {
    let options = FormatOptions::default();
    let open =
        r#"<view class="nut-tabs__titles-item__text-and-more" :class="{ ellipsis: ellipsis }""#;
    let input = nested(&format!("        {open}>{{{{ item.title }}}} </view>\n"));
    let expected = nested(&format!(
        "        {open}\n        >{{{{ item.title }}}} </view>\n"
    ));
    assert_fixed_point(&input, &expected, &options);
}

#[test]
fn whitespace_tail_neighbours_keep_their_layout() {
    let options = FormatOptions::default();
    // No tail: the established sole-interpolation wrap.
    let sole = nested(&format!("        <span>{{{{ {LONG} }}}}</span>\n"));
    let sole_expected = nested(&format!(
        "        <span>{{{{\n          {LONG_PRINTED}\n        }}}}</span>\n"
    ));
    assert_fixed_point(&sole, &sole_expected, &options);
    // Runtime text after the interpolation stays mixed text.
    let mixed = nested(&format!("        <span>{{{{ {LONG} }}}} x</span>\n"));
    let mixed_expected = nested(&format!(
        "        <span\n        >{{{{ {LONG_PRINTED} }}}} x</span>\n"
    ));
    assert_fixed_point(&mixed, &mixed_expected, &options);
}

#[test]
fn whitespace_tail_measures_tab_indentation() {
    // Three tabs are six columns: the 101-column continuation wraps inside the
    // mustache, the 98-column one keeps the opening-tag break.
    let options = FormatOptions {
        use_tabs: true,
        ..FormatOptions::default()
    };
    let tabbed = |body: &str| {
        format!("<template>\n\t<div>\n\t\t<div>\n{body}\t\t</div>\n\t</div>\n</template>\n")
    };
    let printed = |tail: &str| {
        format!(
            "threadIsSending ? $t(\"action.publish_thread\") : $t(\"state.publishing\") + \"{tail}\""
        )
    };
    let source = |tail: &str| {
        format!(
            "\t\t\t<span>{{{{ threadIsSending ? $t('action.publish_thread') : $t('state.publishing') + '{tail}' }}}} </span>\n"
        )
    };
    assert_fixed_point(
        &tabbed(&source("abcdef")),
        &tabbed(&format!(
            "\t\t\t<span>{{{{\n\t\t\t\t{}\n\t\t\t}}}} </span>\n",
            printed("abcdef")
        )),
        &options,
    );
    assert_fixed_point(
        &tabbed(&source("abc")),
        &tabbed(&format!(
            "\t\t\t<span\n\t\t\t>{{{{ {} }}}} </span>\n",
            printed("abc")
        )),
        &options,
    );
}

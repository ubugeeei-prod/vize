//! Whole public SFC output for layout fix-history regressions (#6882).
#![expect(clippy::unwrap_used, reason = "history fixtures assert by panicking")]
use vize_glyph::{FormatOptions, format_sfc};

fn assert_history(name: &str, source: &str) {
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    insta::with_settings!({prepend_module_to_snapshot => false}, {
        insta::assert_binary_snapshot!(name, first.code.as_bytes().to_vec());
    });
    let second = format_sfc(&first.code, &options).unwrap();
    let third = format_sfc(&second.code, &options).unwrap();
    assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
    assert_eq!(second.code, third.code, "fmt must stay at its fixed point");
}

#[test]
fn literal_multiline_attribute_has_complete_first_output() {
    // b9b18337e: test_format_sfc_multiline_attribute_value_is_idempotent.
    let source = "<template>\n  <button\n    class=\"\n      flex items-center gap-2\n      text-start text-lg\n    \"\n    @click=\"go\"\n  >\n    hi\n  </button>\n</template>\n";
    assert_history("history_sfc_layout_literal_multiline_attribute.txt", source);
}

#[test]
fn multiline_comment_has_complete_first_output() {
    // 822db1ca9: test_format_sfc_multiline_comment_is_idempotent.
    let source = "<template>\n  <div>\n    <!-- <div v-if=\"result.action\">\n      {{ result.action!.label }}\n    </div> -->\n    <span>hi</span>\n  </div>\n</template>\n";
    assert_history("history_sfc_layout_multiline_comment.txt", source);
}

#[test]
fn multiline_pre_opening_has_complete_first_output() {
    // 822db1ca9: test_format_sfc_multiline_pre_open_tag_is_idempotent.
    let source = "<template>\n  <div>\n    <pre\n      v-else-if=\"parsedJSON\"\n      class=\"overflow-auto max-h-96\"\n    >{{ formattedJSONString }}\n      </pre>\n  </div>\n</template>\n";
    assert_history("history_sfc_layout_multiline_pre_opening.txt", source);
}

#[test]
fn interpolation_trailing_text_has_complete_first_output() {
    // 822db1ca9: test_format_sfc_multiline_interpolation_with_trailing_text_is_idempotent.
    let source = "<template>\n  <div>\n    <span>\n      {{ $t(\"compose.drafts\", nonEmptyDrafts.length, { named: { v: formatNumber(nonEmptyDrafts.length) } }) }}&#160;\n    </span>\n  </div>\n</template>\n";
    assert_history("history_sfc_layout_interpolation_trailing_text.txt", source);
}

#[test]
fn text_between_interpolations_has_complete_first_output() {
    // 822db1ca9: test_format_sfc_text_between_interpolations_is_idempotent.
    let source = "<template>\n  <span>\n    {{ tsx.compressedToX({ x: bytes(item.compressedSize), yyyyyyyyyyyyyyyy: zzzzzzzzzzzzzz }) }} = {{ tsx.savedXPercent({ x: Math.round((1 - item.compressedSize / item.file.size) * 100) }) }}\n  </span>\n</template>\n";
    assert_history("history_sfc_layout_text_between_interpolations.txt", source);
}

#[test]
fn wrapped_interpolation_has_complete_first_output() {
    // 4abb3d0ec: test_format_sfc_multiline_interpolation_is_idempotent.
    let source = "<template>\n  <div>\n    <span>{{ new Date(version.date).toLocaleDateString(\"en-US\", { year: \"numeric\", month: \"short\", day: \"numeric\" }) }}</span>\n  </div>\n</template>\n";
    assert_history("history_sfc_layout_wrapped_interpolation.txt", source);
}

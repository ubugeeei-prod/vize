//! `v-pre` regions must survive `vize fmt` byte-for-byte.
//!
//! The template formatter copies a whitespace-significant element's content
//! verbatim, and the SFC layer then indents the template block by one level.
//! `<pre>` and `<textarea>` were carved out of that indent step; `v-pre` — which
//! makes *any* element whitespace-significant — was not, so every run pushed the
//! content two more columns right. The drift was unbounded, not a one-off
//! reformat: pass 2 differed from pass 1 and pass 3 from pass 2. (#3379)

use vize_glyph::{FormatOptions, format_sfc};

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
fn v_pre_interpolation_child_keeps_its_indentation() {
    // Minimized from scalar's `Environment.vue`, the file this issue pinned:
    // an interpolation that is the sole child of a `v-pre` element. `{{ … }}`
    // inside `v-pre` is literal text at runtime, so its leading whitespace is
    // rendered output and must not move.
    let source = "<template>\n  <code v-pre>\n    {{ variable }}\n  </code>\n</template>\n";
    assert_stable(source, source);
}

#[test]
fn v_pre_on_a_split_opening_tag_keeps_its_indentation() {
    // scalar's original spelling: the directive sits on its own line of a
    // multi-line opening tag, so the mask has to carry "this element is
    // `v-pre`" across the line break while remembering the tag's name.
    let source = "<template>\n  <code\n    v-pre\n    class=\"font-code\">\n    {{ variable }}\n  </code>\n</template>\n";
    assert_stable(
        source,
        "<template>\n  <code v-pre class=\"font-code\">\n    {{ variable }}\n  </code>\n</template>\n",
    );
}

#[test]
fn v_pre_element_nesting_a_same_name_element_ends_at_the_outer_close() {
    // The inner `</div>` must not end the raw region, or every line after it
    // would pick up SFC indentation again.
    let source = "<template>\n  <div v-pre>\n    <div>x</div>\n    y\n  </div>\n</template>\n";
    assert_stable(source, source);
}

#[test]
fn v_pre_with_other_attributes_is_stable_without_crossing_the_directive() {
    let source = "<template>\n  <code class=\"a\" v-pre>{{ v }}</code>\n</template>\n";
    assert_stable(source, source);
}

#[test]
fn a_self_closing_v_pre_element_opens_no_raw_region() {
    // `<Foo v-pre />` has no content, so the siblings after it are ordinary
    // markup and must still be indented into the template block.
    let source = "<template>\n  <Foo v-pre />\n  <span>\n    a\n  </span>\n</template>\n";
    assert_stable(source, source);
}

#[test]
fn a_data_v_pre_attribute_is_not_the_directive() {
    // Negative guard: `data-v-pre` is a plain attribute, so the element's
    // content is ordinary markup the formatter owns.
    let source = "<template>\n  <div data-v-pre>\n    a\n  </div>\n</template>\n";
    assert_stable(source, source);
}

#[test]
fn an_interpolation_alone_in_a_pre_element_keeps_its_indentation() {
    // Neighborhood guard for the same boundary in the two elements that are
    // whitespace-significant by name.
    let pre = "<template>\n  <pre>\n    {{ variable }}\n  </pre>\n</template>\n";
    assert_stable(pre, pre);

    let textarea = "<template>\n  <textarea>\n    {{ variable }}\n  </textarea>\n</template>\n";
    assert_stable(textarea, textarea);
}

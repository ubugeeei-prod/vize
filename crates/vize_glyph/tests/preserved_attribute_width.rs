//! Preserve-mode text and actual SFC attribute columns share one layout context.
use vize_glyph::{Allocator, FormatOptions, GlyphFormatter};

#[test]
fn preserved_text_keeps_whole_bytes_beside_wrapped_attribute_values() {
    for (use_tabs, source, expected) in [
        (
            false,
            "<template>\n  <button v-bind:title=\"alpha + beta + gamma + delta\">  two  spaces  </button>\n</template>\n",
            "<template>\n  <button\n    :title=\"\n      alpha + beta + gamma + delta\n    \"\n  >  two  spaces  </button>\n</template>\n",
        ),
        (
            true,
            "<template>\n\t<button v-bind:title=\"alpha + beta + gamma + delta\">  two  spaces  </button>\n</template>\n",
            "<template>\n\t<button\n\t\t:title=\"\n\t\t\talpha + beta + gamma + delta\n\t\t\"\n\t>  two  spaces  </button>\n</template>\n",
        ),
    ] {
        let options = FormatOptions {
            print_width: 40,
            use_tabs,
            tab_width: if use_tabs { 4 } else { 2 },
            ..FormatOptions::default()
        };
        let allocator = Allocator::default();
        let formatter =
            GlyphFormatter::new(&options, &allocator).with_template_whitespace_preserved(true);
        let first = formatter.format(source).unwrap();
        assert_eq!(first.code.as_str(), expected);
        assert!(first.changed);
        for _ in 0..2 {
            let next = formatter.format(expected).unwrap();
            assert_eq!(next.code.as_str(), expected);
            assert!(!next.changed);
        }
    }
}

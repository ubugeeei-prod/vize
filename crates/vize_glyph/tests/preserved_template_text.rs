//! #7871: retain rendered whitespace while continuing to format tag syntax.
use vize_glyph::{Allocator, EndOfLine, FormatOptions, GlyphFormatter};

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/sfc-preserved-template-text-7871/App.vue.txt"
);

fn assert_fixed_point(source: &str, expected: &str, options: &FormatOptions) {
    let allocator = Allocator::default();
    let formatter =
        GlyphFormatter::new(options, &allocator).with_template_whitespace_preserved(true);
    let first = formatter.format(source).unwrap();
    assert_eq!(first.code.as_str(), expected);
    assert_eq!(first.changed, source != expected);
    for _ in 0..2 {
        let next = formatter.format(expected).unwrap();
        assert_eq!(next.code.as_str(), expected);
        assert!(!next.changed);
    }
}

#[test]
fn original_and_inline_text_boundaries_keep_their_complete_bytes() {
    for source in [
        ORIGINAL,
        "<template><p>  first\t<i>  nested  </i>\t last  </p></template>\n",
        "<template>\n  <p>\n    first\n\n      second  \n  </p>\n</template>\n",
        "<template><p>  {{ value < limit ? '<i>' : '<b>' }}  </p></template>\n",
        "<template>\n  <!-- keep <i> and {{ raw }} -->\n  <p>&nbsp;  日本語  </p>\n</template>\n",
        "<template>\n  <pre>  raw\n  {{  raw  }}<i  id='raw'> keep </i></pre>\n</template>\n",
        "<template>\n  <div v-pre>  {{  raw  }}<i  id='raw'> keep </i>  </div>\n</template>\n",
    ] {
        assert_fixed_point(source, source, &FormatOptions::default());
        let crlf = source.replace('\n', "\r\n");
        assert_fixed_point(
            &crlf,
            &crlf,
            &FormatOptions {
                end_of_line: EndOfLine::Auto,
                ..FormatOptions::default()
            },
        );
    }
}

#[test]
fn preserved_text_does_not_disable_attribute_formatting() {
    assert_fixed_point(
        "<template>\n  <button  v-bind:title=\"label\">  two  spaces  </button>\n</template>\n",
        "<template>\n  <button :title=\"label\">  two  spaces  </button>\n</template>\n",
        &FormatOptions::default(),
    );
    assert_fixed_point(
        "<template>\n  <button title=\"one\" id=\"two\">  keep  </button>\n</template>\n",
        "<template>\n  <button\n    id=\"two\"\n    title=\"one\"\n  >  keep  </button>\n</template>\n",
        &FormatOptions {
            single_attribute_per_line: true,
            ..FormatOptions::default()
        },
    );
}

#[test]
fn default_formatter_still_condenses_boundaries() {
    let allocator = Allocator::default();
    let options = FormatOptions::default();
    let output = GlyphFormatter::new(&options, &allocator)
        .format(ORIGINAL)
        .unwrap();
    assert_eq!(
        output.code.as_str(),
        "<template>\n  <button type=\"button\"> two  spaces </button>\n</template>\n"
    );
}

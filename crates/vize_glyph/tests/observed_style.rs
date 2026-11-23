//! Compare the original style writer to the public formatter on the exact
//! selected tag. The document supplies the newline policy in auto mode.
use std::path::Path;
use vize_glyph::{EndOfLine, FormatOptions, GlyphFormatter};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::container::vue::{DescriptorOptions, Vue};

fn view_options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: Default::default(),
    }
}
fn compare(source: &str, options: &FormatOptions, stock_options: &FormatOptions) -> usize {
    let arena = Allocator::default();
    let owner = Vue.observe_descriptor(&arena, source, view_options());
    let Ok(view) = owner.admitted() else { return 0 };
    let observed = GlyphFormatter::new(options, &arena);
    let stock = GlyphFormatter::new(stock_options, &arena);
    let mut count = 0;
    for style in view.styles() {
        let span = style.original_block().authored_span();
        let input = source.get(span.start as usize..span.end as usize).unwrap();
        let expected = stock.format(input).unwrap();
        let expected = expected
            .code
            .strip_suffix(stock_options.newline_string())
            .unwrap();
        let actual = observed.format_observed_style(style).unwrap();
        assert_eq!(actual.code.as_str(), expected, "{input}");
        assert_eq!(actual.changed, input != expected);
        count += 1;
    }
    count
}

#[test]
fn style_kinds_header_policies_and_whole_document_newlines_match_the_existing_writer() {
    for newline in [EndOfLine::Lf, EndOfLine::Cr, EndOfLine::Crlf] {
        for sort in [false, true] {
            for indent in [false, true] {
                let fixed = FormatOptions {
                    end_of_line: newline,
                    sort_attributes: sort,
                    vue_indent_script_and_style: indent,
                    ..Default::default()
                };
                let auto = FormatOptions {
                    end_of_line: EndOfLine::Auto,
                    ..fixed.clone()
                };
                let source = format!(
                    "<template>🦀</template>{}<style scoped lang='scss'>.x{{color:red}}</style><style module lang='less'>@x: red; .b{{ color: @x; }}</style><style>.a{{background:url('あ')}}</style><style></style>",
                    fixed.newline_string()
                );
                assert_eq!(compare(&source, &auto, &fixed), 4);
            }
        }
    }
}

#[test]
#[ignore = "requires the tracked or external Vue corpus"]
fn stock_block_bytes_on_original_style_views() {
    let manifest = std::fs::read(std::env::var_os("VIZE_FORMAT_SCSS_INPUTS").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let settings = FormatOptions {
        end_of_line: EndOfLine::Lf,
        ..Default::default()
    };
    let mut files = 0;
    let mut styles = 0;
    for name in manifest.split(|b| *b == 0).filter(|p| !p.is_empty()) {
        let p = std::str::from_utf8(name).unwrap();
        let bytes = std::fs::read(root.join(p)).unwrap();
        if let Ok(text) = std::str::from_utf8(&bytes) {
            files += 1;
            styles += compare(text, &settings, &settings);
        }
    }
    assert!(styles > 0);
    println!("stock block output: files={files} styles={styles}");
}

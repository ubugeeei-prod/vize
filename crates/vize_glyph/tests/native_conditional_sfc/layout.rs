use super::{assert_output, options};
use vize_glyph::native_doc::LineEnding;
use vize_l0::Allocator;

#[test]
fn outer_suffix_and_closing_quote_lookahead_choose_the_complete_width_boundary() {
    let source = "<template><p v-if='a+b'/></template><!--tail-->";
    for (width, expected) in [
        (50, "<template><p v-if='a + b' /></template><!--tail-->"),
        (49, "<template><p\n  v-if='a + b'\n/></template><!--tail-->"),
        (14, "<template><p\n  v-if='a + b'\n/></template><!--tail-->"),
        (
            13,
            "<template><p\n  v-if='a +\n    b'\n/></template><!--tail-->",
        ),
        (
            0,
            "<template><p\n  v-if='a +\n    b'\n/></template><!--tail-->",
        ),
    ] {
        // The complete flat source has 50 scalars. On the broken header line,
        // indent2 + v-if=' + a + b + closing quote needs exactly 14 scalars.
        let arena = Allocator::default();
        let owner = assert_output(&arena, source, expected, options(width, 2, LineEnding::Lf));
        assert_eq!(owner.attribute_operands().len(), 1);
        assert!(owner.operands().is_empty());
        assert_eq!(owner.attribute_operands()[0].raw_value(), "a+b");
    }
}

#[test]
fn nested_attribute_and_interpolation_continuations_share_captured_indent_and_endings() {
    let source = "<template><section><p v-if='a+b'>{{1n}}</p></section></template>";
    let flat = "<template><section><p v-if='a + b'>{{ 1n }}</p></section></template>";
    for indent in [0, 2, 4] {
        for (ending, nl) in [(LineEnding::Lf, "\n"), (LineEnding::CrLf, "\r\n")] {
            let attribute = " ".repeat(2 * indent);
            let continuation = " ".repeat(3 * indent);
            let opening_close = " ".repeat(indent);
            let interpolation_close = " ".repeat(2 * indent);
            let broken = format!(
                "<template><section><p{nl}{attribute}v-if='a +{nl}{continuation}b'{nl}{opening_close}>{{{{{nl}{continuation}1n{nl}{interpolation_close}}}}}</p></section></template>"
            );
            for (width, expected) in [(200, flat), (0, broken.as_str())] {
                let arena = Allocator::default();
                let owner = assert_output(&arena, source, expected, options(width, indent, ending));
                assert_eq!(owner.attribute_operands().len(), 1);
                assert_eq!(owner.operands().len(), 1);
                assert_eq!(owner.attribute_operands()[0].raw_value(), "a+b");
                assert_eq!(owner.operands()[0].raw_content(), "1n");
            }
        }
    }
}

use super::{assert_output, options};
use vize_glyph::native_doc::LineEnding;
use vize_l0::Allocator;

#[test]
fn complete_outer_suffix_and_closing_quote_lookahead_choose_exact_binding_width_boundaries() {
    let source = "<template><p :id='a+b'/></template><!--tail-->";
    for (width, expected) in [
        (49, "<template><p :id='a + b' /></template><!--tail-->"),
        (48, "<template><p\n  :id='a + b'\n/></template><!--tail-->"),
        (13, "<template><p\n  :id='a + b'\n/></template><!--tail-->"),
        (
            12,
            "<template><p\n  :id='a +\n    b'\n/></template><!--tail-->",
        ),
        (
            0,
            "<template><p\n  :id='a +\n    b'\n/></template><!--tail-->",
        ),
    ] {
        let arena = Allocator::default();
        let owner = assert_output(&arena, source, expected, options(width, 2, LineEnding::Lf));
        assert_eq!(owner.binding_operands().len(), 1);
        assert_eq!(owner.binding_operands()[0].raw_value(), "a+b");
        assert!(owner.attribute_operands().is_empty());
        assert!(owner.operands().is_empty());
    }
}

#[test]
fn nested_binding_and_interpolation_continuations_keep_captured_indent_and_generated_endings() {
    let source = "<template><section><p :id='a+b'>{{1n}}</p></section></template>";
    let flat = "<template><section><p :id='a + b'>{{ 1n }}</p></section></template>";
    for indent in [0, 2, 4] {
        for (ending, nl) in [(LineEnding::Lf, "\n"), (LineEnding::CrLf, "\r\n")] {
            let attribute = " ".repeat(2 * indent);
            let continuation = " ".repeat(3 * indent);
            let opening_close = " ".repeat(indent);
            let interpolation_close = " ".repeat(2 * indent);
            let broken = format!(
                "<template><section><p{nl}{attribute}:id='a +{nl}{continuation}b'{nl}{opening_close}>{{{{{nl}{continuation}1n{nl}{interpolation_close}}}}}</p></section></template>"
            );
            for (width, expected) in [(200, flat), (0, broken.as_str())] {
                let arena = Allocator::default();
                let owner = assert_output(&arena, source, expected, options(width, indent, ending));
                assert_eq!(owner.binding_operands().len(), 1);
                assert_eq!(owner.operands().len(), 1);
            }
        }
    }
}

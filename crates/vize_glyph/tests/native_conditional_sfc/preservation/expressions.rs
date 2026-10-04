use super::super::{assert_output, options};
use vize_glyph::native_doc::LineEnding;
use vize_l0::Allocator;

#[test]
fn supported_original_expression_families_keep_typed_structure_literals_and_fixed_points() {
    // Expected expressions are authored from the existing immutable Doc rules.
    // Each result is still the complete SFC, with one original attribute owner
    // and one independently retained interpolation owner in source order.
    for (raw, expression) in [
        ("true", "true"),
        ("!(a&&null)", "! (a && null)"),
        ("obj[a]", "obj [ a ]"),
        ("f(1n)", "f ( 1n )"),
        ("{a:1n,b:'x'}", "{ a: 1n, b: 'x' }"),
        ("[a,,b,]", "[ a, , b, ]"),
        ("a?b:c", "a ? b : c"),
        ("a,b", "a, b"),
        ("((a+0xCA_FE))*1_000", "((a + 0xCA_FE)) * 1_000"),
    ] {
        let source = format!("<template><p v-if=\"{raw}\">{{{{1n}}}}</p></template>");
        let expected = format!("<template><p v-if=\"{expression}\">{{{{ 1n }}}}</p></template>");
        let arena = Allocator::default();
        let owner = assert_output(&arena, &source, &expected, options(200, 2, LineEnding::Lf));
        assert_eq!(owner.attribute_operands().len(), 1);
        assert_eq!(owner.operands().len(), 1);
        assert_eq!(owner.attribute_operands()[0].raw_value(), raw);
        assert_eq!(owner.attribute_operands()[0].syntax().source().text(), raw);
        assert_eq!(owner.operands()[0].raw_content(), "1n");
    }
}

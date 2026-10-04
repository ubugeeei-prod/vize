//! Existing source-unit/numeric limits remain actual whole original refusals.

use super::refusals::{refused, span};
use super::*;
use oxc_ast::ast::Expression;
use vize_glyph::native_doc::NativeVue2SfcRefusal as Refusal;
use vize_l1::dialect::vue2::surface::TextRefusal;
use vize_l1::embed::syntax::EmbedHole;

#[test]
fn original_wrapped_source_units_thirty_one_and_thirty_two_keep_actual_admission() {
    let accepted = std::iter::repeat_n("1", 15)
        .collect::<std::vec::Vec<_>>()
        .join(",");
    let source = format!("<template><div>{{{{{accepted}}}}}</div></template>");
    let flat = std::iter::repeat_n("1", 15)
        .collect::<std::vec::Vec<_>>()
        .join(", ");
    fixed(
        &source,
        &format!("<template><div>{{{{{flat}}}}}</div></template>"),
        options(200, 2, LineEnding::Lf),
    );
    let arena = Allocator::default();
    let owner = observe_native_vue2_sfc_in(&arena, &source, NativeVue2SfcOptions::default());
    let syntax = owner.descriptor().component().unwrap().bindings()[0]
        .admitted()
        .unwrap()
        .base();
    let Expression::SequenceExpression(sequence) = syntax.expression().unwrap() else {
        panic!("original Sequence")
    };
    assert_eq!(sequence.expressions.len(), 15);
    assert_eq!(syntax.hole(), None);
    let over = format!("!{accepted}");
    let bad = format!("{{{{{over}}}}}");
    let source = format!("<template><div>{bad}</div></template>");
    let owner = observe_native_vue2_sfc_in(&arena, &source, NativeVue2SfcOptions::default());
    refused(
        &owner,
        Refusal::Text {
            span: span(&source, &bad),
            refusal: TextRefusal::NativeHole(EmbedHole::TokenBudget),
        },
    );
    assert_eq!(
        owner.descriptor().component().unwrap().bindings()[0]
            .chain()
            .unwrap()
            .base()
            .hole(),
        Some(EmbedHole::TokenBudget)
    );
}

#[test]
fn original_numeric_run_four_thousand_ninety_six_and_ninety_seven_are_not_relaxed() {
    let raw = format!("{}n", "9".repeat(4095));
    let source = format!("<template><div>{{{{{raw}}}}}</div></template>");
    fixed(&source, &source, options(0, 2, LineEnding::CrLf));
    let arena = Allocator::default();
    let owner = observe_native_vue2_sfc_in(&arena, &source, NativeVue2SfcOptions::default());
    let syntax = owner.descriptor().component().unwrap().bindings()[0]
        .admitted()
        .unwrap()
        .base();
    let Expression::BigIntLiteral(literal) = syntax.expression().unwrap() else {
        panic!("original scalar")
    };
    assert_eq!(literal.raw.unwrap().as_str(), raw);
    assert_eq!(literal.raw.unwrap().as_str().len(), 4096);
    let over = format!("{}n", "9".repeat(4096));
    let bad = format!("{{{{{over}}}}}");
    let source = format!("<template><div>{bad}</div></template>");
    let owner = observe_native_vue2_sfc_in(&arena, &source, NativeVue2SfcOptions::default());
    refused(
        &owner,
        Refusal::Text {
            span: span(&source, &bad),
            refusal: TextRefusal::NativeHole(EmbedHole::SafetyAdmission),
        },
    );
    assert_eq!(
        owner.descriptor().component().unwrap().bindings()[0]
            .chain()
            .unwrap()
            .base()
            .hole(),
        Some(EmbedHole::SafetyAdmission)
    );
}

#[test]
fn original_child_depth_one_hundred_twenty_eight_and_twenty_nine_keep_whole_bounds() {
    for depth in [128, 129] {
        let body = format!("{}plain{}", "<div>".repeat(depth), "</div>".repeat(depth));
        let source = format!("<template>{body}</template>");
        let arena = Allocator::default();
        let owner = observe_native_vue2_sfc_in(&arena, &source, NativeVue2SfcOptions::default());
        assert!(
            owner
                .descriptor()
                .component()
                .unwrap()
                .bindings()
                .is_empty()
        );
        if depth == 128 {
            fixed(&source, &source, options(0, 2, LineEnding::Lf));
        } else {
            refused(
                &owner,
                Refusal::Template(vize_glyph::native_doc::TemplateRefusal::Unsupported {
                    offset: 5 * 129,
                    syntax: vize_glyph::native_doc::UnsupportedSyntax::NestingLimit,
                }),
            );
        }
    }
}

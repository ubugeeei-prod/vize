use oxc_ast::ast::CommentKind;
use vize_glyph::native_doc::{
    LineEnding, NativeSfcDirectivePolicy, NativeSfcObservation, NativeSfcOptions,
    observe_native_sfc_in,
};
use vize_l0::Allocator;
use vize_l1::SurfaceChild;
use vize_l1::embed::{Grammar, Lang, Shape, syntax::RetainedExpression};
use vize_l1::markup::NativeChildren;

use super::fingerprint;

pub(super) fn options(width: usize, indent: usize, ending: LineEnding) -> NativeSfcOptions {
    let mut options = NativeSfcOptions::default();
    options.directives = NativeSfcDirectivePolicy::FormatConditionals;
    options.print.width = width;
    options.print.indent_width = indent;
    options.print.line_ending = ending;
    options
}

pub(super) fn assert_original(
    owner: &NativeSfcObservation<'_>,
    source: &str,
    options: NativeSfcOptions,
) {
    assert!(core::ptr::eq(owner.source(), source));
    assert!(core::ptr::eq(owner.descriptor().source(), source));
    assert_eq!(owner.options(), options);
    assert_eq!(owner.descriptor().options(), options.descriptor);
    assert!(owner.descriptor().issues().is_empty());
    assert!(owner.descriptor().container().errors.is_empty());
    let selected = owner.selected().unwrap();
    assert!(core::ptr::eq(
        selected.component().block().root_source(),
        source
    ));
    assert_eq!(
        vize_l1::check_fidelity(&selected.component().carrier().tree),
        Ok(())
    );
}

fn assert_syntax(original: &RetainedExpression<'_>, source: &str) {
    assert!(core::ptr::eq(original.source().authored_root(), source));
    assert_eq!(
        original.grammar(),
        Grammar {
            shape: Shape::Expr,
            lang: Lang::Js
        }
    );
    assert!(original.source_type().is_module());
    assert!(!original.source_type().is_typescript());
    assert_eq!(original.parser_prefix(), 2);
    assert_eq!(original.hole(), None);
    assert_eq!(original.diagnostics().count(), 0);
}

// This is an assertion walk over the already-owned original events. It never
// observes/parses another embed and never constructs a replacement admission.
fn assert_event_roots(owner: &NativeSfcObservation<'_>) {
    fn walk<'a>(
        owner: &NativeSfcObservation<'a>,
        children: NativeChildren<'_, 'a>,
        interpolation: &mut usize,
        attribute: &mut usize,
    ) {
        let selected = owner.selected().unwrap();
        for child in children {
            match child.surface() {
                SurfaceChild::Element(_) => {
                    let element = child.into_element().unwrap();
                    for current in element.attributes() {
                        if !matches!(current.surface().name.text, "v-if" | "v-else-if") {
                            continue;
                        }
                        let original = &owner.attribute_operands()[*attribute];
                        let root = original.syntax().expression().unwrap();
                        let admitted = original.admitted_for(selected, current.reborrow()).unwrap();
                        assert!(core::ptr::eq(admitted.selected(), selected));
                        assert!(core::ptr::eq(admitted.operand(), original));
                        assert!(core::ptr::eq(
                            admitted.attribute().surface(),
                            current.surface()
                        ));
                        assert!(core::ptr::eq(
                            admitted.attribute().element(),
                            element.surface()
                        ));
                        assert_eq!(admitted.attribute().ordinal(), current.ordinal());
                        assert!(core::ptr::eq(
                            admitted.expression().unwrap().expression(),
                            root
                        ));
                        assert_eq!(
                            original.value_span().slice(owner.source()),
                            original.raw_value()
                        );
                        assert_syntax(original.syntax(), owner.source());
                        *attribute += 1;
                    }
                    walk(owner, element.children(), interpolation, attribute);
                }
                SurfaceChild::Interpolation(_) => {
                    let original = &owner.operands()[*interpolation];
                    let root = original.syntax().expression().unwrap();
                    let admitted = original.admitted_for(selected, child).unwrap();
                    assert!(core::ptr::eq(
                        admitted.expression().unwrap().expression(),
                        root
                    ));
                    assert_eq!(
                        original.content_span().slice(owner.source()),
                        original.raw_content()
                    );
                    assert_syntax(original.syntax(), owner.source());
                    *interpolation += 1;
                }
                _ => {}
            }
        }
    }
    let (mut interpolation, mut attribute) = (0, 0);
    walk(
        owner,
        owner.selected().unwrap().children(),
        &mut interpolation,
        &mut attribute,
    );
    assert_eq!(interpolation, owner.operands().len());
    assert_eq!(attribute, owner.attribute_operands().len());
}

fn comments(original: &RetainedExpression<'_>) -> Vec<(CommentKind, String, String)> {
    original
        .comments()
        .map(|comment| {
            (
                comment.kind(),
                comment.text().unwrap().to_owned(),
                comment
                    .authored_span()
                    .unwrap()
                    .slice(original.source().authored_root())
                    .to_owned(),
            )
        })
        .collect()
}

fn assert_preserved(before: &RetainedExpression<'_>, after: &RetainedExpression<'_>) {
    assert_eq!(before.grammar(), after.grammar());
    assert_eq!(before.source_type(), after.source_type());
    assert_eq!(
        fingerprint::fingerprint(before, before.expression().unwrap()),
        fingerprint::fingerprint(after, after.expression().unwrap())
    );
    assert_eq!(comments(before), comments(after));
    assert!(!core::ptr::eq(
        before.expression().unwrap(),
        after.expression().unwrap()
    ));
}

pub(super) fn assert_output<'a>(
    arena: &'a Allocator,
    source: &'a str,
    expected: &str,
    options: NativeSfcOptions,
) -> NativeSfcObservation<'a> {
    let original = observe_native_sfc_in(arena, source, options);
    assert_original(&original, source, options);
    assert_eq!(original.refusal(), None);
    assert!(original.document().is_ok());
    assert!(original.interpolation_failure().is_none());
    assert!(original.attribute_failure().is_none());
    assert_event_roots(&original);
    let first = original.format().unwrap();
    assert_eq!(first.code, expected, "{source} / {options:?}");
    assert_eq!(first.changed, expected != source);
    let repeated = original.format().unwrap();
    assert_eq!(repeated.code, expected);
    assert_eq!(repeated.changed, first.changed);
    assert_event_roots(&original);

    // Replay the independently authored whole expected source through the same
    // genuine pipeline, then compare typed structure and exact authored atoms.
    let second_arena = Allocator::default();
    let replay = observe_native_sfc_in(&second_arena, expected, options);
    assert_original(&replay, expected, options);
    assert_eq!(replay.refusal(), None);
    assert!(replay.attribute_failure().is_none());
    assert!(replay.interpolation_failure().is_none());
    assert_event_roots(&replay);
    let fixed = replay.format().unwrap();
    assert_eq!(fixed.code, expected);
    assert!(!fixed.changed);
    assert_eq!(original.operands().len(), replay.operands().len());
    assert_eq!(
        original.attribute_operands().len(),
        replay.attribute_operands().len()
    );
    for (before, after) in original.operands().iter().zip(replay.operands()) {
        assert_preserved(before.syntax(), after.syntax());
    }
    for (before, after) in original
        .attribute_operands()
        .iter()
        .zip(replay.attribute_operands())
    {
        assert_eq!(before.kind(), after.kind());
        assert_eq!(
            before.name_span().slice(source),
            after.name_span().slice(expected)
        );
        assert_preserved(before.syntax(), after.syntax());
    }
    let before = original.selected().unwrap().component().block().span();
    let after = replay.selected().unwrap().component().block().span();
    assert_eq!(
        &source[..before.start as usize],
        &expected[..after.start as usize]
    );
    assert_eq!(
        &source[before.end as usize..],
        &expected[after.end as usize..]
    );
    original
}

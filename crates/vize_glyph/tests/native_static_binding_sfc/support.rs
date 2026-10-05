use super::{assert_original, fingerprint, prior};
use oxc_ast::ast::CommentKind;
use vize_glyph::native_doc::{
    LineEnding, NativeSfcDirectivePolicy, NativeSfcObservation, NativeSfcOptions,
    observe_native_sfc_in,
};
use vize_l0::Allocator;
use vize_l1::embed::{Grammar, Lang, Shape, syntax::RetainedExpression};
use vize_l1::markup::{NativeChildren, NativeTemplateComponent};
use vize_l1::{SurfaceChild, container::Vue};

pub(super) fn options(width: usize, indent: usize, ending: LineEnding) -> NativeSfcOptions {
    let mut options = prior::options(width, indent, ending);
    options.directives = NativeSfcDirectivePolicy::FormatConditionalsAndStaticBindings;
    options
}

pub(super) fn selected<'a>(
    arena: &'a Allocator,
    source: &'a str,
    options: NativeSfcOptions,
) -> NativeTemplateComponent<'a> {
    let descriptor = Vue.observe_descriptor(arena, source, options.descriptor);
    assert_eq!(descriptor.issues(), []);
    assert_eq!(descriptor.container().errors.as_slice(), []);
    NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap()
}

// Assert against already owned original events; this walk never observes an
// embed, constructs an admission, or supplies raw metadata to a provider.
pub(super) fn assert_bindings(owner: &NativeSfcObservation<'_>) {
    fn walk<'a>(
        owner: &NativeSfcObservation<'a>,
        children: NativeChildren<'_, 'a>,
        index: &mut usize,
    ) {
        let selected = owner.selected().unwrap();
        for child in children {
            if let SurfaceChild::Element(_) = child.surface() {
                let element = child.into_element().unwrap();
                for current in element.attributes() {
                    let Some(original) = owner.binding_operands().get(*index) else {
                        continue;
                    };
                    let Some(view) = original.admitted_for(selected, current.reborrow()) else {
                        continue;
                    };
                    assert!(core::ptr::eq(view.selected(), selected));
                    assert!(core::ptr::eq(view.operand(), original));
                    assert!(core::ptr::eq(view.attribute().surface(), current.surface()));
                    assert!(core::ptr::eq(view.attribute().element(), element.surface()));
                    assert_eq!(view.attribute().ordinal(), current.ordinal());
                    assert!(core::ptr::eq(
                        view.expression().unwrap().expression(),
                        original.syntax().expression().unwrap()
                    ));
                    assert_eq!(
                        original.name_span().slice(owner.source()),
                        current.surface().name.text
                    );
                    assert_eq!(
                        original.value_span().slice(owner.source()),
                        original.raw_value()
                    );
                    assert_eq!(
                        original.raw_value(),
                        current.surface().value.as_ref().unwrap().content.text
                    );
                    let syntax = original.syntax();
                    assert!(core::ptr::eq(
                        syntax.source().authored_root(),
                        owner.source()
                    ));
                    assert_eq!(
                        syntax.grammar(),
                        Grammar {
                            shape: Shape::Expr,
                            lang: Lang::Js
                        }
                    );
                    assert!(syntax.source_type().is_module());
                    assert!(!syntax.source_type().is_typescript());
                    assert_eq!(syntax.parser_prefix(), 2);
                    assert_eq!(syntax.hole(), None);
                    assert_eq!(syntax.diagnostics().count(), 0);
                    *index += 1;
                }
                walk(owner, element.children(), index);
            }
        }
    }
    let mut index = 0;
    walk(owner, owner.selected().unwrap().children(), &mut index);
    assert_eq!(index, owner.binding_operands().len());
    assert!(owner.binding_failure().is_none());
}

fn comments(syntax: &RetainedExpression<'_>) -> Vec<(CommentKind, String, String)> {
    syntax
        .comments()
        .map(|comment| {
            (
                comment.kind(),
                comment.text().unwrap().to_owned(),
                comment
                    .authored_span()
                    .unwrap()
                    .slice(syntax.source().authored_root())
                    .to_owned(),
            )
        })
        .collect()
}

pub(super) fn assert_output<'a>(
    arena: &'a Allocator,
    source: &'a str,
    expected: &str,
    options: NativeSfcOptions,
) -> NativeSfcObservation<'a> {
    // The unchanged helper covers complete output/options/changed, repeated
    // printing, independent fixed point and both earlier observation families.
    let owner = prior::assert_output(arena, source, expected, options);
    assert_bindings(&owner);
    let expected_arena = Allocator::default();
    let replay = observe_native_sfc_in(&expected_arena, expected, options);
    assert_original(&replay, expected, options);
    assert_bindings(&replay);
    assert_eq!(replay.refusal(), None);
    let fixed = replay.format().unwrap();
    assert_eq!(fixed.code, expected);
    assert!(!fixed.changed);
    assert_eq!(
        owner.binding_operands().len(),
        replay.binding_operands().len()
    );
    for (before, after) in owner
        .binding_operands()
        .iter()
        .zip(replay.binding_operands())
    {
        assert_eq!(
            before.name_span().slice(source),
            after.name_span().slice(expected)
        );
        assert_eq!(
            before.argument_span().slice(source),
            after.argument_span().slice(expected)
        );
        let (before, after) = (before.syntax(), after.syntax());
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
    assert_eq!(owner.format().unwrap().code, expected);
    assert_bindings(&owner);
    owner
}

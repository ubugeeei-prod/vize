//! Full source/descriptor/native observations/options/outcome stay in one genuine owner.

use vize_glyph::native_doc::{
    LineEnding, NativeSfcObservation, NativeSfcOptions, observe_native_sfc_in,
};
use vize_l0::Allocator;

#[path = "native_scriptless_sfc/budgets.rs"]
mod budgets;
#[path = "native_scriptless_sfc/custody.rs"]
mod custody;
#[path = "native_scriptless_sfc/directive_values.rs"]
mod directive_values;
#[path = "native_scriptless_sfc/fingerprint.rs"]
mod fingerprint;
#[path = "native_scriptless_sfc/layout.rs"]
mod layout;
#[path = "native_scriptless_sfc/lifecycle.rs"]
mod lifecycle;
#[path = "native_scriptless_sfc/preservation.rs"]
mod preservation;
#[path = "native_scriptless_sfc/refusals.rs"]
mod refusals;

fn options(width: usize, indent: usize, ending: LineEnding) -> NativeSfcOptions {
    let mut options = NativeSfcOptions::default();
    options.print.width = width;
    options.print.indent_width = indent;
    options.print.line_ending = ending;
    options
}

fn newline(ending: LineEnding) -> &'static str {
    match ending {
        LineEnding::Lf => "\n",
        LineEnding::CrLf => "\r\n",
    }
}

fn assert_original(owner: &NativeSfcObservation<'_>, source: &str, options: NativeSfcOptions) {
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

fn assert_fixed(source: &str, options: NativeSfcOptions) {
    let arena = Allocator::default();
    let original = observe_native_sfc_in(&arena, source, options);
    assert_original(&original, source, options);
    let first = original.format().unwrap();
    assert_eq!(first.changed, first.code != source);
    let second_arena = Allocator::default();
    let second = observe_native_sfc_in(&second_arena, &first.code, options);
    assert_original(&second, &first.code, options);
    let replay = second.format().unwrap();
    assert_eq!(replay.code, first.code, "{source} / {options:?}");
    assert!(!replay.changed);
    assert_eq!(original.operands().len(), second.operands().len());
    for (before, after) in original.operands().iter().zip(second.operands()) {
        let before = before.syntax();
        let after = after.syntax();
        assert_eq!(before.grammar(), after.grammar());
        assert_eq!(before.source_type(), after.source_type());
        assert_eq!(before.hole(), None);
        assert_eq!(after.hole(), None);
        assert_eq!(
            fingerprint::fingerprint(before, before.expression().unwrap()),
            fingerprint::fingerprint(after, after.expression().unwrap())
        );
        assert_eq!(
            preservation::comments(before),
            preservation::comments(after)
        );
    }
    let blocks = |owner: &NativeSfcObservation<'_>| {
        owner
            .descriptor()
            .container()
            .blocks
            .iter()
            .map(|block| {
                (
                    block.name.to_owned(),
                    block.open_tag.slice(owner.source()).to_owned(),
                    block.close_tag.unwrap().slice(owner.source()).to_owned(),
                    block
                        .attrs
                        .iter()
                        .map(|attr| {
                            (
                                attr.name.to_owned(),
                                attr.value.map(str::to_owned),
                                attr.span.slice(owner.source()).to_owned(),
                            )
                        })
                        .collect::<std::vec::Vec<_>>(),
                )
            })
            .collect::<std::vec::Vec<_>>()
    };
    assert_eq!(blocks(&original), blocks(&second));
    let before = original.selected().unwrap().component().block().span();
    let after = second.selected().unwrap().component().block().span();
    assert_eq!(
        &source[..before.start as usize],
        &first.code[..after.start as usize]
    );
    assert_eq!(
        &source[before.end as usize..],
        &first.code[after.end as usize..]
    );
}

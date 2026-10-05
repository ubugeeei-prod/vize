//! Full original Vue2 SFC documents, not a modern carrier or default route.

use vize_glyph::native_doc::{
    LineEnding, NativeVue2SfcObservation, NativeVue2SfcOptions, observe_native_vue2_sfc_in,
};
use vize_l0::Allocator;

#[path = "native_vue2_scriptless_sfc/capture.rs"]
mod capture;
#[path = "native_vue2_scriptless_sfc/custody.rs"]
mod custody;
#[path = "native_vue2_scriptless_sfc/layout.rs"]
mod layout;
#[path = "native_vue2_scriptless_sfc/lifecycle.rs"]
mod lifecycle;
#[path = "native_vue2_scriptless_sfc/limits.rs"]
mod limits;
#[path = "native_vue2_scriptless_sfc/recovery.rs"]
mod recovery;
#[path = "native_vue2_scriptless_sfc/refusals.rs"]
mod refusals;

fn options(width: usize, indent: usize, ending: LineEnding) -> NativeVue2SfcOptions {
    let mut options = NativeVue2SfcOptions::default();
    options.print.width = width;
    options.print.indent_width = indent;
    options.print.line_ending = ending;
    options
}
fn original(owner: &NativeVue2SfcObservation<'_>, source: &str, expected: NativeVue2SfcOptions) {
    assert!(core::ptr::eq(owner.source(), source));
    assert!(core::ptr::eq(owner.descriptor().source(), source));
    assert_eq!(owner.options(), expected);
    assert_eq!(owner.descriptor().options(), expected.descriptor);
    assert!(owner.descriptor().issues().is_empty());
    assert!(owner.descriptor().container().errors.is_empty());
    let selected = owner.selected().unwrap();
    assert!(core::ptr::eq(selected.observation(), owner.descriptor()));
    assert!(core::ptr::eq(
        selected.component(),
        owner.descriptor().component().unwrap()
    ));
    assert!(core::ptr::eq(selected.block().root_source(), source));
    assert_eq!(
        selected.block().span().slice(source),
        selected.block().source()
    );
    assert_eq!(vize_l1::check_fidelity(selected.component().tree()), Ok(()));
}
fn fixed(source: &str, expected: &str, options: NativeVue2SfcOptions) {
    let arena = Allocator::default();
    let owner = observe_native_vue2_sfc_in(&arena, source, options);
    original(&owner, source, options);
    let before = custody::facts(&owner);
    let first = owner.format().unwrap();
    assert_eq!(first.code, expected, "{source} / {options:?}");
    assert_eq!(first.changed, source != expected);
    assert_eq!(owner.format().unwrap().code, first.code);
    assert_eq!(custody::facts(&owner), before);
    let second_arena = Allocator::default();
    let second = observe_native_vue2_sfc_in(&second_arena, &first.code, options);
    original(&second, &first.code, options);
    let after = second.format().unwrap();
    assert_eq!(after.code, first.code, "fixed point {source} / {options:?}");
    assert!(!after.changed);
    let before_block = owner.selected().unwrap().block().span();
    let after_block = second.selected().unwrap().block().span();
    assert_eq!(
        &source[..before_block.start as usize],
        &first.code[..after_block.start as usize]
    );
    assert_eq!(
        &source[before_block.end as usize..],
        &first.code[after_block.end as usize..]
    );
    assert_eq!(
        owner.descriptor().component().unwrap().bindings().len(),
        second.descriptor().component().unwrap().bindings().len()
    );
}

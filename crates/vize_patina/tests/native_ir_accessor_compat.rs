//! Downstream access to the deprecated getters and their original borrow contracts.

use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::container::vue::{DescriptorOptions, L1Ir};
use vize_l1_to_l2::native_file::lower_sfc_native;
use vize_l2::file::L2Ir;
use vize_patina::native::{
    canonical::NativeCanonicalLint,
    sfc::{NativeSfcLintOwner, NativeSfcTemplateOwner},
};

#[test]
#[allow(deprecated)]
fn old_owner_getters_remain_public_with_the_original_borrow_signatures() {
    fn setup<'o, 'a>(owner: &'o NativeSfcLintOwner<'a>) -> (&'o L1Ir<'a>, &'o L2Ir<'a>) {
        (owner.descriptor(), owner.file())
    }
    fn template<'o, 'a>(owner: &'o NativeSfcTemplateOwner<'a>) -> &'o L1Ir<'a> {
        owner.descriptor()
    }

    let _: for<'o, 'a> fn(&'o NativeSfcLintOwner<'a>) -> (&'o L1Ir<'a>, &'o L2Ir<'a>) = setup;
    let _: for<'o, 'a> fn(&'o NativeSfcTemplateOwner<'a>) -> &'o L1Ir<'a> = template;
}

#[test]
#[allow(deprecated)]
fn old_canonical_getter_retains_the_owner_borrow_after_the_lint_wrapper_is_dropped() {
    let arena = Allocator::new();
    let source = "<template><img alt='original'/></template>";
    let owner = lower_sfc_native(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: Default::default(),
        },
    );
    let old = {
        let moved = Box::new(NativeCanonicalLint::from_vue(owner.admitted().unwrap()));
        moved.file()
    };
    let canonical = {
        let lint = NativeCanonicalLint::from_vue(owner.admitted().unwrap());
        lint.l2ir()
    };

    assert!(core::ptr::eq(old, canonical));
    assert!(core::ptr::eq(old, owner.file().unwrap().l2ir()));
}

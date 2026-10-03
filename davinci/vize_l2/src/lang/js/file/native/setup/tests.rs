extern crate std;

use super::*;
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    markup::NativeTemplateComponent,
};

std::thread_local! {
    static FAULT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
pub(super) fn after_park() {
    FAULT.with(|fault| {
        if fault.replace(false) {
            panic!("injected original setup park interruption");
        }
    });
}
#[test]
fn caught_setup_park_unwind_keeps_stock_owner_and_cannot_parse_or_complete_again() {
    let arena = Allocator::default();
    let source = "<template></template><script setup>/*whole*/ const value=1</script>";
    let descriptor = Vue.observe_descriptor(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let selected = NativeTemplateComponent::parse_in(&arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap();
    let mut owner = NativeTemplateOwner::new(selected).unwrap_or_else(|_| panic!("native owner"));
    FAULT.with(|fault| fault.set(true));
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| owner.parse_setup_program()))
            .is_err()
    );
    let body = owner
        .retained_setup()
        .unwrap()
        .program()
        .unwrap()
        .body
        .as_ptr();
    assert_eq!(owner.retained_setup().unwrap().comments().count(), 1);
    assert_eq!(
        owner.parse_setup_program().unwrap_err().kind,
        Kind::Interrupted
    );
    assert_eq!(
        owner
            .retained_setup()
            .unwrap()
            .program()
            .unwrap()
            .body
            .as_ptr(),
        body
    );
    assert!(owner.begin().is_err());
    let moved = owner.finish();
    assert_eq!(
        moved
            .retained_setup()
            .unwrap()
            .program()
            .unwrap()
            .body
            .as_ptr(),
        body
    );
    assert!(moved.view().is_err());
    assert!(moved.setup().is_err());
    assert!(moved.file().unwrap().units().is_empty());
}

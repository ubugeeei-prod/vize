//! Real Descriptor, original Program and native cursor construction only.

use crate::lang::js::{NativeTemplateFile, NativeTemplateOwner};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    container::{Vue, vue::DescriptorOptions},
    markup::{NativeTemplateComponent, SurfaceParseOptions},
};

pub(super) fn owner<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateOwner<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let selected = NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap();
    let mut owner = NativeTemplateOwner::new(selected).unwrap_or_else(|_| panic!("actual owner"));
    if owner.selected().setup().is_some() {
        owner.parse_setup_program().unwrap();
    }
    owner
}

pub(super) fn completed<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateFile<'a> {
    let mut owner = owner(arena, source);
    {
        let mut walk = owner.begin_setup().unwrap();
        for child in walk.selected().children() {
            walk.child(child).unwrap();
        }
        walk.complete().unwrap();
    }
    owner.finish()
}

use crate::container::Vue;
use crate::container::vue::DescriptorOptions;
use crate::markup::NativeTemplateComponent;
use vize_l0::Allocator;
use vize_l0::config::{VueDialect, VueVersion};

fn selected<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateComponent<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: crate::SurfaceParseOptions::default(),
        },
    );
    NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap()
}

mod observations;
mod ownership;
mod refusals;

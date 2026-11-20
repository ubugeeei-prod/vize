use crate::container::Vue;
use crate::container::vue::DescriptorOptions;
use crate::markup::SelectedTemplateComponent;
use vize_l0::Allocator;
use vize_l0::config::{VueDialect, VueVersion};

fn selected<'a>(arena: &'a Allocator, source: &'a str) -> SelectedTemplateComponent<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: crate::markup::SurfaceParseOptions::default(),
        },
    );
    SelectedTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap()
}

mod observations;
mod ownership;
mod refusals;

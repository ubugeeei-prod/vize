use crate::container::Vue;
use crate::container::vue::DescriptorOptions;
use crate::markup::NativeTemplateComponent;
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};

mod ownership;
mod refusals;

fn selected<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateComponent<'a>, &'static str> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: crate::SurfaceParseOptions::default(),
        },
    );
    NativeTemplateComponent::parse_in(
        arena,
        descriptor
            .admitted()
            .map_err(|_| "fixture descriptor refused")?,
    )
    .map_err(|_| "fixture component parse failed")?
    .ok_or("fixture has no selected template")
}

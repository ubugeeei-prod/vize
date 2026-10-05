use super::*;
use crate::container::{Vue, vue::DescriptorOptions};
use crate::embed::{DecodeSegmentKind, SourceError};
use crate::markup::{NativeRootTextError, NativeTemplateGrammar};
use crate::{SurfaceChild, SurfaceParseOptions};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};

mod ownership;
mod refusal;
mod source;

fn selected<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateComponent<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap()
}

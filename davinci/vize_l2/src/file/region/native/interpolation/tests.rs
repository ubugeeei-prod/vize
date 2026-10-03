use super::interruption::{Fault, set_fault};
use crate::file::NativeFileInterpolationState as State;
use crate::lang::js::{
    NativeInterpolationInput, NativeTemplateIssueKind as Kind, NativeTemplateOwner,
};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceChild, SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    markup::NativeTemplateComponent,
};

mod interruption;
mod ownership;
mod refusal;

type Test = Result<(), &'static str>;
fn check(value: bool) -> Test {
    if value {
        Ok(())
    } else {
        Err("required original-owner condition")
    }
}
fn same<T: PartialEq>(actual: T, expected: T) -> Test {
    check(actual == expected)
}
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
            template: SurfaceParseOptions::default(),
        },
    );
    NativeTemplateComponent::parse_in(arena, descriptor.admitted().map_err(|_| "descriptor")?)
        .map_err(|_| "selected parse")?
        .ok_or("template")
}
fn owner<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateOwner<'a>, &'static str> {
    NativeTemplateOwner::new(selected(arena, source)?).map_err(|_| "original File start")
}

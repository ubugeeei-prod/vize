use super::{NativeInterpolationInput, NativeInterpolationInputError};
use vize_l0::{
    Allocator, String,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    markup::NativeTemplateComponent,
};

mod ownership;
mod refusals;

type Test = Result<(), String>;

fn check(value: bool, message: &str) -> Test {
    if value {
        Ok(())
    } else {
        Err(String::from(message))
    }
}

fn same<T: core::fmt::Debug + PartialEq>(actual: T, expected: T, message: &str) -> Test {
    if actual == expected {
        Ok(())
    } else {
        Err(alloc::format!("{message}: expected {expected:?}, got {actual:?}").into())
    }
}

fn selected<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateComponent<'a>, String> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    NativeTemplateComponent::parse_in(
        arena,
        descriptor.admitted().map_err(|_| "descriptor refused")?,
    )
    .map_err(|_| "component refused")?
    .ok_or_else(|| String::from("selected template missing"))
}

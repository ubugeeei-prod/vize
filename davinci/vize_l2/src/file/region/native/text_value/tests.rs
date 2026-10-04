use super::interruption::{Fault, observations, reset_observations, set_fault};
use crate::file::{NativeFileTextValueState as State, NativeTextValuePolicyError as Policy};
use crate::lang::js::{NativeTemplateFile, NativeTemplateIssueKind as Kind, NativeTemplateOwner};
use crate::op::{Op, TextOp};
use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
    id::NodeId,
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    markup::NativeTemplateComponent,
};
mod interruption;
mod ownership;
mod refusal;
mod source;
type Test = Result<(), &'static str>;
fn check(value: bool) -> Test {
    if value {
        Ok(())
    } else {
        Err("original prepared root Text condition")
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
    NativeTemplateComponent::parse_in(arena, descriptor.admitted().map_err(|_| "Descriptor")?)
        .map_err(|_| "original selected parse")?
        .ok_or("template")
}
fn owner<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateOwner<'a>, &'static str> {
    NativeTemplateOwner::new(selected(arena, source)?).map_err(|_| "original File owner")
}
fn completed<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateFile<'a>, &'static str> {
    let mut original = owner(arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        for child in walk.selected().children() {
            walk.root_text_value(child)
                .map_err(|_| "actual prepared root event")?;
        }
        walk.complete().map_err(|_| "complete")?;
    }
    Ok(original.finish())
}
fn text<'f, 'a>(file: &'f crate::file::FileArtifact<'a>) -> Result<&'f TextOp<'a>, &'static str> {
    let [Op::Text(text)] = file.artifact().root().ops.as_slice() else {
        return Err("one actual Text allocation");
    };
    Ok(text.as_ref())
}

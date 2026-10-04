//! Independent original-source, canonical allocation and custody laws.

use crate::file::{FileArtifact, NativeFileAttributeValueState as State};
use crate::lang::js::{NativeTemplateFile, NativeTemplateIssueKind as Kind, NativeTemplateOwner};
use crate::op::{ElementOp, Op};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
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
        Err("required original value condition")
    }
}
fn same<T: PartialEq>(actual: T, expected: T) -> Test {
    check(actual == expected)
}
fn selected<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateComponent<'a>, &'static str> {
    let observation = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    NativeTemplateComponent::parse_in(arena, observation.admitted().map_err(|_| "descriptor")?)
        .map_err(|_| "selected parse")?
        .ok_or("template")
}
fn owner<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateOwner<'a>, &'static str> {
    NativeTemplateOwner::new(selected(arena, source)?).map_err(|_| "owning File")
}
fn lower<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateFile<'a>, &'static str> {
    let mut original = owner(arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        for child in selected.children() {
            walk.child(child).map_err(|_| "original child")?;
        }
        walk.complete().map_err(|_| "complete")?;
    }
    Ok(core::hint::black_box(original.finish()))
}
fn element<'f, 'a>(
    file: &'f FileArtifact<'a>,
    index: usize,
) -> Result<&'f ElementOp<'a>, &'static str> {
    match file.artifact().root().ops.get(index) {
        Some(Op::Element(element)) => Ok(element),
        _ => Err("actual Element"),
    }
}

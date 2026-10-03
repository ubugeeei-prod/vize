use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{
        Vue,
        vue::{DescriptorObservation, DescriptorOptions},
    },
    markup::NativeTemplateComponent,
};
use vize_l2::lang::js::{NativeTemplateFile, NativeTemplateOwner};
mod accepted;
mod refusal;

fn descriptor<'a>(arena: &'a Allocator, source: &'a str) -> DescriptorObservation<'a> {
    Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    )
}
fn owner<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateOwner<'a>, &'static str> {
    let descriptor = descriptor(arena, source);
    let selected =
        NativeTemplateComponent::parse_in(arena, descriptor.admitted().map_err(|_| "descriptor")?)
            .map_err(|_| "selected parse")?
            .ok_or("selected template")?;
    NativeTemplateOwner::new(selected).map_err(|_| "owner")
}
fn complete<'a>(owner: NativeTemplateOwner<'a>) -> Result<NativeTemplateFile<'a>, &'static str> {
    let mut owner = core::hint::black_box(owner);
    {
        let mut walk = owner.begin().map_err(|_| "native begin")?;
        for child in walk.selected().children() {
            walk.child(child).map_err(|_| "original child")?;
        }
        walk.complete().map_err(|_| "normal original end")?;
    }
    Ok(core::hint::black_box(owner.finish()))
}
#[track_caller]
fn check(condition: bool) -> Result<(), &'static str> {
    if condition {
        Ok(())
    } else {
        Err("required selected setup condition")
    }
}

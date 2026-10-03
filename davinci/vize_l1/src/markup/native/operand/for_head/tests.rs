use super::super::super::{NativeAttribute, NativeTemplateComponent, NativeTemplateGrammar};
use super::{NativeAttributeForHead, NativeAttributeOperandError};
use crate::container::Vue;
use crate::container::vue::DescriptorOptions;
use crate::embed::Lang;
use crate::embed::syntax::{EmbedHole, ForHeadHole, NativeForInput, NativeForRefusal};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};

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
    let admitted = descriptor
        .admitted()
        .map_err(|_| "fixture descriptor refused")?;
    NativeTemplateComponent::parse_in(arena, admitted)
        .map_err(|_| "fixture component parse failed")?
        .ok_or("fixture has no selected template")
}

fn first_attribute<'s, 'a>(owner: &'s NativeTemplateComponent<'a>) -> NativeAttribute<'s, 'a> {
    owner
        .children()
        .next()
        .unwrap()
        .into_element()
        .unwrap()
        .attributes()
        .next()
        .unwrap()
}

mod ownership;
mod refusals;

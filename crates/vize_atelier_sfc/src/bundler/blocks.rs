use std::borrow::Cow;

use vize_carton::String;

use crate::{SfcCustomBlock, SfcParseOptions, SfcStyleBlock, parse_sfc};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SfcBlockAttribute {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundlerStyleBlock {
    pub content: String,
    pub src: Option<String>,
    pub lang: Option<String>,
    pub scoped: bool,
    pub module: bool,
    pub module_name: Option<String>,
    pub index: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundlerCustomBlock {
    pub block_type: String,
    pub content: String,
    pub src: Option<String>,
    pub attrs: Vec<SfcBlockAttribute>,
    pub index: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SfcSrcInfo {
    pub script_src: Option<String>,
    pub template_src: Option<String>,
}

pub fn extract_style_blocks(source: &str, filename: Option<&str>) -> Vec<BundlerStyleBlock> {
    parse_descriptor(source, filename).map_or_else(Vec::new, |descriptor| {
        descriptor
            .styles
            .iter()
            .enumerate()
            .map(style_block_to_bundler)
            .collect()
    })
}

pub fn extract_custom_blocks(source: &str, filename: Option<&str>) -> Vec<BundlerCustomBlock> {
    parse_descriptor(source, filename).map_or_else(Vec::new, |descriptor| {
        descriptor
            .custom_blocks
            .iter()
            .enumerate()
            .map(custom_block_to_bundler)
            .collect()
    })
}

pub fn extract_src_info(source: &str, filename: Option<&str>) -> SfcSrcInfo {
    let Some(descriptor) = parse_descriptor(source, filename) else {
        return SfcSrcInfo {
            script_src: None,
            template_src: None,
        };
    };

    let script_src = descriptor
        .script
        .as_ref()
        .or(descriptor.script_setup.as_ref())
        .and_then(|script| script.src.as_deref())
        .map(String::from);
    let template_src = descriptor
        .template
        .as_ref()
        .and_then(|template| template.src.as_deref())
        .map(String::from);

    SfcSrcInfo {
        script_src,
        template_src,
    }
}

pub fn has_scoped_style(source: &str, filename: Option<&str>) -> bool {
    parse_descriptor(source, filename)
        .is_some_and(|descriptor| descriptor.styles.iter().any(|style| style.scoped))
}

pub(super) fn parse_descriptor<'a>(
    source: &'a str,
    filename: Option<&str>,
) -> Option<crate::SfcDescriptor<'a>> {
    parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.unwrap_or("anonymous.vue").into(),
            ..Default::default()
        },
    )
    .ok()
}

fn style_block_to_bundler((index, style): (usize, &SfcStyleBlock<'_>)) -> BundlerStyleBlock {
    let module_attr = style.attrs.get("module");
    let module_name = module_attr.and_then(|value| {
        let value = value.as_ref();
        if value.is_empty() {
            None
        } else {
            Some(String::from(value))
        }
    });

    BundlerStyleBlock {
        content: String::from(style.content.as_ref()),
        src: style.src.as_deref().map(String::from),
        lang: style.lang.as_deref().map(String::from),
        scoped: style.scoped,
        module: module_attr.is_some(),
        module_name,
        index: index as u32,
    }
}

fn custom_block_to_bundler((index, block): (usize, &SfcCustomBlock<'_>)) -> BundlerCustomBlock {
    let mut attrs = block_attrs(&block.attrs);
    attrs.sort_by(|left, right| left.name.as_str().cmp(right.name.as_str()));
    BundlerCustomBlock {
        block_type: String::from(block.block_type.as_ref()),
        content: String::from(block.content.as_ref()),
        src: block
            .attrs
            .get("src")
            .map(|value| String::from(value.as_ref())),
        attrs,
        index: index as u32,
    }
}

fn block_attrs(
    attrs: &vize_carton::FxHashMap<Cow<'_, str>, Cow<'_, str>>,
) -> Vec<SfcBlockAttribute> {
    attrs
        .iter()
        .map(|(name, value)| SfcBlockAttribute {
            name: String::from(name.as_ref()),
            value: if value.is_empty() {
                None
            } else {
                Some(String::from(value.as_ref()))
            },
        })
        .collect()
}

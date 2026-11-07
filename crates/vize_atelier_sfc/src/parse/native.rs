//! Project the complete L1 compatibility container into the public descriptor.

use std::borrow::Cow;
use vize_carton::FxHashMap;
use vize_l1::container::Vue;
use vize_l1::container::vue::compat::{
    CompatBlock, CompatBlockLocation, CompatSplitError, split_blocks, tag_name_eq,
};

use crate::types::{
    BlockLocation, PadOption, SfcCustomBlock, SfcDescriptor, SfcError, SfcParseOptions,
    SfcScriptBlock, SfcStyleBlock, SfcTemplateBlock,
};

const TAG_TEMPLATE: &[u8] = b"template";
const TAG_SCRIPT: &[u8] = b"script";
const TAG_STYLE: &[u8] = b"style";

struct NativeTemplateProjection<'a> {
    loc: BlockLocation,
    lang: Option<&'a str>,
}

fn location(value: CompatBlockLocation) -> BlockLocation {
    BlockLocation {
        start: value.start,
        end: value.end,
        tag_start: value.tag_start,
        tag_end: value.tag_end,
        start_line: value.start_line,
        start_column: value.start_column,
        end_line: value.end_line,
        end_column: value.end_column,
    }
}

fn split_error(error: CompatSplitError) -> SfcError {
    SfcError {
        message: error.message,
        code: Some(error.code.into()),
        loc: Some(location(error.location)),
    }
}

fn template_only_projection<'a>(
    source: &'a str,
    options: &SfcParseOptions,
) -> Option<NativeTemplateProjection<'a>> {
    if options.pad != PadOption::None
        || options.ignore_empty
        || options.template_parse_options.is_some()
    {
        return None;
    }
    let observed = Vue.observe_template_only(source)?;
    let span = observed.block().span();
    let start = span.start as usize;
    let end = span.end as usize;
    let content = source.get(start..end)?;
    let prefix = source.get(..start)?;
    let start_line = memchr::memchr_iter(b'\n', prefix.as_bytes()).count() + 1;
    let start_column = prefix.rfind('\n').map_or(start + 1, |at| start - at);
    let end_line = start_line + memchr::memchr_iter(b'\n', content.as_bytes()).count();
    let end_column = content
        .rfind('\n')
        .map_or(start_column + content.len(), |at| content.len() - at);
    Some(NativeTemplateProjection {
        loc: BlockLocation {
            start,
            end,
            tag_start: observed.open_tag().start as usize,
            tag_end: observed.close_tag().end as usize,
            start_line,
            start_column,
            end_line,
            end_column,
        },
        lang: observed.lang(),
    })
}

pub(super) fn parse_sfc<'a>(
    source: &'a str,
    options: SfcParseOptions,
) -> Result<SfcDescriptor<'a>, SfcError> {
    if let Some(projection) = template_only_projection(source, &options)
        && let Some(content) = source.get(projection.loc.start..projection.loc.end)
    {
        let lang = projection.lang.map(Cow::Borrowed);
        let mut attrs = FxHashMap::default();
        if let Some(value) = projection.lang {
            attrs.insert(Cow::Borrowed("lang"), Cow::Borrowed(value));
        }
        return Ok(SfcDescriptor {
            filename: Cow::Owned(options.filename.into()),
            source: Cow::Borrowed(source),
            template: Some(SfcTemplateBlock {
                content: Cow::Borrowed(content),
                loc: projection.loc,
                lang,
                src: None,
                attrs,
            }),
            ..Default::default()
        });
    }
    parse_complete_sfc(source, options)
}

fn parse_complete_sfc<'a>(
    source: &'a str,
    options: SfcParseOptions,
) -> Result<SfcDescriptor<'a>, SfcError> {
    let mut descriptor = SfcDescriptor {
        filename: Cow::Owned(options.filename.into()),
        source: Cow::Borrowed(source),
        ..Default::default()
    };
    let mut policy_error = None;
    split_blocks(source, |block: CompatBlock<'a>| {
        let CompatBlock {
            tag_name,
            attrs,
            content,
            location: block_location,
        } = block;
        let loc = location(block_location);
        if tag_name_eq(tag_name, TAG_TEMPLATE) {
            if descriptor.template.is_some() {
                policy_error = Some(SfcError {
                    message: "SFC can only contain one <template> block".into(),
                    code: Some("DUPLICATE_TEMPLATE".into()),
                    loc: Some(loc),
                });
                return false;
            }
            descriptor.template = Some(SfcTemplateBlock {
                content,
                loc,
                lang: attrs.get("lang").cloned(),
                src: attrs.get("src").cloned(),
                attrs,
            });
        } else if tag_name_eq(tag_name, TAG_SCRIPT) {
            let is_setup = attrs.contains_key("setup");
            let script = SfcScriptBlock {
                content,
                loc,
                lang: attrs.get("lang").cloned(),
                src: attrs.get("src").cloned(),
                setup: is_setup,
                attrs,
                bindings: None,
            };
            if is_setup {
                if descriptor.script_setup.is_some() {
                    policy_error = Some(SfcError {
                        message: "SFC can only contain one <script setup> block".into(),
                        code: Some("DUPLICATE_SCRIPT_SETUP".into()),
                        loc: Some(script.loc),
                    });
                    return false;
                }
                descriptor.script_setup = Some(script);
            } else {
                if descriptor.script.is_some() {
                    policy_error = Some(SfcError {
                        message: "SFC can only contain one <script> block".into(),
                        code: Some("DUPLICATE_SCRIPT".into()),
                        loc: Some(script.loc),
                    });
                    return false;
                }
                descriptor.script = Some(script);
            }
        } else if tag_name_eq(tag_name, TAG_STYLE) {
            let scoped = attrs.contains_key("scoped");
            let module = attrs.contains_key("module").then(|| {
                attrs
                    .get("module")
                    .filter(|value| !value.is_empty())
                    .cloned()
                    .unwrap_or(Cow::Borrowed("$style"))
            });
            for css_var in vize_croquis::sfc::__internal::extract_css_vars(&content) {
                if !descriptor
                    .css_vars
                    .iter()
                    .any(|existing| existing.as_ref() == css_var.as_str())
                {
                    descriptor.css_vars.push(Cow::Owned(css_var.into()));
                }
            }
            descriptor.styles.push(SfcStyleBlock {
                content,
                loc,
                lang: attrs.get("lang").cloned(),
                src: attrs.get("src").cloned(),
                scoped,
                module,
                attrs,
            });
        } else if let Ok(tag) = core::str::from_utf8(tag_name)
            && !vize_carton::is_void_tag(tag)
        {
            descriptor.custom_blocks.push(SfcCustomBlock {
                block_type: Cow::Borrowed(tag),
                content,
                loc,
                attrs,
            });
        }
        true
    })
    .map_err(split_error)?;
    if let Some(error) = policy_error {
        return Err(error);
    }
    Ok(descriptor)
}

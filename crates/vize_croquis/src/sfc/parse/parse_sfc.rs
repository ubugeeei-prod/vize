use crate::sfc::types::{
    BlockLocation, SfcCustomBlock, SfcDescriptor, SfcError, SfcParseOptions, SfcScriptBlock,
    SfcStyleBlock, SfcTemplateBlock,
};
use std::borrow::Cow;
use vize_l1::container::vue::compat::{
    CompatBlock, CompatBlockLocation, CompatSplitError, split_blocks, tag_name_eq,
};

// Tag name bytes for fast comparison
const TAG_TEMPLATE: &[u8] = b"template";
const TAG_SCRIPT: &[u8] = b"script";
const TAG_STYLE: &[u8] = b"style";

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

/// Parse a Vue SFC into a descriptor with zero-copy strings.
///
/// The L1 container walk supplies every block boundary, attribute and source
/// location. This adapter only applies the shipped descriptor policy while
/// preserving borrowed slices in the returned descriptor.
pub fn parse_sfc<'a>(
    source: &'a str,
    options: SfcParseOptions,
) -> Result<SfcDescriptor<'a>, SfcError> {
    split_sfc(source, options, true)
}

/// [`parse_sfc`] without the per-style `v-bind()` scan: the descriptor is
/// identical except that `css_vars` stays empty.
///
/// For block-level readers that never compile styles (Musea's Art files).
pub fn parse_sfc_without_css_vars<'a>(
    source: &'a str,
    options: SfcParseOptions,
) -> Result<SfcDescriptor<'a>, SfcError> {
    split_sfc(source, options, false)
}

fn split_sfc<'a>(
    source: &'a str,
    options: SfcParseOptions,
    collect_css_vars: bool,
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
            let script_block = SfcScriptBlock {
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
                        loc: Some(script_block.loc),
                    });
                    return false;
                }
                descriptor.script_setup = Some(script_block);
            } else {
                if descriptor.script.is_some() {
                    policy_error = Some(SfcError {
                        message: "SFC can only contain one <script> block".into(),
                        code: Some("DUPLICATE_SCRIPT".into()),
                        loc: Some(script_block.loc),
                    });
                    return false;
                }
                descriptor.script = Some(script_block);
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
            if collect_css_vars {
                for css_var in crate::sfc::extract_css_vars(&content) {
                    if !descriptor
                        .css_vars
                        .iter()
                        .any(|existing| existing.as_ref() == css_var.as_str())
                    {
                        descriptor.css_vars.push(Cow::Owned(css_var.into()));
                    }
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

//! Check existing original frame spans, never split or parse the source again.

use super::NativeSfcLintRefusal as Refusal;
use vize_l0::{SourceRoot, Span};
use vize_l1::{
    container::{Block, vue::DescriptorObservation},
    embed::Lang,
};

pub(super) fn check(descriptor: &DescriptorObservation<'_>) -> Result<(), Refusal> {
    let admitted = descriptor.admitted().map_err(|error| Refusal::Descriptor {
        issues: error.issues().to_vec(),
        errors: error.errors().to_vec(),
    })?;
    let root = admitted.root();
    let blocks = &descriptor.container().blocks;
    if blocks.len() != 2 || admitted.ordinary().is_some() || admitted.styles().len() != 0 {
        return Err(Refusal::UnsupportedEnvelope {
            span: root.whole_block().span(),
        });
    }
    let setup = admitted.setup().ok_or(Refusal::UnsupportedEnvelope {
        span: root.whole_block().span(),
    })?;
    let template = admitted.template().ok_or(Refusal::UnsupportedEnvelope {
        span: root.whole_block().span(),
    })?;
    if setup.lang() != Lang::Ts || setup.container_index() == template.container_index() {
        return Err(Refusal::UnsupportedEnvelope {
            span: setup.block().span(),
        });
    }
    let mut end = 0;
    for (index, block) in blocks.iter().enumerate() {
        if index != setup.container_index() && index != template.container_index() {
            return Err(Refusal::UnsupportedEnvelope {
                span: block.open_tag,
            });
        }
        whitespace(root, Span::new(end, block.open_tag.start))?;
        check_block(root, block)?;
        end = block.close_tag.ok_or(Refusal::SourceMismatch)?.end;
    }
    whitespace(root, Span::new(end, root.whole_block().span().end))
}

fn whitespace(root: SourceRoot<'_>, span: Span) -> Result<(), Refusal> {
    let text = root
        .source()
        .get(span.start as usize..span.end as usize)
        .ok_or(Refusal::SourceMismatch)?;
    if text.bytes().all(|byte| byte.is_ascii_whitespace()) {
        Ok(())
    } else {
        Err(Refusal::UnsupportedEnvelope { span })
    }
}

fn check_block(root: SourceRoot<'_>, block: &Block<'_>) -> Result<(), Refusal> {
    let reject = || Refusal::UnsupportedEnvelope {
        span: block.open_tag,
    };
    let source = root.source();
    let name = root
        .whole_block()
        .span_of(block.name)
        .ok_or(Refusal::SourceMismatch)?;
    if name.start
        != block
            .open_tag
            .start
            .checked_add(1)
            .ok_or(Refusal::SourceMismatch)?
        || block.open_tag.end != block.content.start
        || source.as_bytes().get(block.open_tag.start as usize) != Some(&b'<')
        || source.as_bytes().get(
            block
                .open_tag
                .end
                .checked_sub(1)
                .ok_or(Refusal::SourceMismatch)? as usize,
        ) != Some(&b'>')
    {
        return Err(reject());
    }
    // Attribute names, values, separators and roles were classified by the
    // genuine Descriptor Policy at each original block emission. Do not repeat
    // that parser or infer a second classification from authored spellings.
    // The distinct SFC host additionally authenticates the narrower original
    // product frame grammar through these retained spans, without a header parse.
    attributes(root, block, name.end)?;
    let close = block.close_tag.ok_or(Refusal::SourceMismatch)?;
    if block.content.end != close.start {
        return Err(reject());
    }
    let raw = source
        .get(close.start as usize..close.end as usize)
        .ok_or(Refusal::SourceMismatch)?;
    let closing_name_end = 2 + block.name.len();
    if !raw.starts_with("</")
        || !raw.ends_with('>')
        || !raw
            .get(2..closing_name_end)
            .is_some_and(|name| name.eq_ignore_ascii_case(block.name))
    {
        return Err(Refusal::UnsupportedEnvelope { span: close });
    }
    let separator = Span::new(close.start + closing_name_end as u32, close.end - 1);
    frame_whitespace(root, separator)?;
    Ok(())
}

fn frame_whitespace(root: SourceRoot<'_>, span: Span) -> Result<(), Refusal> {
    let gap = root
        .source()
        .get(span.start as usize..span.end as usize)
        .ok_or(Refusal::SourceMismatch)?;
    if gap
        .bytes()
        .all(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
    {
        Ok(())
    } else {
        Err(Refusal::UnsupportedEnvelope { span })
    }
}

/// Metadata classification stays with the original Descriptor Policy. These
/// source-bound name/value/delimiter joins check only the separate SFC frame
/// profile: four separator bytes and SP/TAB around '='. Values remain opaque.
fn attributes(root: SourceRoot<'_>, block: &Block<'_>, mut end: u32) -> Result<(), Refusal> {
    for attribute in &block.attrs {
        let name = root
            .whole_block()
            .span_of(attribute.name)
            .ok_or(Refusal::SourceMismatch)?;
        frame_whitespace(root, Span::new(end, name.start))?;
        if name.start != attribute.span.start || name.end > attribute.span.end {
            return Err(Refusal::SourceMismatch);
        }
        if let Some(value) = attribute.value {
            let value = root
                .whole_block()
                .span_of(value)
                .ok_or(Refusal::SourceMismatch)?;
            let prefix_span = Span::new(name.end, value.start);
            let prefix = root
                .source()
                .get(prefix_span.start as usize..prefix_span.end as usize)
                .ok_or(Refusal::SourceMismatch)?;
            let (prefix, quote) = if let Some(prefix) = prefix.strip_suffix('"') {
                (prefix, Some('"'))
            } else if let Some(prefix) = prefix.strip_suffix('\'') {
                (prefix, Some('\''))
            } else {
                (prefix, None)
            };
            if prefix.trim_matches([' ', '\t']) != "=" {
                return Err(Refusal::UnsupportedEnvelope { span: prefix_span });
            }
            let suffix_span = Span::new(value.end, attribute.span.end);
            let suffix = root
                .source()
                .get(suffix_span.start as usize..suffix_span.end as usize)
                .ok_or(Refusal::SourceMismatch)?;
            if !match quote {
                Some('"') => suffix == "\"",
                Some('\'') => suffix == "'",
                None => suffix.is_empty(),
                _ => false,
            } {
                return Err(Refusal::UnsupportedEnvelope { span: suffix_span });
            }
            // Original unquoted values stop at LF but retain an immediate CR.
            // Native metadata trims that CR, so this profile cannot certify it.
            if quote.is_none()
                && root.source().as_bytes().get(attribute.span.end as usize) == Some(&b'\r')
            {
                return Err(Refusal::UnsupportedEnvelope {
                    span: Span::new(attribute.span.end, attribute.span.end + 1),
                });
            }
            end = attribute.span.end;
        } else {
            frame_whitespace(root, Span::new(name.end, attribute.span.end))?;
            end = name.end;
        }
    }
    frame_whitespace(root, Span::new(end, block.open_tag.end - 1))
}

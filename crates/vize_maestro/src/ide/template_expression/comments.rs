//! Comments are not navigable symbols, before any checker/fallback request.
use crate::ide::{IdeContext, expression_regions};
use crate::virtual_code::{ArtCursorPosition, BlockType};

pub(crate) fn is_in_template_comment(ctx: &IdeContext<'_>) -> bool {
    if !matches!(
        &ctx.block_type,
        Some(BlockType::Template | BlockType::Art(ArtCursorPosition::VariantTemplate(_)))
    ) {
        return false;
    }
    comment_at_offset(&ctx.content, ctx.offset)
}

pub(super) fn comment_at_offset(content: &str, offset: usize) -> bool {
    let Some(before) = content.get(..offset) else {
        return false;
    };
    if crate::ide::completion::is_inside_html_comment(content, offset) {
        return false;
    }
    let start = if super::is_in_mustache_expression(content, offset) {
        before.rfind("{{").map(|start| start + 2)
    } else {
        super::directive_expression_start(content, offset)
    };
    let Some(start) = start else { return false };
    let Some(rest) = content.get(start..) else {
        return false;
    };
    let end = match content.as_bytes().get(start.saturating_sub(1)) {
        Some(b'"') => rest.find('"'),
        Some(b'\'') => rest.find('\''),
        Some(b'[') => rest.find(']'),
        _ => rest.find("}}"),
    }
    .unwrap_or(rest.len());
    expression_regions::html_comment_at(
        rest.get(..end).unwrap_or_default(),
        offset.saturating_sub(start),
    )
}

#[cfg(test)]
mod tests {
    use super::comment_at_offset;
    #[test]
    fn original_comment_words_are_not_navigable_and_adjacent_code_is() {
        for value in [
            "// close 😀\nclose()",
            "/* close 日本語 */ close()",
            "`raw // close ${close /* comment */}`",
        ] {
            let source = vize_l0::cstr!("<button @click=\"{value}\" />");
            let comment = if value.starts_with('`') {
                "comment"
            } else {
                "close"
            };
            assert!(comment_at_offset(
                &source,
                source.find(comment).unwrap() + 2
            ));
            assert!(!comment_at_offset(
                &source,
                source.rfind("close").unwrap() + 2
            ));
        }
        for source in [
            "<p>{{ count /* count 😀 */ }}</p>",
            "<p>{{ /* count */ count }}</p>",
        ] {
            let comment = source.find("/*").unwrap() + 4;
            assert!(comment_at_offset(source, comment));
        }
        for source in [
            "<p title=\"/* close */\" />",
            "<p>{{ '/* close */' }}</p>",
            "<p>{{ `// close` }}</p>",
            "<p>{{ /[/*]/.test(close) }}</p>",
            "<p :title=\"&quot;/* close */&quot; + count\" />",
            "<p :title=\"&#34;// close&#34; + count\" />",
        ] {
            assert!(!comment_at_offset(
                source,
                source.find("close").unwrap() + 2
            ));
        }
    }
}

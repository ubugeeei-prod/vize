//! Diagnostic types for vize_patina linter.
//!
//! Uses `CompactString` for efficient small string storage.
//! Split into:
//! - [`types`]: Core diagnostic data structures
//! - [`formatting`]: Markdown rendering and help text formatting

mod compact_help;
pub mod formatting;
mod types;

pub use formatting::{HelpRenderTarget, render_help};
pub use types::{Fix, HelpLevel, LintDiagnostic, LintSummary, Severity, TextEdit};

#[cfg(test)]
mod tests {
    use super::{HelpLevel, HelpRenderTarget, compact_help, formatting, render_help};
    use vize_s0::ToCompactString;

    #[test]
    fn test_help_level_full() {
        let level = HelpLevel::Full;
        let help = "**Why:** Use `:key` for tracking.\n\n```vue\n<li :key=\"id\">\n```";
        let result = level.process(help);
        // Full mode preserves raw markdown
        assert_eq!(result, Some(help.to_compact_string()));
    }

    #[test]
    fn test_help_level_none() {
        let level = HelpLevel::None;
        let result = level.process("Any help text");
        assert_eq!(result, None);
    }

    #[test]
    fn test_help_level_short_strips_markdown() {
        let level = HelpLevel::Short;
        let help = "**Why:** The `:key` attribute helps Vue track items.\n\n**Fix:**\n```vue\n<li :key=\"id\">\n```";
        let result = level.process(help);
        assert_eq!(
            result,
            Some("Why: The :key attribute helps Vue track items.".to_compact_string())
        );
    }

    #[test]
    fn test_help_level_short_skips_code_blocks() {
        let level = HelpLevel::Short;
        let help = "```vue\n<li :key=\"id\">\n```\nUse unique keys";
        let result = level.process(help);
        assert_eq!(result, Some("Use unique keys".to_compact_string()));
    }

    #[test]
    fn test_help_level_short_simple_text() {
        let level = HelpLevel::Short;
        let help = "Add a key attribute to the element";
        let result = level.process(help);
        assert_eq!(
            result,
            Some("Add a key attribute to the element".to_compact_string())
        );
    }

    #[test]
    fn test_compact_help_text_with_backticks() {
        let result = compact_help::compact_help_text("Use `v-model` instead of `{{ }}`");
        assert_eq!(result, "Use v-model instead of {{ }}");
    }

    #[test]
    fn test_compact_help_text_removes_examples_in_all_locales() {
        assert_eq!(
            compact_help::compact_help_text(
                "Use a method (e.g. @click=\"handler\") or a function (e.g. @click=\"() => run()\").",
            ),
            "Use a method or a function."
        );
        assert_eq!(
            compact_help::compact_help_text(
                "メソッド参照（例: @click=\"handler\"）または関数（例: @click=\"() => run()\"）を使用してください。",
            ),
            "メソッド参照または関数を使用してください。"
        );
        assert_eq!(
            compact_help::compact_help_text(
                "请使用方法引用（例如 @click=\"handler\"）或函数（例如 @click=\"() => run()\"）。",
            ),
            "请使用方法引用或函数。"
        );
    }

    #[test]
    fn test_compact_help_text_keeps_only_the_action_sentence() {
        assert_eq!(
            compact_help::compact_help_text(
                "Apply the safe rewrite. Reason: the longer explanation belongs to full help.",
            ),
            "Apply the safe rewrite."
        );
        assert_eq!(
            compact_help::compact_help_text(
                "安全な修正を適用してください。理由：詳細はfullヘルプにあります。"
            ),
            "安全な修正を適用してください。"
        );
        assert_eq!(
            compact_help::compact_help_text("Call storeToRefs(store) before destructuring."),
            "Call storeToRefs(store) before destructuring."
        );
    }

    #[test]
    fn test_render_markdown_bold() {
        let result = formatting::render_markdown_to_ansi("**bold** text");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_render_markdown_inline_code() {
        let result = formatting::render_markdown_to_ansi("Use `v-model` directive");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_render_markdown_header() {
        let result = formatting::render_markdown_to_ansi("# Why");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_render_markdown_code_block() {
        let result = formatting::render_markdown_to_ansi("```vue\n<li :key=\"id\">\n```");
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_render_markdown_plain_text() {
        let result = formatting::render_markdown_to_ansi("plain text");
        assert_eq!(result, "plain text");
    }

    #[test]
    fn test_render_markdown_underscore_bold() {
        let result = formatting::render_markdown_to_ansi("__bold__ text");
        insta::assert_snapshot!(result.as_str());
    }

    // render_help tests

    #[test]
    fn test_render_help_ansi() {
        let md = "**bold** and `code`";
        let result = render_help(md, HelpRenderTarget::Ansi);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_render_help_plain_text() {
        let md = "**Why:** Use `:key` for tracking.\n\n```vue\n<li :key=\"id\">\n```";
        let result = render_help(md, HelpRenderTarget::PlainText);
        assert_eq!(result, "Why: Use :key for tracking.\n\n  <li :key=\"id\">");
    }

    #[test]
    fn test_render_help_markdown_passthrough() {
        let md = "**bold** and `code`";
        let result = render_help(md, HelpRenderTarget::Markdown);
        assert_eq!(result, md);
    }

    // strip_markdown tests

    #[test]
    fn test_strip_markdown_bold_and_code() {
        let result = formatting::strip_markdown("**bold** and `code`");
        assert_eq!(result, "bold and code");
    }

    #[test]
    fn test_strip_markdown_headers() {
        let result = formatting::strip_markdown("# Title\n## Subtitle\nBody text");
        assert_eq!(result, "Title\nSubtitle\nBody text");
    }

    #[test]
    fn test_strip_markdown_code_block() {
        let result = formatting::strip_markdown("Before\n```vue\n<div>code</div>\n```\nAfter");
        assert_eq!(result, "Before\n  <div>code</div>\nAfter");
    }

    #[test]
    fn test_strip_markdown_plain_text() {
        let result = formatting::strip_markdown("plain text");
        assert_eq!(result, "plain text");
    }
}

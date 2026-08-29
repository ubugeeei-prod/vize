//! a11y/media-has-caption
//!
//! Require `<video>` and `<audio>` elements to have captions.
//!
//! Media elements should have a `<track kind="captions">` child for
//! accessibility. Alternatively, `muted` attribute or `aria-label` can
//! satisfy the requirement for some cases.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <video src="movie.mp4"></video>
//! ```
//!
//! ### Valid
//! ```vue
//! <video src="movie.mp4">
//!   <track kind="captions" src="captions.vtt" />
//! </video>
//! <video src="movie.mp4" muted></video>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupBindingKind, MarkupContext, MarkupElement, MarkupNode, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "a11y/media-has-caption",
    description: "Require media elements to have captions",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Require media elements to have captions
#[derive(Default)]
pub struct MediaHasCaption;

impl MediaHasCaption {
    fn has_exact_static_attribute(element: &MarkupElement<'_>, name: &str) -> bool {
        let mut found = false;
        element.walk_bindings(&mut |binding| {
            if binding.kind() == MarkupBindingKind::Attribute
                && binding.is_unqualified_arg_exact(name)
            {
                found = true;
            }
        });
        found
    }

    fn has_exact_named_prop(element: &MarkupElement<'_>, name: &str) -> bool {
        let mut found = false;
        element.walk_bindings(&mut |binding| {
            if matches!(
                binding.kind(),
                MarkupBindingKind::Attribute | MarkupBindingKind::Bind
            ) && binding.is_static_unqualified_arg_exact(name)
            {
                found = true;
            }
        });
        found
    }

    fn first_exact_static_attribute_value<'a>(
        element: &MarkupElement<'a>,
        name: &str,
    ) -> Option<&'a str> {
        let mut seen = false;
        let mut value = None;
        element.walk_bindings(&mut |binding| {
            if !seen
                && binding.kind() == MarkupBindingKind::Attribute
                && binding.is_unqualified_arg_exact(name)
            {
                seen = true;
                value = binding.static_value();
            }
        });
        value
    }

    fn has_caption_track(element: &MarkupElement<'_>, transparent_fragments: bool) -> bool {
        let mut found = false;
        element.walk_children(&mut |child| {
            if let MarkupNode::Element(child_element) = child
                && child_element.is_unqualified_tag_exact("track")
                && let Some(kind) = Self::first_exact_static_attribute_value(&child_element, "kind")
                && (kind == "captions" || kind == "descriptions")
            {
                found = true;
            }
            // JSX lowering splices fragments in child position. Preserve that
            // boundary for direct OXC IR without changing Vue `<template>`.
            if let MarkupNode::Element(child_element) = child
                && transparent_fragments
                && child_element.tag().is_empty()
                && Self::has_caption_track(&child_element, transparent_fragments)
            {
                found = true;
            }
        });
        found
    }

    fn check_element(
        ctx: &mut LintContext<'_>,
        element: &MarkupElement<'_>,
        transparent_fragments: bool,
    ) {
        if element.is_component() {
            return;
        }

        if !element.is_unqualified_tag_exact("video") && !element.is_unqualified_tag_exact("audio")
        {
            return;
        }

        if Self::has_exact_static_attribute(element, "muted") {
            return;
        }

        if Self::has_exact_named_prop(element, "aria-label") {
            return;
        }

        if Self::has_exact_named_prop(element, "aria-labelledby") {
            return;
        }

        if Self::has_caption_track(element, transparent_fragments) {
            return;
        }

        let tag = element.tag();
        let message = ctx.t_fmt("a11y/media-has-caption.message", &[("tag", tag)]);
        let help = ctx.t("a11y/media-has-caption.help");
        ctx.warn_at_with_help(message, element.range(), help);
    }
}

impl MarkupRule for MediaHasCaption {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        let transparent_fragments = ctx.is_jsx();
        Self::check_element(ctx.lint(), element, transparent_fragments);
    }
}

impl Rule for MediaHasCaption {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Self::check_element(ctx, &MarkupElement::new(element), false);
    }
}

#[cfg(test)]
mod tests {
    use super::MediaHasCaption;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(MediaHasCaption));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_video_with_track() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<video src="movie.mp4"><track kind="captions" src="captions.vtt" /></video>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_video_muted() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<video src="movie.mp4" muted></video>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_video_with_aria_label() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<video src="movie.mp4" aria-label="Movie clip"></video>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_video_with_bound_aria_label() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<video src="movie.mp4" :aria-label="label"></video>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_audio_with_bound_aria_labelledby() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<audio src="podcast.mp3" :aria-labelledby="labelId"></audio>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_video_no_captions() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<video src="movie.mp4"></video>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_audio_no_captions() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<audio src="podcast.mp3"></audio>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_component_skipped() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<VideoPlayer src="movie.mp4"></VideoPlayer>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}

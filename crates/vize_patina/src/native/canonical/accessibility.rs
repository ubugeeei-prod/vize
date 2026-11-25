//! Two independent accessibility results over a single canonical node walk.
use super::{NativeCanonicalLint, NativeCanonicalLintRefusal, NativeLintFinding};
use collect::{Candidate, Candidates};
mod collect;
use crate::{
    attribute_policy::ImageAltMessages,
    native::{IMG_ALT_RULE, TABINDEX_NO_POSITIVE_RULE, tabindex_no_positive},
};
use vize_l0::{
    SmallVec, Span,
    diag::{Advisory, Diagnostic, DiagnosticPart, MessageLookup, PartKind, Stage},
};

type Spans = SmallVec<[Span; 8]>;

/// Findings in authored byte order, or the original refusal for each rule.
#[derive(Debug)]
pub struct AccessibilityFindings {
    /// The complete response for `a11y/tabindex-no-positive`.
    pub tabindex_no_positive: Result<Vec<NativeLintFinding>, NativeCanonicalLintRefusal>,
    /// The complete response for `a11y/img-alt`.
    pub img_alt: Result<Vec<NativeLintFinding>, NativeCanonicalLintRefusal>,
}

impl NativeCanonicalLint<'_, '_> {
    /// Run the tabindex and image-alt rules in one traversal of this exact File.
    /// Each rule retains its own first refusal. Catalog lookups have the same
    /// keys, count and order as requesting tabindex first and image-alt second.
    #[must_use]
    pub fn accessibility(&self, messages: &(impl MessageLookup + ?Sized)) -> AccessibilityFindings {
        vize_l0::profile!(
            "patina.native.canonical.accessibility",
            self.check_accessibility(messages)
        )
    }

    fn check_accessibility(
        &self,
        messages: &(impl MessageLookup + ?Sized),
    ) -> AccessibilityFindings {
        if self.l2ir.artifact().root().ops.is_empty() {
            return AccessibilityFindings {
                tabindex_no_positive: Ok(Vec::new()),
                img_alt: Ok(Vec::new()),
            };
        }
        let mut candidates = Candidates::default();
        candidates.collect(self.l2ir);
        let tabindex_no_positive = candidates.tabindex.render(
            TABINDEX_NO_POSITIVE_RULE,
            || tabindex_no_positive::Messages::new(messages),
            tabindex_no_positive::Messages::texts,
        );
        let img_alt = candidates.images.render(
            IMG_ALT_RULE,
            || ImageAltMessages::from_lookup(|key| messages.lookup(key)),
            |text| (text.message(), text.help()),
        );
        AccessibilityFindings {
            tabindex_no_positive,
            img_alt,
        }
    }
}

// Each standalone rule requests its catalog at its first actual finding,
// including when a later node refuses the whole query. Render in that same
// rule order after the one shared traversal.
impl Candidate {
    #[inline]
    fn render<T>(
        &mut self,
        rule_name: &'static str,
        messages: impl FnOnce() -> T,
        texts: impl FnOnce(&T) -> (&str, &str),
    ) -> Result<Vec<NativeLintFinding>, NativeCanonicalLintRefusal> {
        if self.spans.is_empty() {
            return self.outcome().map(|()| Vec::new());
        }
        let rendered = messages();
        self.outcome()?;
        let (headline, help) = texts(&rendered);
        Ok(render_findings(&mut self.spans, rule_name, headline, help))
    }
}

fn render_findings(
    spans: &mut Spans,
    rule_name: &'static str,
    headline: &str,
    help: &str,
) -> Vec<NativeLintFinding> {
    let header = |span| NativeLintFinding {
        rule_name,
        diagnostic: Diagnostic::new(Advisory::Warning, Stage::Surface, span, headline),
    };
    let add_help = |finding: &mut NativeLintFinding| {
        finding.diagnostic.parts.push(DiagnosticPart::new(
            PartKind::Help,
            finding.diagnostic.span,
            help,
        ));
    };
    let complete = |span| {
        let mut finding = header(span);
        add_help(&mut finding);
        finding
    };
    if let [only] = spans.as_slice() {
        return vec![complete(*only)];
    }
    spans.sort_unstable_by_key(|span| (span.start, span.end));
    if !spans.spilled() {
        return spans.iter().copied().map(complete).collect();
    }
    // Drop the heap-backed spans before allocating structured help for every
    // diagnostic. The first pass has already built the final localized headers.
    let mut output: Vec<_> = core::mem::take(spans).into_iter().map(header).collect();
    for finding in &mut output {
        add_help(finding);
    }
    output
}

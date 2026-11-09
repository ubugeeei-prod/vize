//! Template comment directives from the resident L1 surface tree.

use vize_l0::directive::{DirectiveKind, parse_level_severity, parse_vize_directive};
use vize_l0::{CompactString, Span, cstr};
use vize_l1::markup::directive::{DirectivePrefix, DirectiveSyntax, VueDirectives};
use vize_l1::markup::{Element, ElementClose, Interpolation, SurfaceChild, SurfaceTree, Token};
use vize_relief::SourceLocation;

use super::LintContext;

struct CommentDirective<'a> {
    content: &'a str,
    span: Span,
    kind: DirectiveKind,
}

struct PendingForget {
    rules: Vec<CompactString>,
    marker: (u32, u32),
}

/// Original template comments and their directive effects.
///
/// The surface tree owns comment admission, so strings, attributes and
/// recovered junk cannot manufacture suppression policy.
pub(crate) struct TemplateCommentDirectives<'t, 'a> {
    surface: &'t SurfaceTree<'a>,
    directives: Vec<CommentDirective<'a>>,
}

impl<'t, 'a> TemplateCommentDirectives<'t, 'a> {
    pub(crate) fn new(surface: &'t SurfaceTree<'a>) -> Self {
        let mut directives = Vec::new();
        collect_directives(surface.source, &surface.children, &mut directives);
        directives.sort_unstable_by_key(|directive| directive.span.start);
        Self {
            surface,
            directives,
        }
    }

    /// Register suppression-only effects before template-level callbacks run.
    pub(crate) fn prescan(&self, ctx: &mut LintContext<'_>) {
        self.prescan_children(ctx, &self.surface.children);
    }

    /// Replay a diagnostic-producing directive at the original comment visit.
    pub(crate) fn process_at(&self, ctx: &mut LintContext<'_>, start: u32) {
        let Some(directive) = self.directive_at(start) else {
            return;
        };
        process_vize_directive(ctx, directive.content, directive.span, directive.kind);
    }

    pub(crate) fn contains(&self, start: u32) -> bool {
        self.directive_at(start).is_some()
    }

    fn directive_at(&self, start: u32) -> Option<&CommentDirective<'a>> {
        let index = self
            .directives
            .partition_point(|directive| directive.span.start < start);
        self.directives
            .get(index)
            .filter(|directive| directive.span.start == start)
    }

    fn prescan_children(&self, ctx: &mut LintContext<'_>, children: &[SurfaceChild<'a>]) {
        vize_l0::recursion::ensure_sufficient_stack(|| {
            let mut forget_next_child = Vec::new();
            for (index, child) in children.iter().enumerate() {
                self.prescan_child(ctx, children, index, child, &mut forget_next_child);
            }
            for pending in forget_next_child {
                ctx.disable_exact_targets(&pending.rules, Vec::new(), pending.marker);
            }
        });
    }

    fn prescan_child(
        &self,
        ctx: &mut LintContext<'_>,
        siblings: &[SurfaceChild<'a>],
        index: usize,
        child: &SurfaceChild<'a>,
        forget_next_child: &mut Vec<PendingForget>,
    ) {
        match child {
            SurfaceChild::Comment(token) => {
                let span = token_span(self.surface.source, token);
                let content = comment_body(token.text);
                ctx.register_lint_comment(content, span.start);
                let line = ctx.offset_to_line(span.start);
                let Some(directive) = parse_vize_directive(content, line, span.start) else {
                    return;
                };
                match directive.kind {
                    DirectiveKind::Expected => ctx.expect_error_next_line(line),
                    DirectiveKind::Level => {
                        if let Some(severity) = parse_level_severity(&directive.payload) {
                            ctx.set_severity_override_next_line(line, severity);
                        }
                    }
                    DirectiveKind::IgnoreStart => ctx.push_ignore_region(line),
                    DirectiveKind::IgnoreEnd => ctx.pop_ignore_region(line),
                    DirectiveKind::Forget => forget_next_child.push(PendingForget {
                        rules: forget_rules(&directive.payload),
                        marker: (span.start, span.end),
                    }),
                    _ => {}
                }
            }
            SurfaceChild::Element(element) => {
                if !forget_next_child.is_empty() {
                    let targets = self.forgotten_element_targets(siblings, index, element);
                    apply_forgotten_targets(ctx, forget_next_child, targets);
                }
                self.prescan_children(ctx, &element.children);
            }
            SurfaceChild::Text(token)
                if !token.text.trim().is_empty() && !forget_next_child.is_empty() =>
            {
                let span = token_span(self.surface.source, token);
                apply_forgotten_targets(ctx, forget_next_child, vec![(span.start, span.end)]);
            }
            SurfaceChild::Interpolation(interpolation) if !forget_next_child.is_empty() => {
                let span = interpolation_span(self.surface.source, interpolation);
                apply_forgotten_targets(ctx, forget_next_child, vec![(span.start, span.end)]);
            }
            SurfaceChild::Cdata(_)
            | SurfaceChild::ProcessingInstruction(_)
            | SurfaceChild::Unexpected(_)
            | SurfaceChild::Text(_)
            | SurfaceChild::Interpolation(_) => {}
        }
    }

    fn forgotten_element_targets(
        &self,
        siblings: &[SurfaceChild<'a>],
        index: usize,
        element: &Element<'a>,
    ) -> Vec<(u32, u32)> {
        let span = element_span(self.surface.source, element);
        let mut targets = vec![(span.start, span.end)];
        if !has_directive(self.surface.source, element, "if") {
            return targets;
        }
        for sibling in siblings.iter().skip(index + 1) {
            let SurfaceChild::Element(branch) = sibling else {
                continue;
            };
            let span = element_span(self.surface.source, branch);
            if has_directive(self.surface.source, branch, "else-if") {
                targets.push((span.start, span.end));
                continue;
            }
            if has_directive(self.surface.source, branch, "else") {
                targets.push((span.start, span.end));
            }
            break;
        }
        targets
    }
}

fn collect_directives<'a>(
    source: &'a str,
    children: &[SurfaceChild<'a>],
    directives: &mut Vec<CommentDirective<'a>>,
) {
    vize_l0::recursion::ensure_sufficient_stack(|| {
        for child in children {
            match child {
                SurfaceChild::Comment(token) => {
                    let span = token_span(source, token);
                    let content = comment_body(token.text);
                    let line = source
                        .get(..span.start as usize)
                        .unwrap_or(source)
                        .bytes()
                        .filter(|byte| *byte == b'\n')
                        .count() as u32
                        + 1;
                    if let Some(directive) = parse_vize_directive(content, line, span.start) {
                        directives.push(CommentDirective {
                            content,
                            span,
                            kind: directive.kind,
                        });
                    }
                }
                SurfaceChild::Element(element) => {
                    collect_directives(source, &element.children, directives);
                }
                _ => {}
            }
        }
    });
}

pub(crate) fn process_vize_directive(
    ctx: &mut LintContext<'_>,
    content: &str,
    span: Span,
    kind: DirectiveKind,
) {
    let line = ctx.offset_to_line(span.start);
    let loc = SourceLocation { span };
    match kind {
        DirectiveKind::Todo => {
            if let Some(directive) = parse_vize_directive(content, line, span.start) {
                let message = if directive.payload.is_empty() {
                    CompactString::from("TODO")
                } else {
                    cstr!("TODO: {}", directive.payload)
                };
                ctx.current_rule = "vize/todo";
                ctx.warn(message, &loc);
            }
        }
        DirectiveKind::Fixme => {
            if let Some(directive) = parse_vize_directive(content, line, span.start) {
                let message = if directive.payload.is_empty() {
                    CompactString::from("FIXME")
                } else {
                    cstr!("FIXME: {}", directive.payload)
                };
                ctx.current_rule = "vize/fixme";
                ctx.error(message, &loc);
            }
        }
        DirectiveKind::Expected => ctx.expect_error_next_line(line),
        DirectiveKind::IgnoreStart => ctx.push_ignore_region(line),
        DirectiveKind::IgnoreEnd => ctx.pop_ignore_region(line),
        DirectiveKind::Level => {
            if let Some(directive) = parse_vize_directive(content, line, span.start)
                && let Some(severity) = parse_level_severity(&directive.payload)
            {
                ctx.set_severity_override_next_line(line, severity);
            }
        }
        DirectiveKind::Deprecated => {
            if let Some(directive) = parse_vize_directive(content, line, span.start) {
                let message = if directive.payload.is_empty() {
                    CompactString::from("Deprecated")
                } else {
                    cstr!("Deprecated: {}", directive.payload)
                };
                ctx.current_rule = "vize/deprecated";
                ctx.warn(message, &loc);
            }
        }
        DirectiveKind::Forget => {
            if let Some(directive) = parse_vize_directive(content, line, span.start)
                && directive.payload.is_empty()
            {
                ctx.current_rule = "vize/forget";
                ctx.warn(CompactString::from("@vize:forget requires a reason"), &loc);
            }
        }
        DirectiveKind::Docs | DirectiveKind::DevOnly | DirectiveKind::Unknown => {}
    }
}

fn apply_forgotten_targets(
    ctx: &mut LintContext<'_>,
    pending: &mut Vec<PendingForget>,
    targets: Vec<(u32, u32)>,
) {
    for directive in pending.drain(..) {
        ctx.disable_exact_targets(&directive.rules, targets.clone(), directive.marker);
    }
}

fn forget_rules(payload: &str) -> Vec<CompactString> {
    payload
        .split(|character: char| character == ',' || character.is_ascii_whitespace())
        .map(|candidate| candidate.trim_matches(['"', '\'', '(', ')', '[', ']', '{', '}', ';']))
        .filter(|candidate| looks_like_rule_id(candidate))
        .map(CompactString::from)
        .collect()
}

fn looks_like_rule_id(candidate: &str) -> bool {
    let Some((namespace, name)) = candidate.split_once('/') else {
        return false;
    };
    !namespace.is_empty()
        && !name.is_empty()
        && candidate.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'_' | b'@' | b'.' | b'$')
        })
}

fn has_directive(source: &str, element: &Element<'_>, expected: &str) -> bool {
    element.open.attrs.iter().any(|attribute| {
        VueDirectives
            .decompose(attribute.name.text, offset(source, attribute.name.text))
            .ok()
            .flatten()
            .is_some_and(|directive| {
                directive.prefix == DirectivePrefix::Full
                    && directive.name.slice(source) == expected
            })
    })
}

fn comment_body(text: &str) -> &str {
    text.strip_prefix("<!--")
        .and_then(|text| text.strip_suffix("-->"))
        .unwrap_or(text)
}

fn element_span(source: &str, element: &Element<'_>) -> Span {
    let start = offset(source, element.open.lt_name.text);
    let open_end = token_span(source, &element.open.gt).end;
    let child_end = element
        .children
        .iter()
        .map(|child| child_span(source, child).end)
        .max()
        .unwrap_or(open_end);
    let end = match &element.close {
        ElementClose::Present(close) => token_span(source, &close.gt).end,
        ElementClose::Implicit | ElementClose::Missing | ElementClose::NotExpected => {
            open_end.max(child_end)
        }
    };
    Span::new(start, end)
}

fn child_span(source: &str, child: &SurfaceChild<'_>) -> Span {
    match child {
        SurfaceChild::Element(element) => element_span(source, element),
        SurfaceChild::Interpolation(interpolation) => interpolation_span(source, interpolation),
        SurfaceChild::Text(token)
        | SurfaceChild::Comment(token)
        | SurfaceChild::Cdata(token)
        | SurfaceChild::ProcessingInstruction(token)
        | SurfaceChild::Unexpected(token) => token_span(source, token),
    }
}

fn interpolation_span(source: &str, interpolation: &Interpolation<'_>) -> Span {
    Span::new(
        offset(source, interpolation.open.text),
        token_span(source, &interpolation.close).end,
    )
}

fn token_span(source: &str, token: &Token<'_>) -> Span {
    let start = offset(source, token.text);
    Span::new(start, start + token.text.len() as u32)
}

fn offset(source: &str, slice: &str) -> u32 {
    let offset = (slice.as_ptr() as usize).saturating_sub(source.as_ptr() as usize);
    debug_assert!(offset <= source.len() && offset <= u32::MAX as usize);
    offset as u32
}

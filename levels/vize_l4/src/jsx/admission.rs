use vize_l0::{FxHashSet, Span, dom_tag_config};
use vize_l2::lang::js::JsxNode;
use vize_l3::jsx::{JsxDecisionKind as Kind, NativeJsxAnalysis};

use super::{JsxEmitError, JsxEmitErrorKind as Error, invalid, kind};

pub(super) fn check(analysis: &NativeJsxAnalysis<'_>) -> Result<(), JsxEmitError> {
    let owner = analysis.owner();
    let file = owner.file();
    let source = file.artifact().source();
    let failure = |kind| JsxEmitError {
        kind,
        span: Span::new(0, u32::try_from(source.len()).unwrap_or(u32::MAX)),
    };
    let original = owner
        .observation()
        .admitted()
        .ok_or_else(|| failure(Error::InvalidDecision))?;
    let [unit] = file.units() else {
        return Err(failure(Error::SourceWindow));
    };
    if unit.profile.typescript || original.source_type().is_typescript() {
        return Err(failure(Error::Typescript));
    }
    if !file.is_complete()
        || unit.span.start != 0
        || unit.span.end as usize != source.len()
        || !core::ptr::eq(original.source(), source)
    {
        return Err(failure(Error::SourceWindow));
    }
    if original.program().hashbang.is_some() || !original.program().directives.is_empty() {
        return Err(failure(Error::DirectiveOrHashbang));
    }
    for comment in owner.observation().comments() {
        let span = comment.content_span();
        let text = source
            .get(span.start as usize..span.end as usize)
            .ok_or_else(|| failure(Error::InvalidDecision))?;
        // The pinned Vue transform recognizes factory pragmas in actual
        // parser comments. A source string containing these bytes is unrelated.
        if text.match_indices("@jsx").any(|(offset, _)| {
            text.get(offset + 4..).is_some_and(|suffix| {
                suffix.starts_with(js_whitespace)
                    && !suffix.trim_start_matches(js_whitespace).is_empty()
            })
        }) {
            return Err(JsxEmitError {
                kind: Error::FactoryPragma,
                span: Span::new(comment.span.start, comment.span.end),
            });
        }
    }
    for decision in analysis.decisions() {
        let node = decision.node();
        if matches!(decision.kind(), Kind::Opening { .. } | Kind::Closing) {
            let span = node.span().ok_or_else(|| invalid(node))?;
            if owner
                .observation()
                .comments()
                .iter()
                .any(|comment| span.start <= comment.span.start && comment.span.end <= span.end)
            {
                return Err(JsxEmitError::at(Error::TagComment, node));
            }
        }
        match decision.kind() {
            Kind::Intrinsic(name)
                if name == "search"
                    || (!dom_tag_config::is_html_tag(name)
                        && !dom_tag_config::is_svg_tag(name)) =>
            {
                return Err(JsxEmitError::at(Error::UnknownIntrinsic, node));
            }
            Kind::Member => return Err(JsxEmitError::at(Error::MemberTag, node)),
            Kind::Text(_) if has_entities(node)? => {
                return Err(JsxEmitError::at(Error::TextNormalization, node));
            }
            Kind::AttributeString(_) if has_entities(node)? => {
                return Err(JsxEmitError::at(Error::AttributeNormalization, node));
            }
            Kind::Opening { .. } => {
                let mut names = FxHashSet::default();
                for attribute in node.children().skip(1) {
                    let name = match kind(analysis, attribute)? {
                        Kind::StaticAttribute { name, .. } | Kind::ExpressionAttribute { name } => {
                            name
                        }
                        _ => return Err(invalid(attribute)),
                    };
                    if !names.insert(name) {
                        return Err(JsxEmitError::at(Error::DuplicateAttribute, attribute));
                    }
                }
            }
            Kind::Element => check_element(analysis, node)?,
            _ => {}
        }
    }
    Ok(())
}

fn js_whitespace(character: char) -> bool {
    character.is_whitespace() || character == '\u{feff}'
}

fn has_entities(node: JsxNode<'_, '_>) -> Result<bool, JsxEmitError> {
    Ok(node.source().ok_or_else(|| invalid(node))?.contains('&'))
}

fn check_element(
    analysis: &NativeJsxAnalysis<'_>,
    node: JsxNode<'_, '_>,
) -> Result<(), JsxEmitError> {
    let mut children = node.children();
    let opening = children.next().ok_or_else(|| invalid(node))?;
    let name = opening.children().next().ok_or_else(|| invalid(opening))?;
    if matches!(kind(analysis, name)?, Kind::Component(_)) {
        for child in children {
            if !matches!(kind(analysis, child)?, Kind::Closing | Kind::EmptyContainer) {
                return Err(JsxEmitError::at(Error::ComponentChildren, child));
            }
        }
    }
    Ok(())
}

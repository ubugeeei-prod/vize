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
    for decision in analysis.decisions() {
        let node = decision.node();
        match decision.kind() {
            Kind::Intrinsic(name)
                if !dom_tag_config::is_html_tag(name) && !dom_tag_config::is_svg_tag(name) =>
            {
                return Err(JsxEmitError::at(Error::UnknownIntrinsic, node));
            }
            Kind::Member => return Err(JsxEmitError::at(Error::MemberTag, node)),
            Kind::Text(_) if needs_normalization(node)? => {
                return Err(JsxEmitError::at(Error::TextNormalization, node));
            }
            Kind::AttributeString(_) if needs_normalization(node)? => {
                return Err(JsxEmitError::at(Error::AttributeNormalization, node));
            }
            Kind::Opening { .. } => {
                let mut names = FxHashSet::default();
                for attribute in node.children().skip(1) {
                    let Kind::StaticAttribute { name, .. } = kind(analysis, attribute)? else {
                        return Err(invalid(attribute));
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

fn needs_normalization(node: JsxNode<'_, '_>) -> Result<bool, JsxEmitError> {
    Ok(node
        .source()
        .ok_or_else(|| invalid(node))?
        .contains(['&', '\n', '\r', '\t']))
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

//! The locals of a `defineProps` destructure.
//!
//! `const { a, b: c = 1, ...rest } = defineProps()` reads prop `a` as `a`,
//! prop `b` as `c` with default `1`, and the remaining props as `rest`. Vue
//! binds a local named like its key as a prop, a renamed local as an aliased
//! prop, and the rest object as a reactive constant.

use super::bindings::{SetupBinding, SetupBindingKind};
use alloc::vec::Vec;
use oxc_ast::ast::{BindingPattern, ObjectPattern, PropertyKey};
use oxc_span::GetSpan;
use vize_l0::Span;

/// One `defineProps` destructure. Spans are authored file coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropsDestructure {
    /// The object pattern.
    pub pattern: Span,
    /// Destructured props in source order.
    pub fields: Vec<PropsField>,
    /// The rest element's identifier.
    pub rest: Option<Span>,
}

/// One destructured prop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropsField {
    /// The prop key's text: an identifier, or a string literal's contents.
    pub key: Span,
    /// The local identifier.
    pub local: Span,
    /// The default value expression.
    pub default: Option<Span>,
}

/// The destructure and its bindings, or `None` for a pattern Vue rejects
/// (computed or numeric key, nested pattern) or one whose key text is not
/// its source text (an escaped string literal).
pub(super) fn destructure(
    pattern: &ObjectPattern<'_>,
    mut authored: impl FnMut(oxc_span::Span) -> Span,
) -> Option<(PropsDestructure, Vec<SetupBinding>)> {
    let mut fields = Vec::with_capacity(pattern.properties.len());
    let mut bindings = Vec::with_capacity(pattern.properties.len() + 1);
    for property in &pattern.properties {
        let (key, key_span) = match &property.key {
            PropertyKey::StaticIdentifier(id) if !property.computed => (id.name.as_str(), id.span),
            PropertyKey::StringLiteral(literal)
                if literal.value.len() + 2 == literal.span.size() as usize =>
            {
                let inner = oxc_span::Span::new(literal.span.start + 1, literal.span.end - 1);
                (literal.value.as_str(), inner)
            }
            _ => return None,
        };
        let (local, default) = match &property.value {
            BindingPattern::BindingIdentifier(id) => (id, None),
            BindingPattern::AssignmentPattern(assignment) => match &assignment.left {
                BindingPattern::BindingIdentifier(id) => (id, Some(assignment.right.span())),
                _ => return None,
            },
            _ => return None,
        };
        let kind = if local.name == key {
            SetupBindingKind::Props
        } else {
            SetupBindingKind::PropsAliased
        };
        let local_span = authored(local.span);
        fields.push(PropsField {
            key: authored(key_span),
            local: local_span,
            default: default.map(&mut authored),
        });
        bindings.push(SetupBinding {
            name: local_span,
            kind,
            source: None,
        });
    }
    let rest = match pattern.rest.as_deref().map(|rest| &rest.argument) {
        None => None,
        Some(BindingPattern::BindingIdentifier(id)) => {
            let name = authored(id.span);
            bindings.push(SetupBinding {
                name,
                kind: SetupBindingKind::SetupReactiveConst,
                source: None,
            });
            Some(name)
        }
        Some(_) => return None,
    };
    let destructure = PropsDestructure {
        pattern: authored(pattern.span),
        fields,
        rest,
    };
    Some((destructure, bindings))
}

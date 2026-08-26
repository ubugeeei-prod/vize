//! Shared helpers of the legacy surface collector
//! ([`super::surface_old`]), split under the source budget: expression
//! texts, the classifier sets, the modifiers-product parser, and the
//! scope/ambiguity predicates.

use vize_atelier_core::{DirectiveNode, ElementNode, ElementType, ExpressionNode, PropNode};
use vize_s0::String;

use super::surface::{PName, is_simple_ident};

pub fn trimmed(text: &str) -> String {
    String::from(text.trim())
}

/// One expression position: `None` = a compound rebuild.
pub fn exp_text(exp: &ExpressionNode<'_>) -> Option<String> {
    match exp {
        ExpressionNode::Simple(simple) => Some(trimmed(simple.content)),
        ExpressionNode::Compound(_) => None,
    }
}

/// A directive's name position as the shared shape.
pub fn arg_name(dir: &DirectiveNode<'_>) -> PName {
    match dir.arg.as_ref() {
        None => PName::Spread,
        Some(ExpressionNode::Simple(exp)) if exp.is_static => {
            PName::Static(String::from(exp.content))
        }
        Some(exp) => PName::Dynamic(exp_text_of(exp)),
    }
}

pub fn exp_text_of(exp: &ExpressionNode<'_>) -> Option<String> {
    match exp {
        ExpressionNode::Simple(simple) => Some(trimmed(simple.content)),
        ExpressionNode::Compound(_) => None,
    }
}

/// The names the S1→S2 classifier treats as built-in directives — the
/// exclusion set of the custom-directive kind (mirrors
/// `vize_s1_to_s2::lower::directive::classify`).
pub fn is_builtin_name(name: &str) -> bool {
    matches!(
        name,
        "if" | "else-if"
            | "else"
            | "for"
            | "model"
            | "slot"
            | "bind"
            | "on"
            | "html"
            | "text"
            | "show"
            | "cloak"
            | "once"
            | "pre"
            | "memo"
    )
}

/// The still-deferred set (plus structural residue the S2 lowering drops
/// under `drop.structural-duplicate`).
pub fn is_excluded_builtin(name: &str) -> bool {
    matches!(
        name,
        "html"
            | "text"
            | "show"
            | "cloak"
            | "once"
            | "memo"
            | "pre"
            | "if"
            | "else-if"
            | "else"
            | "for"
    )
}

/// Parse the modifiers-product object (`{ lazy: true, trim: true }`)
/// back into names.
pub fn product_modifiers(exp: &ExpressionNode<'_>) -> Vec<String> {
    let ExpressionNode::Simple(simple) = exp else {
        return Vec::new();
    };
    simple
        .content
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .split(',')
        .filter_map(|part| {
            let name = part.trim().strip_suffix(": true")?.trim();
            (!name.is_empty()).then(|| String::from(name))
        })
        .collect()
}

/// Whether the element's authored surface makes the two lanes' component
/// classifiers disagree (the shared ambiguity detector of the
/// `models_classifier`-free design: such owners flow into the
/// pattern-scope-style skip through [`PSurface::pattern_scoped`] only
/// when they carry models — see `classifier_ambiguous`).
pub fn classifier_ambiguous(el: &ElementNode<'_>) -> bool {
    let has_is = el.props.iter().any(|prop| match prop {
        PropNode::Attribute(attr) => attr.name == "is",
        PropNode::Directive(dir) => {
            dir.name == "is"
                || (dir.name == "bind"
                    && matches!(dir.arg.as_ref(), Some(ExpressionNode::Simple(exp)) if exp.content == "is"))
        }
    });
    if has_is {
        return true;
    }
    let native = vize_s0::is_native_tag(el.tag);
    let legacy_component = el.tag == "component"
        || el.tag.chars().next().is_some_and(char::is_uppercase)
        || el.tag.contains('-');
    // The S2 classifier is `!is_native_tag`; disagreement is exactly a
    // non-native tag the legacy default options leave as an element.
    !native && !legacy_component && el.tag != "template" && el.tag != "slot"
}

/// The slot scope an owner opens over its children, when one applies
/// (`enter_v_slot_scope_if_needed` mirrored): `Some(pattern)` where
/// `pattern` says whether the params are a destructuring pattern.
pub fn slot_scope(el: &ElementNode<'_>) -> Option<bool> {
    if el.children.is_empty() || (el.tag_type != ElementType::Component && el.tag != "template") {
        return None;
    }
    let dir = el.props.iter().find_map(|prop| match prop {
        PropNode::Directive(dir) if dir.name == "slot" => Some(dir.as_ref()),
        _ => None,
    })?;
    let params = dir.exp.as_ref().and_then(exp_text_of)?;
    if params.is_empty() {
        return None;
    }
    Some(!is_simple_ident(params.as_str()))
}

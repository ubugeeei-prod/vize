//! Ref targets of `<component>`: a dynamic `:is` target, or a static `is`.

use oxc_ast::ast::Expression;
use vize_carton::{String, append, is_native_tag};
use vize_croquis::Croquis;
use vize_relief::{ElementNode, Namespace, PropNode};

use super::{RegisteredRefKind, push_ts_string_literal};

pub(super) fn dynamic_component_reference(
    element: &ElementNode<'_>,
    summary: &Croquis,
) -> Option<String> {
    if element.tag != "component" {
        return None;
    }
    let expression = element.props.iter().find_map(|prop| {
        let PropNode::Directive(directive) = prop else {
            return None;
        };
        if directive.name != "bind"
            || !matches!(
                directive.arg.as_ref(),
                Some(vize_relief::ExpressionNode::Simple(arg))
                    if arg.is_static && arg.content == "is"
            )
        {
            return None;
        }
        directive.exp.as_ref()
    })?;
    if let vize_relief::ExpressionNode::Simple(expression) = expression {
        let reference = expression.content.trim();
        let mut segments = reference.split('.');
        if let Some(first) = segments.next()
            && is_ascii_identifier(first)
        {
            let rest = segments.collect::<Vec<_>>();
            if rest.iter().all(|segment| is_ascii_identifier(segment)) {
                if rest.is_empty() {
                    // The template reads a ref binding unwrapped: `:is="tag"`
                    // with `const tag = ref<'div' | 'span'>()` renders the tag.
                    return Some(if unwrapped_in_template(summary, first) {
                        vize_carton::cstr!("import('vue').UnwrapRef<typeof {first}>")
                    } else {
                        vize_carton::cstr!("typeof {first}")
                    });
                }
                let mut reference_type =
                    vize_carton::cstr!("import('vue').UnwrapRef<typeof {first}>");
                for segment in rest {
                    append!(reference_type, "[\"{segment}\"]");
                }
                return Some(reference_type);
            }
        }

        // The inference-only alias for a complex root target lives inside the
        // template closure, while this registry is declared in setup scope.
        // Project supported target unions directly from the retained AST so
        // the registry never names that inner alias.
        let alias = vize_croquis::drawer::dynamic_component_alias(element.loc.span.start);
        let owns_alias = vize_croquis::facts::component_usage_slice(summary)
            .iter()
            .any(|usage| {
                usage.start == element.loc.span.start && usage.name.as_str() == alias.as_str()
            });
        if owns_alias {
            let retained = expression.js_ast?.as_expression()?;
            return dynamic_component_target_type(retained.ast);
        }
    }
    None
}

/// Whether the template auto-unwraps the setup binding `name`.
fn unwrapped_in_template(summary: &Croquis, name: &str) -> bool {
    use vize_croquis::BindingType;
    matches!(
        crate::virtual_ts::script_facts::binding_type(summary, name),
        Some(BindingType::SetupRef | BindingType::SetupMaybeRef | BindingType::SetupLet)
    )
}

/// The ref of `<component is="div">`: a static `is` names its target directly.
/// A native tag renders that element. Any other string reaches
/// `resolveDynamicComponent` verbatim (`vue:` included), which resolves only
/// registered components, never a `<script setup>` import of the same name.
pub(super) fn static_component_is_kind(element: &ElementNode<'_>) -> Option<RegisteredRefKind> {
    if element.tag != "component" {
        return None;
    }
    let target = element.props.iter().find_map(|prop| match prop {
        PropNode::Attribute(attribute) if attribute.name == "is" => {
            attribute.value.as_ref().map(|value| value.content)
        }
        _ => None,
    })?;
    if target.is_empty() {
        return None;
    }
    if is_native_tag(target) {
        return Some(RegisteredRefKind::Element {
            tag: String::from(target),
            is_svg: matches!(element.ns, Namespace::Svg),
        });
    }
    Some(RegisteredRefKind::GlobalOrElement {
        tag: String::from(target),
        is_svg: matches!(element.ns, Namespace::Svg),
    })
}

fn dynamic_component_target_type(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::Identifier(identifier) => {
            Some(vize_carton::cstr!("typeof {}", identifier.name))
        }
        Expression::StringLiteral(literal) => {
            let mut reference = crate::virtual_ts::TextBuf::default();
            push_ts_string_literal(&mut reference, literal.value.as_str());
            Some(reference.into_compact())
        }
        Expression::ConditionalExpression(conditional) => {
            let consequent = dynamic_component_target_type(&conditional.consequent)?;
            let alternate = dynamic_component_target_type(&conditional.alternate)?;
            Some(vize_carton::cstr!("{consequent} | {alternate}"))
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            dynamic_component_target_type(&parenthesized.expression)
        }
        Expression::TSAsExpression(assertion) => {
            dynamic_component_target_type(&assertion.expression)
        }
        Expression::TSSatisfiesExpression(satisfies) => {
            dynamic_component_target_type(&satisfies.expression)
        }
        Expression::TSNonNullExpression(non_null) => {
            dynamic_component_target_type(&non_null.expression)
        }
        _ => None,
    }
}

fn is_ascii_identifier(segment: &str) -> bool {
    let mut chars = segment.chars();
    chars
        .next()
        .is_some_and(|char| char == '_' || char == '$' || char.is_ascii_alphabetic())
        && chars.all(|char| char == '_' || char == '$' || char.is_ascii_alphanumeric())
}

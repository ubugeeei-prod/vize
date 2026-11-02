//! Keep nested callback values in one contextual whole-props check.

use oxc_allocator::Allocator;
use oxc_ast::ast::{ArrowFunctionExpression, Function};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use oxc_syntax::scope::ScopeFlags;
use vize_croquis::croquis::{ComponentUsage, PassedProp};

pub(super) fn whole_literal_owns_value(
    usage: &ComponentUsage,
    prop: &PassedProp,
    typed_fallthrough: bool,
) -> bool {
    if typed_fallthrough
        || !usage.spread_props.is_empty()
        || !prop.is_dynamic
        || matches!(prop.name.as_str(), "class" | "style")
        || crate::virtual_ts::helpers::component_aria_prop_keys(prop.name.as_str()).is_some()
        || super::super::model_update::is_model_prop(usage, prop)
    {
        return false;
    }
    let Some(value) = prop.value.as_deref() else {
        return false;
    };
    if !contains_function(value) || crate::virtual_ts::scope::is_inline_callback_prop(prop) {
        return false;
    }
    let key = crate::virtual_ts::helpers::component_prop_key(prop.name.as_str());
    // A merged or overwritten entry cannot replace each authored value check.
    usage
        .props
        .iter()
        .filter(|other| crate::virtual_ts::helpers::component_prop_key(other.name.as_str()) == key)
        .count()
        == 1
}

fn contains_function(value: &str) -> bool {
    let value = value.trim();
    // Scalars and ordinary calls keep the allocation-free path.
    if !(value.contains("=>")
        || value.contains("function")
        || (value.starts_with(['{', '[']) && value.contains('(')))
        || !vize_l0::expression_guard::expression_is_safe_to_parse(value)
    {
        return false;
    }
    let allocator = Allocator::new();
    let Ok(expression) = Parser::new(&allocator, value, SourceType::ts()).parse_expression() else {
        return false;
    };
    let span = expression.span();
    if value.get(span.start as usize..span.end as usize) != Some(value) {
        return false;
    }
    let mut visitor = Functions(false);
    visitor.visit_expression(&expression);
    visitor.0
}

struct Functions(bool);
impl<'a> Visit<'a> for Functions {
    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {
        self.0 = true;
    }
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {
        self.0 = true;
    }
}

#[cfg(test)]
mod tests {
    use super::contains_function;

    #[test]
    fn real_function_nodes_distinguish_callbacks_from_quoted_markers_and_recovery() {
        for value in [
            "{ run: () => 'ok' }",
            "{ run(value) { return value } }",
            "[function() {}]",
        ] {
            assert!(contains_function(value), "{value}");
        }
        for value in [
            "'a=>b'",
            "/=>/.test(value)",
            "{ text: 'a=>b' }",
            "items.map(format)",
            "{ run: () =>",
            "{ run: () => 1 } trailing",
        ] {
            assert!(!contains_function(value), "{value}");
        }
    }
}

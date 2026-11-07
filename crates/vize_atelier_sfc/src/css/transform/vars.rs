//! JavaScript values paired with CSS `v-bind()` property names.

use crate::types::TemplateCompileOptions;
use crate::vite_plugin::js_string::push_js_string_literal;
use std::borrow::Cow;
use vize_atelier_core::{
    BindingMetadata, CompoundExpressionChild, ExpressionNode, SimpleExpressionNode, SourceLocation,
    TransformContext, TransformOptions, process_expression,
};
use vize_carton::{Box, String};

pub(crate) fn transform_value(
    expression: &str,
    bindings: Option<&BindingMetadata>,
    inline: bool,
    is_ts: bool,
) -> String {
    let allocator = vize_carton::pool::acquire();
    let exp = ExpressionNode::Simple(Box::new_in(
        SimpleExpressionNode::new(
            expression,
            false,
            SourceLocation::new(0, expression.len() as u32),
        ),
        &&*allocator,
    ));
    let mut context = TransformContext::new(
        &allocator,
        expression,
        TransformOptions {
            prefix_identifiers: true,
            inline,
            is_ts,
            binding_metadata: bindings.cloned(),
            ..Default::default()
        },
    );
    match process_expression(&mut context, &exp, false) {
        ExpressionNode::Simple(simple) => String::from(simple.content),
        ExpressionNode::Compound(compound) => {
            let mut output = String::default();
            for child in &compound.children {
                match child {
                    CompoundExpressionChild::Simple(simple) => output.push_str(simple.content),
                    CompoundExpressionChild::String(text) => output.push_str(text),
                    CompoundExpressionChild::Symbol(helper) => {
                        output.push('_');
                        output.push_str(helper.name());
                    }
                    _ => {}
                }
            }
            output
        }
    }
}

pub(crate) fn inject_ssr_values(
    options: &mut TemplateCompileOptions,
    vars: &[Cow<'_, str>],
    scope_id: &str,
    bindings: Option<&BindingMetadata>,
) {
    if !options.ssr || vars.is_empty() {
        return;
    }
    let mut output = String::from("{");
    for (index, expression) in vars.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let name = if options.is_prod {
            super::prod_scoped_v_bind_name(scope_id, expression)
        } else {
            super::scoped_v_bind_name(scope_id, expression)
        };
        let mut key = String::from(":--");
        key.push_str(&name);
        push_js_string_literal(&mut output, &key);
        output.push_str(": (");
        output.push_str(&transform_value(expression, bindings, false, options.is_ts));
        output.push(')');
    }
    output.push('}');
    options.ssr_css_vars = Some(output);
}

//! Attribute and directive processing methods for the parser.
//!
//! Handles attribute names, directive names/arguments/modifiers,
//! attribute data (values), and finalization of attribute and directive nodes.

use vize_carton::{Box, String, Vec};
use vize_relief::ast::{
    AttributeNode, ConstantType, DirectiveNode, ExpressionNode, PropNode, SimpleExpressionNode,
    TextNode,
};

use crate::tokenizer::QuoteType;

use super::{CurrentAttribute, CurrentDirective, Parser};

impl<'a> Parser<'a> {
    /// Process attribute name
    pub(super) fn on_attrib_name_impl(&mut self, start: usize, end: usize) {
        let name = self.get_source(start, end);
        self.current_attr = Some(CurrentAttribute {
            name: name.into(),
            name_start: start,
            name_end: end,
            value_start: None,
            value_end: None,
            value_content: None,
            _marker: std::marker::PhantomData,
        });
    }

    /// Process directive name
    pub(super) fn on_dir_name_impl(&mut self, start: usize, end: usize) {
        let raw_name = self.get_source(start, end);
        let name = super::callbacks::parse_directive_name(raw_name);

        self.current_dir = Some(CurrentDirective {
            name: name.into(),
            raw_name: raw_name.into(),
            name_start: start,
            name_end: end,
            arg: None,
            modifiers: Vec::new_in(self.allocator),
            value_start: None,
            value_end: None,
            value_content: None,
            _marker: std::marker::PhantomData,
        });
    }

    /// Process directive argument
    pub(super) fn on_dir_arg_impl(&mut self, start: usize, end: usize) {
        let arg: String = self.get_source(start, end).into();
        // Check if dynamic arg (was inside [ ])
        let is_dynamic = start > 0 && self.source.as_bytes().get(start - 1) == Some(&b'[');
        if let Some(ref mut dir) = self.current_dir {
            dir.arg = Some((arg, start, end, is_dynamic));
        }
    }

    /// Process directive modifier
    pub(super) fn on_dir_modifier_impl(&mut self, start: usize, end: usize) {
        let modifier: String = self.get_source(start, end).into();
        if let Some(ref mut dir) = self.current_dir {
            dir.modifiers.push((modifier, start, end));
        }
    }

    /// Process attribute data (value content)
    pub(super) fn on_attrib_data_impl(&mut self, start: usize, end: usize) {
        let source = self.source;
        self.accumulate_attr_or_dir_value(&source[start..end], start, end);
    }

    /// Process attribute entity
    pub(super) fn on_attrib_entity_impl(&mut self, ch: char, start: usize, end: usize) {
        let mut content = [0_u8; 4];
        self.accumulate_attr_or_dir_value(ch.encode_utf8(&mut content), start, end);
    }

    /// Helper to accumulate attribute or directive value
    fn accumulate_attr_or_dir_value(&mut self, content: &str, start: usize, end: usize) {
        // Update current attribute
        if let Some(ref mut attr) = self.current_attr {
            Self::accumulate_value(
                &mut attr.value_content,
                &mut attr.value_start,
                &mut attr.value_end,
                content,
                start,
                end,
            );
        }

        // Update current directive
        if let Some(ref mut dir) = self.current_dir {
            Self::accumulate_value(
                &mut dir.value_content,
                &mut dir.value_start,
                &mut dir.value_end,
                content,
                start,
                end,
            );
        }
    }

    /// Helper to accumulate value content
    #[inline]
    fn accumulate_value(
        content_field: &mut Option<String>,
        start_field: &mut Option<usize>,
        end_field: &mut Option<usize>,
        content: &str,
        start: usize,
        end: usize,
    ) {
        if content_field.is_none() {
            *start_field = Some(start);
            *content_field = Some(content.into());
        } else if let Some(existing) = content_field.as_mut() {
            existing.push_str(content);
        }
        *end_field = Some(end);
    }

    /// Process attribute end
    pub(super) fn on_attrib_end_impl(&mut self, quote: QuoteType, end: usize) {
        // Handle regular attribute
        if let Some(attr) = self.current_attr.take() {
            self.finish_attribute(attr, quote, end);
        }

        // Handle directive
        if let Some(dir) = self.current_dir.take() {
            self.finish_directive(dir, quote, end);
        }
    }

    /// Finish building an attribute node
    fn finish_attribute(&mut self, attr: CurrentAttribute<'a>, quote: QuoteType, end: usize) {
        let loc = self.create_loc(attr.name_start, end);
        let name_loc = self.create_loc(attr.name_start, attr.name_end);

        let mut attr_node = AttributeNode::new(attr.name.clone(), loc);
        attr_node.name_loc = name_loc;

        // Add value if present
        if let (Some(v_start), Some(v_end), Some(v_content)) =
            (attr.value_start, attr.value_end, attr.value_content)
        {
            let value_loc = self.create_loc(v_start, v_end);
            attr_node.value = Some(TextNode::new(v_content, value_loc));
        } else if matches!(quote, QuoteType::Double | QuoteType::Single) {
            // alt="" or alt='' → empty string value (not boolean "true")
            let empty_loc: vize_relief::SourceLocation = self.create_loc(end, end);
            attr_node.value = Some(TextNode::new("", empty_loc));
        }

        if let Some(ref mut current) = self.current_element {
            let boxed = Box::new_in(attr_node, self.allocator);
            current.props.push(PropNode::Attribute(boxed));
        }
    }

    /// Finish building a directive node
    fn finish_directive(&mut self, dir: CurrentDirective<'a>, _quote: QuoteType, end: usize) {
        let loc = self.create_loc(dir.name_start, end);

        let mut dir_node = DirectiveNode::new(self.allocator, dir.name.clone(), loc);
        dir_node.raw_name = Some(dir.raw_name);

        // Vue 3.4+ same-name shorthand: `:foo` without a value is `:foo="foo"`
        // Pre-compute the shorthand expression before moving dir.arg
        let shorthand_exp = if dir.name == "bind" && dir.value_start.is_none() {
            if let Some((ref arg_content, arg_start, arg_end, false)) = dir.arg {
                Some((vize_carton::camelize(arg_content), arg_start, arg_end))
            } else {
                None
            }
        } else {
            None
        };

        // Add argument if present
        if let Some((arg_content, arg_start, arg_end, is_dynamic)) = dir.arg {
            let arg_loc = self.create_loc(arg_start, arg_end);
            let mut arg_expr = SimpleExpressionNode::new(arg_content, !is_dynamic, arg_loc);
            if is_dynamic {
                arg_expr.const_type = ConstantType::NotConstant;
            }
            let arg_boxed = Box::new_in(arg_expr, self.allocator);
            dir_node.arg = Some(ExpressionNode::Simple(arg_boxed));
        }

        // Add modifiers
        for (mod_content, mod_start, mod_end) in dir.modifiers {
            let mod_loc = self.create_loc(mod_start, mod_end);
            let mod_expr = SimpleExpressionNode::new(mod_content, true, mod_loc);
            dir_node.modifiers.push(mod_expr);
        }

        // Add expression if present
        if let (Some(v_start), Some(v_end), Some(v_content)) =
            (dir.value_start, dir.value_end, dir.value_content)
        {
            let exp_loc = self.create_loc(v_start, v_end);
            let exp_node = SimpleExpressionNode::new(v_content, false, exp_loc);
            let exp_boxed = Box::new_in(exp_node, self.allocator);
            dir_node.exp = Some(ExpressionNode::Simple(exp_boxed));
        } else if let Some((camelized, s_start, s_end)) = shorthand_exp {
            // Apply same-name shorthand: synthesize expression from arg name
            let exp_loc = self.create_loc(s_start, s_end);
            let exp_node = SimpleExpressionNode::new(&*camelized, false, exp_loc);
            let exp_boxed = Box::new_in(exp_node, self.allocator);
            dir_node.exp = Some(ExpressionNode::Simple(exp_boxed));
            dir_node.shorthand = true;
        }

        if let Some(ref mut current) = self.current_element {
            let boxed = Box::new_in(dir_node, self.allocator);
            current.props.push(PropNode::Directive(boxed));
        }
    }
}

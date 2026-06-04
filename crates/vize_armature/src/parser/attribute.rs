//! Attribute and directive processing methods for the parser.
//!
//! Handles attribute names, directive names/arguments/modifiers,
//! attribute data (values), and finalization of attribute and directive nodes.

use vize_carton::{Box, String, Vec, appends};
use vize_relief::{
    ast::{
        AttributeNode, ConstantType, DirectiveNode, ExpressionNode, PropNode, SimpleExpressionNode,
        TextNode,
    },
    errors::{CompilerError, ErrorCode},
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

    /// Process the end of a full attribute or directive head.
    pub(super) fn on_attrib_name_end_impl(&mut self, end: usize) {
        if let Some(ref mut attr) = self.current_attr {
            attr.name_end = end;
        }

        if let Some(ref mut dir) = self.current_dir {
            dir.name_end = end;
        }
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
        if start >= end {
            let loc = self.create_loc(start, end);
            self.errors.push(CompilerError::new(
                ErrorCode::MissingDirectiveModifier,
                Some(loc),
            ));
            return;
        }

        let modifier: String = self.get_source(start, end).into();
        if let Some(ref mut dir) = self.current_dir {
            dir.modifiers.push((modifier, start, end));
        }
    }

    /// Process attribute data (value content)
    pub(super) fn on_attrib_data_impl(&mut self, start: usize, end: usize) {
        let source = self.get_source(start, end).to_owned();
        self.accumulate_attr_or_dir_value(&source, start, end);
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

    fn prop_loc_end(&self, quote: QuoteType, end: usize, name_end: usize) -> usize {
        match quote {
            QuoteType::NoValue => name_end,
            QuoteType::Double | QuoteType::Single => (end + 1).min(self.source.len()),
            QuoteType::Unquoted => end,
        }
    }

    fn has_duplicate_attribute(&self, name: &str) -> bool {
        self.current_element.as_ref().is_some_and(|current| {
            current.props.iter().any(|prop| {
                matches!(prop, PropNode::Attribute(existing) if existing.name.as_str() == name)
            })
        })
    }

    /// Finish building an attribute node
    fn finish_attribute(&mut self, attr: CurrentAttribute<'a>, quote: QuoteType, end: usize) {
        let loc_end = self.prop_loc_end(quote, end, attr.name_end);
        let loc = self.create_loc(attr.name_start, loc_end);
        let name_loc = self.create_loc(attr.name_start, attr.name_end);

        if self.has_duplicate_attribute(attr.name.as_str()) {
            // Vue recovers from duplicate attributes — codegen ignores
            // repeats and keeps the first occurrence — but linters still
            // want to see both in the AST so they can warn about the
            // repeat. So we KEEP both in `props` and emit a
            // *recoverable* diagnostic; downstream pipelines treat it
            // as non-fatal so a `<div id=a id=b>` no longer yields a
            // 0-byte module marked as success. (#958)
            let mut message = String::with_capacity(attr.name.len() + 79);
            appends!(
                message,
                "Duplicate attribute `",
                attr.name.as_str(),
                "`. Keeping the repeated attribute so parsing can continue."
            );
            self.errors.push(CompilerError::with_message(
                ErrorCode::DuplicateAttribute,
                message,
                Some(name_loc.clone()),
            ));
        }

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
    fn finish_directive(&mut self, dir: CurrentDirective<'a>, quote: QuoteType, end: usize) {
        let loc_end = self.prop_loc_end(quote, end, dir.name_end);
        let loc = self.create_loc(dir.name_start, loc_end);

        if dir.name.is_empty() {
            let mut message = String::with_capacity(dir.raw_name.len() + 80);
            appends!(
                message,
                "Directive `",
                dir.raw_name.as_str(),
                "` is missing a name. Ignoring it so the rest of the tag can be parsed."
            );
            self.errors.push(CompilerError::with_message(
                ErrorCode::MissingDirectiveName,
                message,
                Some(loc),
            ));
            return;
        }

        // The `.foo` shorthand is equivalent to `v-bind:foo.prop`: detect it
        // before `raw_name` is moved so we can synthesize a `prop` modifier.
        let dot_shorthand_prop = dir.raw_name.starts_with('.')
            && !dir
                .modifiers
                .iter()
                .any(|(content, _, _)| content == "prop");

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

        // The `.foo` shorthand is equivalent to `v-bind:foo.prop`: synthesize
        // a leading `prop` modifier so codegen treats it as a DOM property bind.
        // (Vue's parser injects this modifier for the dot shorthand.)
        if dot_shorthand_prop {
            let mod_loc = self.create_loc(dir.name_start, dir.name_start + 1);
            let mod_expr = SimpleExpressionNode::new("prop", true, mod_loc);
            dir_node.modifiers.push(mod_expr);
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

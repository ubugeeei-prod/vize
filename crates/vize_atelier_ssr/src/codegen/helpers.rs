//! HTML escaping utilities and child/control-flow processing for SSR codegen.

use vize_atelier_core::ast::{
    CommentNode, ElementType, ForNode, IfNode, InterpolationNode, RuntimeHelper, TemplateChildNode,
    TextNode,
};

use super::SsrCodegenContext;
use vize_carton::{cstr, FxHashSet, String, ToCompactString};

impl<'a> SsrCodegenContext<'a> {
    /// Process a list of children nodes
    pub(crate) fn process_children(
        &mut self,
        children: &[TemplateChildNode<'a>],
        as_fragment: bool,
        disable_nested_fragments: bool,
        disable_comment: bool,
    ) {
        self.process_children_with_fallthrough_attrs(
            children,
            as_fragment,
            disable_nested_fragments,
            disable_comment,
            false,
        );
    }

    /// Process root-level children and inherit `_attrs` into a single renderable
    /// root, matching Vue's fallthrough attrs behavior for SSR.
    pub(crate) fn process_root_children(
        &mut self,
        children: &[TemplateChildNode<'a>],
        as_fragment: bool,
        disable_nested_fragments: bool,
        disable_comment: bool,
    ) {
        self.process_children_with_fallthrough_attrs(
            children,
            as_fragment,
            disable_nested_fragments,
            disable_comment,
            true,
        );
    }

    fn process_children_with_fallthrough_attrs(
        &mut self,
        children: &[TemplateChildNode<'a>],
        as_fragment: bool,
        disable_nested_fragments: bool,
        disable_comment: bool,
        inherit_attrs: bool,
    ) {
        if as_fragment {
            self.push_string_part_static("<!--[-->");
        }

        let fallthrough_child_index = if inherit_attrs && !as_fragment {
            single_fallthrough_child_index(children)
        } else {
            None
        };

        for (index, child) in children.iter().enumerate() {
            self.process_child(
                child,
                disable_nested_fragments,
                disable_comment,
                fallthrough_child_index == Some(index),
            );
        }

        if as_fragment {
            self.push_string_part_static("<!--]-->");
        }
    }

    /// Process a single child node
    pub(crate) fn process_child(
        &mut self,
        child: &TemplateChildNode<'a>,
        disable_nested_fragments: bool,
        disable_comment: bool,
        inherit_attrs: bool,
    ) {
        match child {
            TemplateChildNode::Element(el) => {
                self.process_element_with_fallthrough_attrs(
                    el,
                    disable_nested_fragments,
                    inherit_attrs,
                );
            }
            TemplateChildNode::Text(text) => {
                self.process_text(text);
            }
            TemplateChildNode::Comment(comment) => {
                if !disable_comment {
                    self.process_comment(comment);
                }
            }
            TemplateChildNode::Interpolation(interp) => {
                self.process_interpolation(interp);
            }
            TemplateChildNode::If(if_node) => {
                self.process_if(
                    if_node,
                    disable_nested_fragments,
                    disable_comment,
                    inherit_attrs,
                );
            }
            TemplateChildNode::For(for_node) => {
                self.process_for(for_node, disable_nested_fragments);
            }
            TemplateChildNode::IfBranch(_) => {
                // Handled by process_if
            }
            TemplateChildNode::TextCall(_) | TemplateChildNode::CompoundExpression(_) => {
                // These don't appear in SSR since transformText is not used
            }
            TemplateChildNode::Hoisted(_) => {
                // Hoisting is not used in SSR
            }
        }
    }

    /// Process a text node
    fn process_text(&mut self, text: &TextNode) {
        self.push_string_part_static(&escape_html(&text.content));
    }

    /// Process a comment node
    fn process_comment(&mut self, comment: &CommentNode) {
        self.push_string_part_static("<!--");
        self.push_string_part_static(&comment.content);
        self.push_string_part_static("-->");
    }

    /// Process an interpolation node ({{ expr }})
    fn process_interpolation(&mut self, interp: &InterpolationNode) {
        use vize_atelier_core::ast::ExpressionNode;

        self.use_ssr_helper(RuntimeHelper::SsrInterpolate);

        let exp = match &interp.content {
            ExpressionNode::Simple(simple) => self.strip_ctx_for_scoped_params(&simple.content),
            ExpressionNode::Compound(_) => "_ctx.value".to_compact_string(), // placeholder
        };

        self.push_string_part_dynamic(&cstr!("_ssrInterpolate({exp})"));
    }

    /// Process an if node
    pub(crate) fn process_if(
        &mut self,
        if_node: &IfNode<'a>,
        disable_nested_fragments: bool,
        disable_comment: bool,
        inherit_attrs: bool,
    ) {
        // Flush current push before if statement
        self.flush_push();

        for (i, branch) in if_node.branches.iter().enumerate() {
            self.push_indent();

            if i == 0 {
                // First branch: if
                self.push("if (");
                if let Some(condition) = &branch.condition {
                    self.push_expression(condition);
                }
                self.push(") {\n");
            } else if branch.condition.is_some() {
                // else-if
                self.push("} else if (");
                if let Some(condition) = &branch.condition {
                    self.push_expression(condition);
                }
                self.push(") {\n");
            } else {
                // else
                self.push("} else {\n");
            }

            self.indent_level += 1;

            // Check if branch needs fragment
            let needs_fragment =
                !disable_nested_fragments && rendered_child_count(&branch.children) > 1;

            self.process_children_with_fallthrough_attrs(
                &branch.children,
                needs_fragment,
                disable_nested_fragments,
                disable_comment,
                inherit_attrs,
            );
            self.flush_push();

            self.indent_level -= 1;
        }

        // If no else branch, emit empty comment
        if if_node.branches.iter().all(|b| b.condition.is_some()) {
            self.push_indent();
            self.push("} else {\n");
            self.indent_level += 1;
            self.push_string_part_static("<!---->");
            self.flush_push();
            self.indent_level -= 1;
        }

        self.push_indent();
        self.push("}\n");
    }

    /// Process a for node
    pub(crate) fn process_for(&mut self, for_node: &ForNode<'a>, disable_nested_fragments: bool) {
        // Flush current push before for statement
        self.flush_push();

        self.use_ssr_helper(RuntimeHelper::SsrRenderList);

        // Fragment markers for v-for
        if !disable_nested_fragments {
            self.push_indent();
            self.push("_push(`<!--[-->`)\n");
        }

        self.push_indent();
        self.push("_ssrRenderList(");
        self.push_expression(&for_node.source);
        self.push(", (");

        // Value alias
        if let Some(value) = &for_node.value_alias {
            self.push_expression(value);
        }
        // Key alias
        if let Some(key) = &for_node.key_alias {
            self.push(", ");
            self.push_expression(key);
        }
        // Index alias
        if let Some(index) = &for_node.object_index_alias {
            self.push(", ");
            self.push_expression(index);
        }

        self.push(") => {\n");
        self.indent_level += 1;

        self.push_scoped_params(collect_for_scoped_params(for_node));

        // Process for body
        let needs_fragment =
            !disable_nested_fragments && rendered_child_count(&for_node.children) > 1;
        self.process_children(&for_node.children, needs_fragment, true, false);
        self.flush_push();

        self.pop_scoped_params();

        self.indent_level -= 1;
        self.push_indent();
        self.push("})\n");

        // Closing fragment marker
        if !disable_nested_fragments {
            self.push_indent();
            self.push("_push(`<!--]-->`)\n");
        }
    }

    /// Push an expression node
    pub(crate) fn push_expression(&mut self, expr: &vize_atelier_core::ast::ExpressionNode) {
        use vize_atelier_core::ast::ExpressionNode;

        match expr {
            ExpressionNode::Simple(simple) => {
                let content = self.strip_ctx_for_scoped_params(&simple.content);
                self.push(&content);
            }
            ExpressionNode::Compound(compound) => {
                // Flatten compound expression
                let mut content = String::default();
                for child in &compound.children {
                    use vize_atelier_core::ast::CompoundExpressionChild;
                    match child {
                        CompoundExpressionChild::Simple(s) => content.push_str(&s.content),
                        CompoundExpressionChild::String(s) => content.push_str(s),
                        CompoundExpressionChild::Symbol(helper) => {
                            content.push('_');
                            content.push_str(helper.name());
                        }
                        _ => {}
                    }
                }
                let content = self.strip_ctx_for_scoped_params(&content);
                self.push(&content);
            }
        }
    }
}

fn collect_for_scoped_params(for_node: &ForNode) -> FxHashSet<String> {
    let mut params = FxHashSet::default();

    if let Some(value) = &for_node.value_alias {
        collect_expression_params(value, &mut params);
    }
    if let Some(key) = &for_node.key_alias {
        collect_expression_params(key, &mut params);
    }
    if let Some(index) = &for_node.object_index_alias {
        collect_expression_params(index, &mut params);
    }

    params
}

pub(crate) fn collect_expression_params(
    expr: &vize_atelier_core::ast::ExpressionNode,
    params: &mut FxHashSet<String>,
) {
    let content = match expr {
        vize_atelier_core::ast::ExpressionNode::Simple(simple) => simple.content.clone(),
        vize_atelier_core::ast::ExpressionNode::Compound(compound) => {
            let mut content = String::default();
            for child in &compound.children {
                use vize_atelier_core::ast::CompoundExpressionChild;
                match child {
                    CompoundExpressionChild::Simple(simple) => content.push_str(&simple.content),
                    CompoundExpressionChild::String(value) => content.push_str(value),
                    _ => {}
                }
            }
            if content.is_empty() {
                compound.loc.source.clone()
            } else {
                content
            }
        }
    };
    extract_destructure_params(content.trim(), params);
}

pub(crate) fn extract_destructure_params(value: &str, params: &mut FxHashSet<String>) {
    if value.starts_with('(') && value.ends_with(')') {
        extract_destructure_params(value[1..value.len() - 1].trim(), params);
        return;
    }

    if value.contains(',') && !value.starts_with('{') && !value.starts_with('[') {
        for part in split_top_level(value) {
            extract_destructure_params(part.trim(), params);
        }
        return;
    }

    if value.starts_with('{') && value.ends_with('}') {
        for part in split_top_level(&value[1..value.len() - 1]) {
            let part = part.trim();
            if let Some(rest) = part.strip_prefix("...") {
                collect_identifier_param(rest.trim(), params);
                continue;
            }
            if let Some(eq_pos) = part.find('=') {
                extract_destructure_params(part[..eq_pos].trim(), params);
                continue;
            }
            if let Some(colon_pos) = part.find(':') {
                extract_destructure_params(part[colon_pos + 1..].trim(), params);
                continue;
            }
            extract_destructure_params(part, params);
        }
    } else if value.starts_with('[') && value.ends_with(']') {
        for part in split_top_level(&value[1..value.len() - 1]) {
            let part = part.trim();
            if let Some(rest) = part.strip_prefix("...") {
                collect_identifier_param(rest.trim(), params);
            } else {
                extract_destructure_params(part, params);
            }
        }
    } else {
        collect_identifier_param(value, params);
    }
}

fn collect_identifier_param(value: &str, params: &mut FxHashSet<String>) {
    if is_valid_identifier(value) {
        params.insert(value.to_compact_string());
    }
}

fn split_top_level(value: &str) -> std::vec::Vec<&str> {
    let mut parts = std::vec::Vec::new();
    let mut depth = 0i32;
    let mut start = 0;

    for (index, byte) in value.bytes().enumerate() {
        match byte {
            b'{' | b'[' | b'(' => depth += 1,
            b'}' | b']' | b')' => depth -= 1,
            b',' if depth == 0 => {
                parts.push(&value[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }

    parts.push(&value[start..]);
    parts
}

fn is_valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// Escape HTML special characters
pub(crate) fn escape_html(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            _ => result.push(c),
        }
    }
    result
}

/// Escape HTML attribute value
pub(crate) fn escape_html_attr(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            _ => result.push(c),
        }
    }
    result
}

fn single_fallthrough_child_index(children: &[TemplateChildNode]) -> Option<usize> {
    let mut index = None;

    for (current_index, child) in children.iter().enumerate() {
        if !is_fallthrough_root_candidate(child) {
            continue;
        }

        if index.is_some() {
            return None;
        }
        index = Some(current_index);
    }

    index
}

fn is_fallthrough_root_candidate(child: &TemplateChildNode) -> bool {
    matches!(
        child,
        TemplateChildNode::Element(_) | TemplateChildNode::If(_)
    )
}

fn rendered_child_count(children: &[TemplateChildNode]) -> usize {
    children
        .iter()
        .map(|child| match child {
            TemplateChildNode::Element(el) if el.tag_type == ElementType::Template => {
                rendered_child_count(&el.children)
            }
            _ => 1,
        })
        .sum()
}

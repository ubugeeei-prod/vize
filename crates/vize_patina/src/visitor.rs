//! AST visitor for lint rule execution.
//!
//! High-performance visitor with minimal allocations.

use crate::context::{ElementContext, LintContext};
use crate::rule::Rule;
use vize_carton::directive::{DirectiveKind, parse_level_severity, parse_vize_directive};
use vize_carton::{CompactString, cstr, profile};
use vize_relief::ast::{
    CommentNode, ElementNode, ExpressionNode, PropNode, RootNode, SourceLocation, TemplateChildNode,
};

/// Visit the AST and run all rules
pub struct LintVisitor<'a, 'ctx, 'rules> {
    ctx: &'ctx mut LintContext<'a>,
    rules: &'rules [Box<dyn Rule>],
    rule_names: &'rules [&'static str],
    run_exit_element_rules: bool,
    /// When true, suppress all diagnostics for the next element
    forget_next_element: bool,
    /// Optional per-rule keep mask, parallel to `rules`. When `Some`, a rule is
    /// dispatched only where its entry is `true`. Used by the JSX/TSX fallback
    /// lowering pass to skip rules already handled by the zero-cost markup IR
    /// pass, so a migrated rule never reports twice. `None` (the common
    /// template path) runs every rule with no extra work.
    keep_mask: Option<&'rules [bool]>,
}

impl<'a, 'ctx, 'rules> LintVisitor<'a, 'ctx, 'rules> {
    /// Create a new visitor
    #[inline]
    pub fn new(
        ctx: &'ctx mut LintContext<'a>,
        rules: &'rules [Box<dyn Rule>],
        rule_names: &'rules [&'static str],
        run_exit_element_rules: bool,
    ) -> Self {
        Self {
            ctx,
            rules,
            rule_names,
            run_exit_element_rules,
            forget_next_element: false,
            keep_mask: None,
        }
    }

    /// Create a visitor that dispatches only the rules whose `keep_mask` entry
    /// is `true`. The mask must be the same length as `rules`.
    ///
    /// The JSX/TSX lint path uses this for its fallback lowering pass: rules
    /// that already ran over the zero-cost markup IR are masked out here so a
    /// migrated rule produces exactly one diagnostic, not one per backend.
    #[inline]
    pub fn with_rule_filter(
        ctx: &'ctx mut LintContext<'a>,
        rules: &'rules [Box<dyn Rule>],
        rule_names: &'rules [&'static str],
        run_exit_element_rules: bool,
        keep_mask: &'rules [bool],
    ) -> Self {
        Self {
            ctx,
            rules,
            rule_names,
            run_exit_element_rules,
            forget_next_element: false,
            keep_mask: Some(keep_mask),
        }
    }

    /// Whether the rule at `index` is active under the current [`Self::keep_mask`].
    ///
    /// Hot path: with no mask (the common template path) this is a constant
    /// `true`; the JSX fallback pass uses it to skip rules the markup IR pass
    /// already handled. Reads only `keep_mask`, so it does not conflict with the
    /// `&mut self.ctx` the dispatch loops hold.
    #[inline]
    fn rule_active(keep_mask: Option<&[bool]>, index: usize) -> bool {
        match keep_mask {
            Some(mask) => mask[index],
            None => true,
        }
    }

    /// Visit the root node and traverse the AST
    #[inline]
    pub fn visit_root(&mut self, root: &RootNode<'a>) {
        // Pre-scan suppression directives so they can suppress diagnostics
        // produced by `run_on_template` rules (which fire before per-element
        // traversal would register them). Without this pass, directives are
        // registered too late and template-phase rules —
        // `vue/no-dupe-v-else-if`, `vue/no-mutating-props`, etc. — can't be
        // suppressed. (#968, #1196)
        self.prescan_suppression_directives(root);

        // Run template-level checks under one profiling span. Rule dispatch
        // happens for every file, so profiling once around the callback batch is
        // cheaper than creating a span for every individual rule callback.
        let keep_mask = self.keep_mask;
        profile!("patina.rules.run_on_template", {
            for (index, (rule, rule_name)) in self
                .rules
                .iter()
                .zip(self.rule_names.iter().copied())
                .enumerate()
            {
                if !Self::rule_active(keep_mask, index) {
                    continue;
                }
                self.ctx.current_rule = rule_name;
                rule.run_on_template(self.ctx, root);
            }
        });

        // Visit children
        for child in root.children.iter() {
            self.visit_child(child);
        }
    }

    /// Walk the AST and register every suppression-only directive into the
    /// context up-front. Directive kinds that emit diagnostics (Todo, Fixme,
    /// Deprecated) are intentionally NOT processed here; they rely on the
    /// existing ordering that runs during the main traversal.
    fn prescan_suppression_directives(&mut self, root: &RootNode<'a>) {
        let mut forget_next_child = false;
        self.prescan_suppression_in_children(&root.children, &mut forget_next_child);
    }

    fn prescan_suppression_in_children(
        &mut self,
        children: &[TemplateChildNode<'a>],
        forget_next_child: &mut bool,
    ) {
        for (index, node) in children.iter().enumerate() {
            self.prescan_suppression_in_child(children, index, node, forget_next_child);
        }
    }

    fn prescan_suppression_in_child(
        &mut self,
        siblings: &[TemplateChildNode<'a>],
        index: usize,
        node: &TemplateChildNode<'a>,
        forget_next_child: &mut bool,
    ) {
        match node {
            TemplateChildNode::Comment(comment) => {
                if let Some(kind) = comment.directive {
                    let line = self.ctx.offset_to_line(comment.loc.start.offset);
                    match kind {
                        DirectiveKind::Expected => {
                            self.ctx.expect_error_next_line(line);
                        }
                        DirectiveKind::Level => {
                            if let Some(d) = parse_vize_directive(
                                &comment.content,
                                line,
                                comment.loc.start.offset,
                            ) && let Some(severity) = parse_level_severity(&d.payload)
                            {
                                self.ctx.set_severity_override_next_line(line, severity);
                            }
                        }
                        DirectiveKind::IgnoreStart => {
                            self.ctx.push_ignore_region(line);
                        }
                        DirectiveKind::IgnoreEnd => {
                            self.ctx.pop_ignore_region(line);
                        }
                        DirectiveKind::Forget => {
                            *forget_next_child = true;
                        }
                        _ => {}
                    }
                }
            }
            TemplateChildNode::Element(el) => {
                if *forget_next_child {
                    *forget_next_child = false;
                    self.disable_forgotten_element(siblings, index, el);
                }
                self.prescan_suppression_in_children(&el.children, forget_next_child);
            }
            TemplateChildNode::If(if_node) => {
                if *forget_next_child {
                    *forget_next_child = false;
                    self.disable_loc_range(&if_node.loc);
                }
                for branch in if_node.branches.iter() {
                    self.prescan_suppression_in_children(&branch.children, forget_next_child);
                }
            }
            TemplateChildNode::For(for_node) => {
                if *forget_next_child {
                    *forget_next_child = false;
                    self.disable_loc_range(&for_node.loc);
                }
                self.prescan_suppression_in_children(&for_node.children, forget_next_child);
            }
            _ => {}
        }
    }

    fn disable_forgotten_element(
        &mut self,
        siblings: &[TemplateChildNode<'a>],
        index: usize,
        el: &ElementNode<'a>,
    ) {
        self.disable_loc_range(&el.loc);

        if !element_has_directive(el, "if") {
            return;
        }

        for sibling in siblings.iter().skip(index + 1) {
            let TemplateChildNode::Element(branch) = sibling else {
                continue;
            };
            if element_has_directive(branch, "else-if") {
                self.disable_loc_range(&branch.loc);
                continue;
            }
            if element_has_directive(branch, "else") {
                self.disable_loc_range(&branch.loc);
            }
            break;
        }
    }

    fn disable_loc_range(&mut self, loc: &SourceLocation) {
        let start_line = self.ctx.offset_to_line(loc.start.offset);
        let end_line = self.ctx.offset_to_line(loc.end.offset);
        self.ctx.disable_all(start_line, Some(end_line));
    }

    #[inline]
    fn visit_child(&mut self, node: &TemplateChildNode<'a>) {
        match node {
            TemplateChildNode::Element(el) => {
                if self.forget_next_element {
                    self.forget_next_element = false;
                    let start_line = self.ctx.offset_to_line(el.loc.start.offset);
                    let end_line = self.ctx.offset_to_line(el.loc.end.offset);
                    self.ctx.disable_all(start_line, Some(end_line));
                }
                self.visit_element(el);
            }
            TemplateChildNode::Interpolation(interp) => {
                // Coalesce all interpolation rule callbacks into one span for
                // the same reason as template-level checks: callback dispatch is
                // hot and individual rule spans add measurable overhead.
                let keep_mask = self.keep_mask;
                profile!("patina.rules.check_interpolation", {
                    for (index, (rule, rule_name)) in self
                        .rules
                        .iter()
                        .zip(self.rule_names.iter().copied())
                        .enumerate()
                    {
                        if !Self::rule_active(keep_mask, index) {
                            continue;
                        }
                        self.ctx.current_rule = rule_name;
                        rule.check_interpolation(self.ctx, interp);
                    }
                });
            }
            TemplateChildNode::If(if_node) => {
                if self.forget_next_element {
                    self.forget_next_element = false;
                    let start_line = self.ctx.offset_to_line(if_node.loc.start.offset);
                    let end_line = self.ctx.offset_to_line(if_node.loc.end.offset);
                    self.ctx.disable_all(start_line, Some(end_line));
                }
                self.visit_if(if_node);
            }
            TemplateChildNode::For(for_node) => {
                if self.forget_next_element {
                    self.forget_next_element = false;
                    let start_line = self.ctx.offset_to_line(for_node.loc.start.offset);
                    let end_line = self.ctx.offset_to_line(for_node.loc.end.offset);
                    self.ctx.disable_all(start_line, Some(end_line));
                }
                self.visit_for(for_node);
            }
            TemplateChildNode::Comment(comment) => {
                if let Some(kind) = comment.directive {
                    self.process_vize_directive(comment, kind);
                }
            }
            TemplateChildNode::Text(_) => {}
            _ => {}
        }
    }

    /// Process `@vize:` directives on comment nodes.
    fn process_vize_directive(&mut self, comment: &CommentNode, kind: DirectiveKind) {
        let line = self.ctx.offset_to_line(comment.loc.start.offset);
        let loc = &comment.loc;

        match kind {
            DirectiveKind::Todo => {
                // Parse the payload from the comment content
                if let Some(d) = parse_vize_directive(&comment.content, line, loc.start.offset) {
                    let msg = if d.payload.is_empty() {
                        CompactString::from("TODO")
                    } else {
                        cstr!("TODO: {}", d.payload)
                    };
                    self.ctx.current_rule = "vize/todo";
                    self.ctx.warn(msg, loc);
                }
            }
            DirectiveKind::Fixme => {
                if let Some(d) = parse_vize_directive(&comment.content, line, loc.start.offset) {
                    let msg = if d.payload.is_empty() {
                        CompactString::from("FIXME")
                    } else {
                        cstr!("FIXME: {}", d.payload)
                    };
                    self.ctx.current_rule = "vize/fixme";
                    self.ctx.error(msg, loc);
                }
            }
            DirectiveKind::Expected => {
                self.ctx.expect_error_next_line(line);
            }
            DirectiveKind::IgnoreStart => {
                self.ctx.push_ignore_region(line);
            }
            DirectiveKind::IgnoreEnd => {
                self.ctx.pop_ignore_region(line);
            }
            DirectiveKind::Level => {
                if let Some(d) = parse_vize_directive(&comment.content, line, loc.start.offset)
                    && let Some(severity) = parse_level_severity(&d.payload)
                {
                    self.ctx.set_severity_override_next_line(line, severity);
                }
            }
            DirectiveKind::Deprecated => {
                if let Some(d) = parse_vize_directive(&comment.content, line, loc.start.offset) {
                    let msg = if d.payload.is_empty() {
                        CompactString::from("Deprecated")
                    } else {
                        cstr!("Deprecated: {}", d.payload)
                    };
                    self.ctx.current_rule = "vize/deprecated";
                    self.ctx.warn(msg, loc);
                }
            }
            DirectiveKind::Forget => {
                if let Some(d) = parse_vize_directive(&comment.content, line, loc.start.offset) {
                    if d.payload.is_empty() {
                        self.ctx.current_rule = "vize/forget";
                        self.ctx
                            .warn(CompactString::from("@vize:forget requires a reason"), loc);
                    }
                    self.forget_next_element = true;
                }
            }
            // Docs, DevOnly, Unknown: no lint action needed
            _ => {}
        }
    }

    fn visit_element(&mut self, el: &ElementNode<'a>) {
        // Check for v-for and v-if directives using iterators (no allocation)
        let has_v_for = el
            .props
            .iter()
            .any(|p| matches!(p, PropNode::Directive(d) if d.name.as_str() == "for"));
        let has_v_if = el
            .props
            .iter()
            .any(|p| matches!(p, PropNode::Directive(d) if d.name.as_str() == "if" || d.name.as_str() == "else-if"));

        // Extract v-for variables (only allocates if v-for exists)
        let v_for_vars = if has_v_for {
            self.extract_v_for_vars(el)
        } else {
            Vec::new()
        };

        // Build element context with CompactString tag (efficient for small strings)
        let elem_ctx = ElementContext {
            tag: CompactString::from(el.tag.as_str()),
            has_v_for,
            has_v_if,
            v_for_vars,
        };

        self.ctx.push_element(elem_ctx);

        // Enter element - run rules. Element/directive/exit/branch callbacks
        // follow the same coalesced-span pattern as root/interpolation checks:
        // one guard around the rule batch, not one guard per rule.
        let keep_mask = self.keep_mask;
        profile!("patina.rules.enter_element", {
            for (index, (rule, rule_name)) in self
                .rules
                .iter()
                .zip(self.rule_names.iter().copied())
                .enumerate()
            {
                if !Self::rule_active(keep_mask, index) {
                    continue;
                }
                self.ctx.current_rule = rule_name;
                rule.enter_element(self.ctx, el);
            }
        });

        // Check directives
        for prop in el.props.iter() {
            if let PropNode::Directive(dir) = prop {
                profile!("patina.rules.check_directive", {
                    for (index, (rule, rule_name)) in self
                        .rules
                        .iter()
                        .zip(self.rule_names.iter().copied())
                        .enumerate()
                    {
                        if !Self::rule_active(keep_mask, index) {
                            continue;
                        }
                        self.ctx.current_rule = rule_name;
                        rule.check_directive(self.ctx, el, dir);
                    }
                });
            }
        }

        // Visit children
        for child in el.children.iter() {
            self.visit_child(child);
        }

        if self.run_exit_element_rules {
            // Exit element - run rules
            profile!("patina.rules.exit_element", {
                for (index, (rule, rule_name)) in self
                    .rules
                    .iter()
                    .zip(self.rule_names.iter().copied())
                    .enumerate()
                {
                    if !Self::rule_active(keep_mask, index) {
                        continue;
                    }
                    self.ctx.current_rule = rule_name;
                    rule.exit_element(self.ctx, el);
                }
            });
        }

        self.ctx.pop_element();
    }

    #[inline]
    fn visit_if(&mut self, if_node: &vize_relief::ast::IfNode<'a>) {
        // Run if checks
        let keep_mask = self.keep_mask;
        profile!("patina.rules.check_if", {
            for (index, (rule, rule_name)) in self
                .rules
                .iter()
                .zip(self.rule_names.iter().copied())
                .enumerate()
            {
                if !Self::rule_active(keep_mask, index) {
                    continue;
                }
                self.ctx.current_rule = rule_name;
                rule.check_if(self.ctx, if_node);
            }
        });

        // Visit branches
        for branch in if_node.branches.iter() {
            for child in branch.children.iter() {
                self.visit_child(child);
            }
        }
    }

    #[inline]
    fn visit_for(&mut self, for_node: &vize_relief::ast::ForNode<'a>) {
        // Run for checks
        let keep_mask = self.keep_mask;
        profile!("patina.rules.check_for", {
            for (index, (rule, rule_name)) in self
                .rules
                .iter()
                .zip(self.rule_names.iter().copied())
                .enumerate()
            {
                if !Self::rule_active(keep_mask, index) {
                    continue;
                }
                self.ctx.current_rule = rule_name;
                rule.check_for(self.ctx, for_node);
            }
        });

        // Visit children
        for child in for_node.children.iter() {
            self.visit_child(child);
        }
    }

    /// Extract variable names from v-for directive on an element
    #[inline]
    fn extract_v_for_vars(&self, el: &ElementNode<'a>) -> Vec<CompactString> {
        for prop in el.props.iter() {
            if let PropNode::Directive(dir) = prop
                && dir.name.as_str() == "for"
                && let Some(exp) = &dir.exp
            {
                return parse_v_for_variables(exp);
            }
        }
        Vec::new()
    }
}

fn element_has_directive(el: &ElementNode, name: &str) -> bool {
    el.props
        .iter()
        .any(|p| matches!(p, PropNode::Directive(d) if d.name.as_str() == name))
}

/// Parse v-for expression to extract variable names.
///
/// Uses CompactString for efficient small string storage.
///
/// Handles formats like:
/// - `item in items`
/// - `(item, index) in items`
/// - `(value, key, index) in object`
#[inline]
pub fn parse_v_for_variables(exp: &ExpressionNode) -> Vec<CompactString> {
    let content = match exp {
        ExpressionNode::Simple(s) => s.content.as_str(),
        ExpressionNode::Compound(_) => return Vec::new(),
    };

    // Split on " in " or " of " - use byte search for speed
    let bytes = content.as_bytes();
    let (alias_part, _) = if let Some(idx) = find_pattern(bytes, b" in ") {
        (&content[..idx], &content[idx + 4..])
    } else if let Some(idx) = find_pattern(bytes, b" of ") {
        (&content[..idx], &content[idx + 4..])
    } else {
        return Vec::new();
    };

    let alias_str = alias_part.trim();

    // Handle destructuring: (item, index), { id, name }, or [first, second]
    let is_tuple = alias_str.starts_with('(') && alias_str.ends_with(')');
    let is_object = alias_str.starts_with('{') && alias_str.ends_with('}');
    let is_array = alias_str.starts_with('[') && alias_str.ends_with(']');

    if is_tuple || is_object || is_array {
        let inner = &alias_str[1..alias_str.len() - 1];
        // Pre-allocate with estimated capacity
        let mut vars = Vec::with_capacity(3);
        for s in inner.split(',') {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Handle object shorthand: { id } -> id, { id: itemId } -> itemId
            if is_object {
                if let Some(colon_idx) = trimmed.find(':') {
                    // { id: itemId } -> itemId
                    let value_part = trimmed[colon_idx + 1..].trim();
                    if !value_part.is_empty() {
                        vars.push(CompactString::from(value_part));
                    }
                } else {
                    // { id } -> id (shorthand)
                    vars.push(CompactString::from(trimmed));
                }
            } else {
                vars.push(CompactString::from(trimmed));
            }
        }
        vars
    } else {
        // Single variable - avoid allocation if possible
        vec![CompactString::from(alias_str)]
    }
}

/// Fast byte pattern search
#[inline]
fn find_pattern(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }

    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::{CompactString, ExpressionNode, parse_v_for_variables};
    use vize_carton::Bump;
    use vize_relief::ast::SimpleExpressionNode;

    fn make_simple_exp<'a>(allocator: &'a Bump, content: &str) -> ExpressionNode<'a> {
        ExpressionNode::Simple(vize_carton::Box::new_in(
            SimpleExpressionNode::new(
                vize_carton::String::from(content),
                false,
                vize_relief::ast::SourceLocation::STUB,
            ),
            allocator,
        ))
    }

    #[test]
    fn test_parse_v_for_simple() {
        let allocator = Bump::new();
        let exp = make_simple_exp(&allocator, "item in items");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(vars, vec![CompactString::from("item")]);
    }

    #[test]
    fn test_parse_v_for_with_index() {
        let allocator = Bump::new();
        let exp = make_simple_exp(&allocator, "(item, index) in items");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(
            vars,
            vec![CompactString::from("item"), CompactString::from("index")]
        );
    }

    #[test]
    fn test_parse_v_for_object() {
        let allocator = Bump::new();
        let exp = make_simple_exp(&allocator, "(value, key, index) in object");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(
            vars,
            vec![
                CompactString::from("value"),
                CompactString::from("key"),
                CompactString::from("index"),
            ]
        );
    }

    #[test]
    fn test_parse_v_for_object_destructuring() {
        let allocator = Bump::new();
        let exp = make_simple_exp(&allocator, "{ id } in items");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(vars, vec![CompactString::from("id")]);
    }

    #[test]
    fn test_parse_v_for_object_destructuring_multiple() {
        let allocator = Bump::new();
        let exp = make_simple_exp(&allocator, "{ id, name } in items");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(
            vars,
            vec![CompactString::from("id"), CompactString::from("name")]
        );
    }

    #[test]
    fn test_parse_v_for_object_destructuring_with_rename() {
        let allocator = Bump::new();
        let exp = make_simple_exp(&allocator, "{ id: itemId, name: itemName } in items");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(
            vars,
            vec![
                CompactString::from("itemId"),
                CompactString::from("itemName")
            ]
        );
    }

    #[test]
    fn test_parse_v_for_array_destructuring() {
        let allocator = Bump::new();
        let exp = make_simple_exp(&allocator, "[first, second] in items");
        let vars = parse_v_for_variables(&exp);
        assert_eq!(
            vars,
            vec![CompactString::from("first"), CompactString::from("second")]
        );
    }
}

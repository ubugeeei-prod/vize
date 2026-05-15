//! Expression generation helpers.
//!
//! Internal utilities for context-aware identifier prefixing,
//! line comment conversion, and slot parameter stripping.

use crate::options::BindingType;
use vize_croquis::builtins::is_global_allowed;

use super::super::context::CodegenContext;
use vize_carton::String;
use vize_carton::ToCompactString;

/// Prefix identifiers in expression with appropriate prefix based on binding metadata.
/// This is a context-aware version that uses `$setup.` for setup bindings in function mode.
pub(crate) fn prefix_identifiers_with_context(content: &str, ctx: &CodegenContext) -> String {
    use oxc_allocator::Allocator as OxcAllocator;
    use oxc_ast_visit::walk::{
        walk_assignment_expression, walk_object_property, walk_update_expression,
    };
    use oxc_ast_visit::Visit;
    use oxc_parser::Parser;
    use oxc_span::SourceType;
    use vize_carton::FxHashSet;

    // Visitor to collect identifiers and rewrite them with appropriate prefixes / .value
    struct IdentifierVisitor<'a, 'b> {
        rewrites: &'a mut Vec<(usize, usize, String)>,
        local_vars: &'a mut FxHashSet<String>,
        assignment_targets: &'a mut FxHashSet<usize>,
        ctx: &'b CodegenContext,
        offset: u32,
    }

    impl<'a, 'b> Visit<'_> for IdentifierVisitor<'a, 'b> {
        fn visit_identifier_reference(&mut self, ident: &oxc_ast::ast::IdentifierReference<'_>) {
            let name = ident.name.as_str();

            // Skip if local variable
            if self.local_vars.contains(name) {
                return;
            }

            // Skip globals
            if is_global_allowed(name) {
                return;
            }

            // Skip slot params
            if self.ctx.is_slot_param(name) {
                return;
            }

            let is_assignment_target = self
                .assignment_targets
                .contains(&(ident.span.start as usize));

            // Determine prefix based on binding metadata
            let mut binding_type: Option<BindingType> = None;
            let prefix = if let Some(ref metadata) = self.ctx.options.binding_metadata {
                if let Some(binding) = metadata.bindings.get(name) {
                    binding_type = Some(*binding);
                    match binding {
                        BindingType::Props | BindingType::PropsAliased => "$props.",
                        _ => {
                            if self.ctx.options.inline {
                                ""
                            } else {
                                "$setup."
                            }
                        }
                    }
                } else {
                    "_ctx."
                }
            } else {
                "_ctx."
            };

            if is_assignment_target {
                let needs_value = matches!(
                    binding_type,
                    Some(
                        BindingType::SetupLet | BindingType::SetupMaybeRef | BindingType::SetupRef
                    )
                );
                let replacement = if needs_value {
                    let mut out = String::with_capacity(prefix.len() + name.len() + 6);
                    out.push_str(prefix);
                    out.push_str(name);
                    out.push_str(".value");
                    out
                } else if !prefix.is_empty() {
                    let mut out = String::with_capacity(prefix.len() + name.len());
                    out.push_str(prefix);
                    out.push_str(name);
                    out
                } else {
                    name.to_compact_string()
                };
                if replacement != name {
                    let start = (ident.span.start - self.offset) as usize;
                    let end = (ident.span.end - self.offset) as usize;
                    self.rewrites.push((start, end, replacement));
                }
                return;
            }

            if !prefix.is_empty() {
                let start = (ident.span.start - self.offset) as usize;
                let end = (ident.span.end - self.offset) as usize;
                let mut replacement = String::with_capacity(prefix.len() + name.len());
                replacement.push_str(prefix);
                replacement.push_str(name);
                self.rewrites.push((start, end, replacement));
            }
        }

        fn visit_assignment_expression(&mut self, expr: &oxc_ast::ast::AssignmentExpression<'_>) {
            self.collect_assignment_targets(&expr.left);
            walk_assignment_expression(self, expr);
        }

        fn visit_update_expression(&mut self, expr: &oxc_ast::ast::UpdateExpression<'_>) {
            self.collect_simple_assignment_targets(&expr.argument);
            walk_update_expression(self, expr);
        }

        fn visit_object_property(&mut self, prop: &oxc_ast::ast::ObjectProperty<'_>) {
            if prop.shorthand {
                if let oxc_ast::ast::PropertyKey::StaticIdentifier(ident) = &prop.key {
                    let name = ident.name.as_str();

                    // Skip if local variable, global, or slot param
                    if self.local_vars.contains(name)
                        || is_global_allowed(name)
                        || self.ctx.is_slot_param(name)
                    {
                        return;
                    }

                    let mut is_ref = false;
                    let mut needs_unref = false;
                    let prefix = if let Some(ref metadata) = self.ctx.options.binding_metadata {
                        if let Some(binding_type) = metadata.bindings.get(name) {
                            is_ref = self.ctx.options.inline
                                && matches!(binding_type, BindingType::SetupRef);
                            needs_unref = self.ctx.options.inline
                                && matches!(
                                    binding_type,
                                    BindingType::SetupLet | BindingType::SetupMaybeRef
                                );
                            match binding_type {
                                BindingType::Props | BindingType::PropsAliased => "$props.",
                                _ => {
                                    if self.ctx.options.inline {
                                        ""
                                    } else {
                                        "$setup."
                                    }
                                }
                            }
                        } else {
                            "_ctx."
                        }
                    } else {
                        "_ctx."
                    };

                    if !prefix.is_empty() || is_ref || needs_unref {
                        let start = (prop.span.start - self.offset) as usize;
                        let end = (prop.span.end - self.offset) as usize;
                        let (value_prefix, value_suffix) = if needs_unref {
                            ("_unref(", ")")
                        } else if is_ref {
                            ("", ".value")
                        } else {
                            ("", "")
                        };
                        let mut replacement = String::with_capacity(
                            name.len()
                                + 2
                                + value_prefix.len()
                                + prefix.len()
                                + name.len()
                                + value_suffix.len(),
                        );
                        replacement.push_str(name);
                        replacement.push_str(": ");
                        replacement.push_str(value_prefix);
                        if !needs_unref {
                            replacement.push_str(prefix);
                        }
                        replacement.push_str(name);
                        replacement.push_str(value_suffix);
                        self.rewrites.push((start, end, replacement));
                        return;
                    }
                }
            }

            walk_object_property(self, prop);
        }

        fn visit_variable_declarator(&mut self, declarator: &oxc_ast::ast::VariableDeclarator<'_>) {
            // Add local var names to skip list
            if let oxc_ast::ast::BindingPattern::BindingIdentifier(ident) = &declarator.id {
                self.local_vars.insert(ident.name.to_compact_string());
            }
            // Visit init expression
            if let Some(init) = &declarator.init {
                self.visit_expression(init);
            }
        }

        fn visit_arrow_function_expression(
            &mut self,
            arrow: &oxc_ast::ast::ArrowFunctionExpression<'_>,
        ) {
            // Add arrow function params to local vars
            for param in &arrow.params.items {
                if let oxc_ast::ast::BindingPattern::BindingIdentifier(ident) = &param.pattern {
                    self.local_vars.insert(ident.name.to_compact_string());
                }
            }
            // Visit body
            self.visit_function_body(&arrow.body);
        }
    }

    impl<'a, 'b> IdentifierVisitor<'a, 'b> {
        fn collect_assignment_targets(&mut self, target: &oxc_ast::ast::AssignmentTarget<'_>) {
            use oxc_ast::ast::{AssignmentTarget, AssignmentTargetProperty};

            match target {
                AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                    self.assignment_targets.insert(ident.span.start as usize);
                }
                AssignmentTarget::ObjectAssignmentTarget(obj) => {
                    for prop in &obj.properties {
                        match prop {
                            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(
                                prop_ident,
                            ) => {
                                self.assignment_targets
                                    .insert(prop_ident.binding.span.start as usize);
                            }
                            AssignmentTargetProperty::AssignmentTargetPropertyProperty(
                                prop_prop,
                            ) => {
                                self.collect_assignment_targets_maybe_default(&prop_prop.binding);
                            }
                        }
                    }
                    if let Some(rest) = &obj.rest {
                        self.collect_assignment_targets(&rest.target);
                    }
                }
                AssignmentTarget::ArrayAssignmentTarget(arr) => {
                    for elem in arr.elements.iter().flatten() {
                        self.collect_assignment_targets_maybe_default(elem);
                    }
                    if let Some(rest) = &arr.rest {
                        self.collect_assignment_targets(&rest.target);
                    }
                }
                _ => {}
            }
        }

        fn collect_assignment_targets_maybe_default(
            &mut self,
            target: &oxc_ast::ast::AssignmentTargetMaybeDefault<'_>,
        ) {
            use oxc_ast::ast::{AssignmentTargetMaybeDefault, AssignmentTargetProperty};

            match target {
                AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(def) => {
                    self.collect_assignment_targets(&def.binding);
                }
                AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(ident) => {
                    self.assignment_targets.insert(ident.span.start as usize);
                }
                AssignmentTargetMaybeDefault::ObjectAssignmentTarget(obj) => {
                    for prop in &obj.properties {
                        match prop {
                            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(
                                prop_ident,
                            ) => {
                                self.assignment_targets
                                    .insert(prop_ident.binding.span.start as usize);
                            }
                            AssignmentTargetProperty::AssignmentTargetPropertyProperty(
                                prop_prop,
                            ) => {
                                self.collect_assignment_targets_maybe_default(&prop_prop.binding);
                            }
                        }
                    }
                    if let Some(rest) = &obj.rest {
                        self.collect_assignment_targets(&rest.target);
                    }
                }
                AssignmentTargetMaybeDefault::ArrayAssignmentTarget(arr) => {
                    for elem in arr.elements.iter().flatten() {
                        self.collect_assignment_targets_maybe_default(elem);
                    }
                    if let Some(rest) = &arr.rest {
                        self.collect_assignment_targets(&rest.target);
                    }
                }
                _ => {}
            }
        }

        fn collect_simple_assignment_targets(
            &mut self,
            target: &oxc_ast::ast::SimpleAssignmentTarget<'_>,
        ) {
            use oxc_ast::ast::SimpleAssignmentTarget;

            if let SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = target {
                self.assignment_targets.insert(ident.span.start as usize);
            }
        }
    }

    /// Apply collected rewrites to content and return the result
    fn apply_rewrites(content: &str, mut rewrites: Vec<(usize, usize, String)>) -> String {
        if rewrites.is_empty() {
            return content.to_compact_string();
        }
        rewrites.sort_by_key(|rewrite| std::cmp::Reverse(rewrite.0));
        let mut result = content.to_compact_string();
        for (start, end, replacement) in rewrites {
            if start < result.len() && end <= result.len() {
                result.replace_range(start..end, &replacement);
            }
        }
        result
    }

    let allocator = OxcAllocator::default();
    let source_type = SourceType::default().with_module(true);

    // First try: wrap in parentheses to parse as a single expression
    let mut wrapped = String::with_capacity(content.len() + 2);
    wrapped.push('(');
    wrapped.push_str(content);
    wrapped.push(')');
    let parser = Parser::new(&allocator, &wrapped, source_type);
    let parse_result = parser.parse_expression();

    match parse_result {
        Ok(expr) => {
            let mut rewrites: Vec<(usize, usize, String)> = Vec::new();
            let mut local_vars: FxHashSet<String> = FxHashSet::default();
            let mut assignment_targets: FxHashSet<usize> = FxHashSet::default();

            let mut visitor = IdentifierVisitor {
                rewrites: &mut rewrites,
                local_vars: &mut local_vars,
                assignment_targets: &mut assignment_targets,
                ctx,
                offset: 1, // Account for the '(' we added
            };
            visitor.visit_expression(&expr);

            apply_rewrites(content, rewrites)
        }
        Err(_) => {
            // Expression parsing failed -- try parsing as a program
            let allocator2 = OxcAllocator::default();
            let parser2 = Parser::new(&allocator2, content, source_type);
            let parse_result2 = parser2.parse();
            if parse_result2.errors.is_empty() {
                let mut rewrites: Vec<(usize, usize, String)> = Vec::new();
                let mut local_vars: FxHashSet<String> = FxHashSet::default();
                let mut assignment_targets: FxHashSet<usize> = FxHashSet::default();

                let mut visitor = IdentifierVisitor {
                    rewrites: &mut rewrites,
                    local_vars: &mut local_vars,
                    assignment_targets: &mut assignment_targets,
                    ctx,
                    offset: 0,
                };
                visitor.visit_program(&parse_result2.program);

                apply_rewrites(content, rewrites)
            } else {
                content.to_compact_string()
            }
        }
    }
}

/// Convert `// ...` line comments to `/* ... */` block comments.
/// Handles strings (single/double/template) to avoid modifying `//` inside string literals.
pub(crate) fn convert_line_comments_to_block(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let bytes = content.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        match bytes[i] {
            // Skip string literals
            b'\'' | b'"' | b'`' => {
                let quote = bytes[i];
                result.push(quote as char);
                i += 1;
                while i < len {
                    if bytes[i] == b'\\' && i + 1 < len {
                        result.push(bytes[i] as char);
                        result.push(bytes[i + 1] as char);
                        i += 2;
                    } else if bytes[i] == quote {
                        result.push(quote as char);
                        i += 1;
                        break;
                    } else {
                        result.push(bytes[i] as char);
                        i += 1;
                    }
                }
            }
            // Check for //
            b'/' if i + 1 < len && bytes[i + 1] == b'/' => {
                let comment_start = i + 2;
                let mut comment_end = comment_start;
                while comment_end < len && bytes[comment_end] != b'\n' {
                    comment_end += 1;
                }
                let comment_text = &content[comment_start..comment_end].trim_end();
                result.push_str("/* ");
                result.push_str(comment_text);
                result.push_str(" */");
                i = comment_end;
                if i < len && bytes[i] == b'\n' {
                    result.push('\n');
                    i += 1;
                }
            }
            // Skip existing block comments
            b'/' if i + 1 < len && bytes[i + 1] == b'*' => {
                result.push('/');
                result.push('*');
                i += 2;
                while i + 1 < len {
                    if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                        result.push('*');
                        result.push('/');
                        i += 2;
                        break;
                    }
                    result.push(bytes[i] as char);
                    i += 1;
                }
            }
            _ => {
                result.push(bytes[i] as char);
                i += 1;
            }
        }
    }

    result
}

/// Strip `_ctx.` prefix for identifiers that are slot/v-for parameters.
pub(crate) fn strip_ctx_for_slot_params(ctx: &CodegenContext, content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let bytes = content.as_bytes();
    let prefix = b"_ctx.";
    let mut i = 0;

    while i < bytes.len() {
        if i + prefix.len() <= bytes.len() && &bytes[i..i + prefix.len()] == prefix {
            let start = i + prefix.len();
            let mut end = start;
            while end < bytes.len()
                && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_' || bytes[end] == b'$')
            {
                end += 1;
            }
            let ident = &content[start..end];
            if !ident.is_empty() && ctx.is_slot_param(ident) {
                result.push_str(ident);
                i = end;
            } else {
                result.push_str("_ctx.");
                i = start;
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    result
}

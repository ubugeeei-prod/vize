//! Template reference finding.
//!
//! Finds references to symbols in template expressions,
//! including mustache interpolations and directive bindings.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use tower_lsp::lsp_types::{Location, Position, Range};

use super::{IdeContext, ReferencesService};

impl ReferencesService {
    /// Find references to a symbol in the template block.
    pub(super) fn find_references_in_template(ctx: &IdeContext, word: &str) -> Vec<Location> {
        let mut locations = Vec::new();

        let options = vize_atelier_sfc::SfcParseOptions::default();
        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) else {
            return locations;
        };

        let Some(ref template) = descriptor.template else {
            return locations;
        };

        let template_content = template.content.as_ref();
        let template_start_line = template.loc.start_line as u32;

        // Find all occurrences of the word in template
        // This includes:
        // - Interpolations: {{ word }}
        // - Directive expressions: v-if="word", :prop="word", @event="word"

        // Parse template to find expressions
        let allocator = vize_carton::Bump::new();
        let (ast, _) = vize_armature::parse(&allocator, template_content);

        // Extract expression locations from the AST
        let expressions = Self::extract_template_expressions(&ast);

        for (expr_text, expr_offset) in expressions {
            // Find word occurrences within the expression
            let word_positions = Self::find_word_occurrences(&expr_text, word);

            for word_offset_in_expr in word_positions {
                let absolute_offset = expr_offset + word_offset_in_expr;
                let (line, character) = Self::offset_to_position(template_content, absolute_offset);

                locations.push(Location {
                    uri: ctx.uri.clone(),
                    range: Range {
                        start: Position {
                            line: template_start_line + line - 1,
                            character,
                        },
                        end: Position {
                            line: template_start_line + line - 1,
                            character: character + word.len() as u32,
                        },
                    },
                });
            }
        }

        // Also do a simple text search for the word in template
        // This catches cases the AST might miss
        let simple_refs = Self::find_simple_references_in_content(
            template_content,
            word,
            template_start_line - 1,
        );

        for loc in simple_refs {
            // Check if this location is already in our list
            let is_duplicate = locations.iter().any(|existing| {
                existing.range.start.line == loc.range.start.line
                    && existing.range.start.character == loc.range.start.character
            });
            if !is_duplicate {
                locations.push(Location {
                    uri: ctx.uri.clone(),
                    range: loc.range,
                });
            }
        }

        locations
    }

    /// Extract expressions from template AST.
    fn extract_template_expressions<'a>(ast: &vize_armature::RootNode<'a>) -> Vec<(String, usize)> {
        let mut expressions = Vec::new();
        Self::visit_children_for_expressions(&ast.children, &mut expressions);
        expressions
    }

    /// Visit children to extract expressions.
    fn visit_children_for_expressions<'a>(
        children: &[vize_relief::ast::TemplateChildNode<'a>],
        expressions: &mut Vec<(String, usize)>,
    ) {
        use vize_relief::ast::{PropNode, TemplateChildNode};

        for child in children {
            match child {
                TemplateChildNode::Element(el) => {
                    // Check directives
                    for prop in &el.props {
                        if let PropNode::Directive(dir) = prop {
                            if let Some(ref exp) = dir.exp {
                                if let Some((text, offset)) = Self::get_expression_info(exp) {
                                    expressions.push((text, offset));
                                }
                            }
                        }
                    }
                    // Visit children
                    Self::visit_children_for_expressions(&el.children, expressions);
                }
                TemplateChildNode::Interpolation(interp) => {
                    if let Some((text, offset)) = Self::get_expression_info(&interp.content) {
                        expressions.push((text, offset));
                    }
                }
                TemplateChildNode::If(if_node) => {
                    for branch in &if_node.branches {
                        if let Some(ref cond) = branch.condition {
                            if let Some((text, offset)) = Self::get_expression_info(cond) {
                                expressions.push((text, offset));
                            }
                        }
                        Self::visit_children_for_expressions(&branch.children, expressions);
                    }
                }
                TemplateChildNode::For(for_node) => {
                    if let Some((text, offset)) = Self::get_expression_info(&for_node.source) {
                        expressions.push((text, offset));
                    }
                    Self::visit_children_for_expressions(&for_node.children, expressions);
                }
                TemplateChildNode::IfBranch(branch) => {
                    if let Some(ref cond) = branch.condition {
                        if let Some((text, offset)) = Self::get_expression_info(cond) {
                            expressions.push((text, offset));
                        }
                    }
                    Self::visit_children_for_expressions(&branch.children, expressions);
                }
                _ => {}
            }
        }
    }

    /// Get expression text and offset from ExpressionNode.
    fn get_expression_info(expr: &vize_relief::ast::ExpressionNode) -> Option<(String, usize)> {
        use vize_relief::ast::ExpressionNode;

        match expr {
            ExpressionNode::Simple(simple) => {
                if simple.content.is_empty() {
                    None
                } else {
                    Some((simple.content.to_string(), simple.loc.start.offset as usize))
                }
            }
            ExpressionNode::Compound(compound) => {
                // For compound expressions, we can't easily get the text
                // Return the location but mark as compound
                Some(("<compound>".to_string(), compound.loc.start.offset as usize))
            }
        }
    }

    /// Find all occurrences of a word in a string.
    pub(super) fn find_word_occurrences(text: &str, word: &str) -> Vec<usize> {
        let mut positions = Vec::new();
        let mut start = 0;

        while let Some(pos) = text[start..].find(word) {
            let absolute_pos = start + pos;

            // Check word boundaries
            let before_ok =
                absolute_pos == 0 || !Self::is_identifier_char(text.as_bytes()[absolute_pos - 1]);
            let after_ok = absolute_pos + word.len() >= text.len()
                || !Self::is_identifier_char(text.as_bytes()[absolute_pos + word.len()]);

            if before_ok && after_ok {
                positions.push(absolute_pos);
            }

            start = absolute_pos + 1;
        }

        positions
    }

    /// Find simple text references in content.
    fn find_simple_references_in_content(
        content: &str,
        word: &str,
        base_line: u32,
    ) -> Vec<Location> {
        let mut locations = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            let line_positions = Self::find_word_occurrences(line, word);

            for pos in line_positions {
                // Check if this is in a binding context
                // (inside {{ }}, after v-*, after :, after @, etc.)
                if Self::is_in_binding_context(line, pos) {
                    locations.push(Location {
                        uri: tower_lsp::lsp_types::Url::parse("file:///dummy").unwrap(),
                        range: Range {
                            start: Position {
                                line: base_line + line_idx as u32,
                                character: pos as u32,
                            },
                            end: Position {
                                line: base_line + line_idx as u32,
                                character: pos as u32 + word.len() as u32,
                            },
                        },
                    });
                }
            }
        }

        locations
    }

    /// Check if a position is in a binding context.
    pub(super) fn is_in_binding_context(line: &str, pos: usize) -> bool {
        let before = &line[..pos];

        // Check for interpolation: {{
        if before.contains("{{") {
            let last_open = before.rfind("{{").unwrap();
            let close_before = before[last_open..].contains("}}");
            if !close_before {
                return true;
            }
        }

        // Check for directive expressions: ="
        if let Some(eq_pos) = before.rfind('=') {
            let after_eq = &before[eq_pos..];
            if after_eq.contains('"') && !after_eq[after_eq.find('"').unwrap() + 1..].contains('"')
            {
                return true;
            }
        }

        false
    }
}

//! Reserve authored names, including decoded HTML entities and JS escapes.

use oxc_ast::ast::{BindingIdentifier, IdentifierReference};
use oxc_ast_visit::Visit;
use vize_armature::patterns::{MatchPattern, PatternKind, parse_match_pattern};
use vize_s0::{String, ensure_sufficient_stack};

use super::syntax::expression_source;
use crate::{PropNode, RootNode, TemplateChildNode, TransformContext};

struct Names(String);

impl<'a> Visit<'a> for Names {
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        self.add(identifier.name.as_str());
    }

    fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
        self.add(identifier.name.as_str());
    }
}

impl Names {
    fn add(&mut self, name: &str) {
        self.0.push(' ');
        self.0.push_str(name);
    }

    fn expression(&mut self, source: &str) {
        self.add(source);
        let allocator = oxc_allocator::Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, source, oxc_span::SourceType::ts()).parse();
        self.visit_program(&parsed.program);
    }

    fn pattern(&mut self, pattern: &MatchPattern) {
        match &pattern.kind {
            PatternKind::Value(value) => self.expression(&value.text),
            PatternKind::As { pattern, .. } => self.pattern(pattern),
            PatternKind::Or(patterns)
            | PatternKind::Array {
                elements: patterns, ..
            } => {
                for pattern in patterns {
                    self.pattern(pattern);
                }
            }
            PatternKind::Object { properties, .. } => {
                for property in properties {
                    self.pattern(&property.pattern);
                }
            }
            _ => {}
        }
    }

    fn children(&mut self, children: &[TemplateChildNode<'_>], source: &str) {
        for child in children {
            match child {
                TemplateChildNode::Element(el) => {
                    for prop in &el.props {
                        let PropNode::Directive(dir) = prop else {
                            continue;
                        };
                        for exp in dir.exp.iter().chain(dir.arg.iter()) {
                            let text = expression_source(exp, source);
                            self.expression(&text);
                            if matches!(dir.name, "when" | "case")
                                && let Ok(arm) = parse_match_pattern(&text)
                            {
                                self.pattern(&arm.pattern);
                                if let Some(guard) = arm.guard {
                                    self.expression(&guard.text);
                                }
                            }
                        }
                    }
                    ensure_sufficient_stack(|| self.children(&el.children, source));
                }
                TemplateChildNode::Interpolation(node) => {
                    self.expression(&expression_source(&node.content, source))
                }
                _ => {}
            }
        }
    }
}

pub(super) fn unique_prefix(ctx: &TransformContext<'_>, root: &RootNode<'_>) -> String {
    let mut names = Names(String::from(root.source));
    if let Some(metadata) = &ctx.options.binding_metadata {
        for name in metadata.bindings.keys() {
            names.add(name);
        }
    }
    names.children(&root.children, root.source);
    let mut prefix = String::from("__vize_match");
    while names.0.contains(prefix.as_str()) {
        prefix.push('_');
    }
    prefix
}

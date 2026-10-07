//! Conservative ownership facts from the template AST already parsed for linting.

use lightningcss::properties::custom::{Token, TokenOrValue};
use lightningcss::selector::{Component, PseudoClass, Selector};
use vize_l0::{FxHashSet, Namespace, String, ToCompactString, is_html_tag};
use vize_relief::{ElementType, PropNode, RootNode, TemplateChildNode};

#[derive(Default)]
pub(super) struct TemplateTargets {
    classes: FxHashSet<String>,
    ids: FxHashSet<String>,
    tags: FxHashSet<String>,
    unknown_classes: bool,
    unknown_ids: bool,
    unknown_tags: bool,
}

impl TemplateTargets {
    pub(super) fn from_root(root: &RootNode<'_>) -> Self {
        let mut targets = Self::default();
        // A single root can receive a caller's class/id through fallthrough attrs.
        // Require two unconditional native roots before proving their absence.
        let fragment = root
            .children
            .iter()
            .filter(|child| {
                matches!(child, TemplateChildNode::Element(element)
                if element.tag_type == ElementType::Element
                    && element.ns == Namespace::Html && is_html_tag(element.tag)
                    && !element.props.iter().any(|prop| matches!(prop,
                        PropNode::Directive(directive)
                            if matches!(directive.name, "if" | "else" | "else-if" | "for"))))
            })
            .count()
            >= 2;
        targets.unknown_classes = !fragment;
        targets.unknown_ids = !fragment;
        let mut pending: Vec<_> = root.children.iter().collect();
        while let Some(child) = pending.pop() {
            match child {
                TemplateChildNode::Element(element) => {
                    pending.extend(element.children.iter());
                    match element.tag_type {
                        ElementType::Element
                            if element.ns == Namespace::Html && is_html_tag(element.tag) =>
                        {
                            targets.tags.insert(element.tag.to_ascii_lowercase().into());
                        }
                        ElementType::Template => {}
                        // Components/slots can render unknown roots and attributes.
                        _ => targets.mark_unknown(),
                    }
                    for prop in &element.props {
                        match prop {
                            PropNode::Attribute(attribute) => {
                                if [
                                    "is",
                                    "ref",
                                    "className",
                                    "classList",
                                    "innerHTML",
                                    "outerHTML",
                                ]
                                .iter()
                                .any(|name| attribute.name.eq_ignore_ascii_case(name))
                                    || attribute
                                        .name
                                        .get(..2)
                                        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("on"))
                                {
                                    targets.mark_unknown();
                                }
                                if let Some(value) = &attribute.value {
                                    if attribute.name.eq_ignore_ascii_case("class") {
                                        targets.classes.extend(
                                            value
                                                .content
                                                .split_ascii_whitespace()
                                                .map(ToCompactString::to_compact_string),
                                        );
                                    } else if attribute.name.eq_ignore_ascii_case("id") {
                                        targets.ids.insert(value.content.to_compact_string());
                                    }
                                }
                            }
                            PropNode::Directive(directive) => match directive.name {
                                "if" | "else" | "else-if" | "for" | "show" | "once" | "memo"
                                | "text" | "pre" | "cloak" => {}
                                // Bindings include DOM properties and ref callbacks;
                                // v-html, model/custom directives and handlers can mutate DOM.
                                _ => targets.mark_unknown(),
                            },
                        }
                    }
                }
                TemplateChildNode::If(node) => {
                    for branch in &node.branches {
                        pending.extend(branch.children.iter());
                    }
                }
                TemplateChildNode::IfBranch(node) => pending.extend(node.children.iter()),
                TemplateChildNode::For(node) => pending.extend(node.children.iter()),
                TemplateChildNode::Hoisted(_) => targets.mark_unknown(),
                _ => {}
            }
        }
        targets
    }

    fn mark_unknown(&mut self) {
        self.unknown_classes = true;
        self.unknown_ids = true;
        self.unknown_tags = true;
    }

    /// Only a standalone simple global subject can prove foreign ownership.
    /// Compound selectors, lists and filters retain their existing advice.
    pub(super) fn is_foreign_global(&self, selector: &Selector<'_>) -> bool {
        let mut components = selector.iter_raw_match_order();
        let Some(Component::NonTSPseudoClass(PseudoClass::CustomFunction { name, arguments })) =
            components.next()
        else {
            return false;
        };
        if name.as_ref() != "global" || components.next().is_some() {
            return false;
        }
        let mut tokens = arguments.0.iter().filter(|token| !token.is_whitespace());
        match (tokens.next(), tokens.next(), tokens.next()) {
            (
                Some(TokenOrValue::Token(Token::Delim('.'))),
                Some(TokenOrValue::Token(Token::Ident(name))),
                None,
            ) => !self.unknown_classes && !self.classes.contains(name.as_ref()),
            (Some(TokenOrValue::Token(Token::IDHash(name))), None, None) => {
                !self.unknown_ids && !self.ids.contains(name.as_ref())
            }
            (Some(TokenOrValue::Token(Token::Ident(name))), None, None) => {
                !self.unknown_tags && !self.tags.contains(name.to_ascii_lowercase().as_str())
            }
            _ => false,
        }
    }
}

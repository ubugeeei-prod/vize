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

    /// Prove absence only from one fully recognized global subject.
    /// Vue replaces the outer selector with the global argument, so outer
    /// class/ID/type constraints must never supply the absence proof.
    pub(super) fn is_foreign_global(&self, selector: &Selector<'_>) -> bool {
        let mut global = None;
        for component in selector.iter_raw_match_order() {
            match component {
                Component::NonTSPseudoClass(PseudoClass::CustomFunction { name, arguments })
                    if name.as_ref() == "global" && global.is_none() =>
                {
                    global = self.foreign_compound(&arguments.0);
                    if global.is_none() {
                        return false;
                    }
                }
                Component::Class(_) | Component::ID(_) | Component::ExplicitUniversalType => {}
                Component::LocalName(name) if is_html_tag(name.lower_name.as_ref()) => {}
                // Includes multiple globals, every namespace, combinator,
                // nesting, attribute and pseudo/filter component.
                _ => return false,
            }
        }
        global == Some(true)
    }

    fn foreign_compound(&self, arguments: &[TokenOrValue<'_>]) -> Option<bool> {
        let first = arguments.iter().position(|token| !token.is_whitespace())?;
        let last = arguments.iter().rposition(|token| !token.is_whitespace())?;
        let mut tokens = arguments.get(first..=last)?.iter().peekable();
        if let Some(TokenOrValue::Token(Token::Ident(name))) = tokens.peek() {
            if first == last {
                return Some(
                    !self.unknown_tags && !self.tags.contains(name.to_ascii_lowercase().as_str()),
                );
            }
            if !is_html_tag(name.to_ascii_lowercase().as_str()) {
                return None;
            }
            tokens.next();
        } else if matches!(tokens.peek(), Some(TokenOrValue::Token(Token::Delim('*')))) {
            // A universal prefix adds no restriction. The full nonempty
            // class/ID suffix must still establish a necessary absent fact.
            tokens.next();
        }
        let mut foreign = false;
        let mut constrained = false;
        while let Some(token) = tokens.next() {
            match token {
                TokenOrValue::Token(Token::Delim('.')) => {
                    let Some(TokenOrValue::Token(Token::Ident(name))) = tokens.next() else {
                        return None;
                    };
                    foreign |= !self.unknown_classes && !self.classes.contains(name.as_ref());
                }
                TokenOrValue::Token(Token::IDHash(name)) => {
                    foreign |= !self.unknown_ids && !self.ids.contains(name.as_ref());
                }
                // Internal whitespace is a descendant boundary; comments,
                // commas, attributes, pseudos and all unknown tokens refuse proof.
                _ => return None,
            }
            constrained = true;
        }
        constrained.then_some(foreign)
    }
}

//! Conservative ownership facts from the L1 template already parsed for linting.

use lightningcss::properties::custom::{Token, TokenOrValue};
use lightningcss::selector::{Component, PseudoClass, Selector};
use vize_l0::{
    FxHashSet, Namespace, String, ToCompactString, is_html_tag, is_math_ml_tag, is_svg_tag,
};
use vize_l1::markup::directive::{DirectivePrefix, DirectiveSyntax, VueDirectives};
use vize_l1::markup::{Attribute, Element, SurfaceChild, SurfaceTree};

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
    pub(super) fn from_surface(root: &SurfaceTree<'_>) -> Self {
        let mut targets = Self::default();
        // A single root can receive a caller's class/id through fallthrough attrs.
        // Require two unconditional native roots before proving their absence.
        let fragment = root
            .children
            .iter()
            .filter(|child| unconditional_native_root(child))
            .count()
            >= 2;
        targets.unknown_classes = !fragment;
        targets.unknown_ids = !fragment;
        let mut pending: Vec<_> = root
            .children
            .iter()
            .map(|child| (child, None, false))
            .collect();
        while let Some((child, parent, inherited_verbatim)) = pending.pop() {
            if let SurfaceChild::Element(element) = child {
                let tag = element.tag();
                let namespace = element_namespace(parent, tag);
                let verbatim = element_verbatim(element, inherited_verbatim);
                pending.extend(
                    element
                        .children
                        .iter()
                        .map(|child| (child, Some((namespace, tag)), verbatim)),
                );
                if native_html_element(element, parent, verbatim) {
                    targets.tags.insert(tag.to_ascii_lowercase().into());
                } else if !structural_template(element, verbatim) {
                    // Components, slots and foreign namespaces can render
                    // unknown roots and attributes.
                    targets.mark_unknown();
                }
                targets.collect_attributes(element, verbatim);
            }
        }
        targets
    }

    fn collect_attributes(&mut self, element: &Element<'_>, verbatim: bool) {
        for attribute in &element.open.attrs {
            match directive_name(attribute, verbatim) {
                Ok(Some(
                    "if" | "else" | "else-if" | "for" | "show" | "once" | "memo" | "text" | "pre"
                    | "cloak",
                )) => {}
                Ok(Some(_)) | Err(()) => {
                    // Bindings include DOM properties and ref callbacks;
                    // v-html, model/custom directives and handlers can mutate DOM.
                    self.mark_unknown();
                }
                Ok(None) => self.collect_static_attribute(attribute),
            }
        }
    }

    fn collect_static_attribute(&mut self, attribute: &Attribute<'_>) {
        let name = attribute.name.text;
        if [
            "is",
            "ref",
            "className",
            "classList",
            "innerHTML",
            "outerHTML",
        ]
        .iter()
        .any(|candidate| name.eq_ignore_ascii_case(candidate))
            || name
                .get(..2)
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("on"))
        {
            self.mark_unknown();
        }
        let Some(value) = &attribute.value else {
            return;
        };
        let raw = value.content.text;
        let decoded = raw
            .contains('&')
            .then(|| vize_l4::targets::dom::artifact::decode_html_attribute_entities(raw));
        let value = decoded.as_deref().unwrap_or(raw);
        if name.eq_ignore_ascii_case("class") {
            self.classes.extend(
                value
                    .split_ascii_whitespace()
                    .map(ToCompactString::to_compact_string),
            );
        } else if name.eq_ignore_ascii_case("id") {
            self.ids.insert(value.to_compact_string());
        }
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

fn unconditional_native_root(child: &SurfaceChild<'_>) -> bool {
    let SurfaceChild::Element(element) = child else {
        return false;
    };
    let verbatim = element_verbatim(element, false);
    native_html_element(element, None, verbatim) && !has_control_directive(element, verbatim)
}

fn has_control_directive(element: &Element<'_>, verbatim: bool) -> bool {
    element.open.attrs.iter().any(|attribute| {
        directive_name(attribute, verbatim)
            .ok()
            .flatten()
            .is_some_and(|name| matches!(name, "if" | "else" | "else-if" | "for"))
    })
}

fn structural_template(element: &Element<'_>, verbatim: bool) -> bool {
    !verbatim
        && element.tag() == "template"
        && element.open.attrs.iter().any(|attribute| {
            directive_name(attribute, verbatim)
                .ok()
                .flatten()
                .is_some_and(|name| matches!(name, "if" | "else-if" | "else" | "for" | "slot"))
        })
}

fn native_html_element(
    element: &Element<'_>,
    parent: Option<(Namespace, &str)>,
    verbatim: bool,
) -> bool {
    let tag = element.tag();
    element_namespace(parent, tag) == Namespace::Html
        && is_html_tag(tag)
        && !structural_template(element, verbatim)
        && !static_vue_component(element, verbatim)
}

fn static_vue_component(element: &Element<'_>, verbatim: bool) -> bool {
    !verbatim
        && element.open.attrs.iter().any(|attribute| {
            directive_name(attribute, verbatim).is_ok_and(|directive| directive.is_none())
                && attribute.name.text == "is"
                && attribute
                    .value
                    .as_ref()
                    .is_some_and(|value| value.content.text.starts_with("vue:"))
        })
}

fn element_verbatim(element: &Element<'_>, inherited: bool) -> bool {
    inherited
        || element.open.is_verbatim()
        || element.open.attrs.iter().any(|attribute| {
            parsed_directive_name(attribute)
                .ok()
                .flatten()
                .is_some_and(|name| name == "pre")
        })
}

fn directive_name<'a>(attribute: &'a Attribute<'a>, verbatim: bool) -> Result<Option<&'a str>, ()> {
    if verbatim {
        return Ok(None);
    }
    parsed_directive_name(attribute)
}

fn parsed_directive_name<'a>(attribute: &'a Attribute<'a>) -> Result<Option<&'a str>, ()> {
    let raw = attribute.name.text;
    let Some(directive) = VueDirectives.decompose(raw, 0).map_err(|_| ())? else {
        return Ok(None);
    };
    Ok(Some(match directive.prefix {
        DirectivePrefix::Full => directive.name.slice(raw),
        DirectivePrefix::Bind | DirectivePrefix::Prop => "bind",
        DirectivePrefix::On => "on",
        DirectivePrefix::Slot => "slot",
    }))
}

fn element_namespace(parent: Option<(Namespace, &str)>, tag: &str) -> Namespace {
    if is_svg_tag(tag) {
        Namespace::Svg
    } else if is_math_ml_tag(tag) {
        Namespace::MathMl
    } else {
        parent.map_or(Namespace::Html, |(namespace, parent_tag)| match namespace {
            Namespace::Svg if matches!(parent_tag, "foreignObject" | "desc" | "title") => {
                Namespace::Html
            }
            Namespace::MathMl
                if matches!(
                    parent_tag,
                    "annotation-xml" | "mi" | "mo" | "mn" | "ms" | "mtext"
                ) =>
            {
                Namespace::Html
            }
            other => other,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::TemplateTargets;
    use vize_l0::{Allocator, FxHashSet, String};

    fn names(values: &[&str]) -> FxHashSet<String> {
        values.iter().copied().map(String::from).collect()
    }

    fn targets(source: &str) -> TemplateTargets {
        let allocator = Allocator::new();
        let (surface, _, errors) =
            vize_l1::markup::parse_with_authored_diagnostics(&allocator, source);
        assert_eq!(errors.len(), 0, "{errors:?}");
        TemplateTargets::from_surface(&surface)
    }

    #[test]
    fn l1_surface_decodes_static_targets_for_a_fragment() {
        let targets = targets(r#"<div class="lo&#99;al" id="main"/><aside/>"#);
        assert_eq!(targets.unknown_classes, false);
        assert_eq!(targets.unknown_ids, false);
        assert_eq!(targets.unknown_tags, false);
        assert_eq!(targets.classes, names(&["local"]));
        assert_eq!(targets.ids, names(&["main"]));
        assert_eq!(targets.tags, names(&["div", "aside"]));
    }

    #[test]
    fn l1_surface_keeps_verbatim_attributes_static() {
        let targets = targets(r#"<div v-pre :class="foreign" @click="handler"/><aside/>"#);
        assert_eq!(targets.unknown_classes, false);
        assert_eq!(targets.unknown_ids, false);
        assert_eq!(targets.unknown_tags, false);
        assert_eq!(targets.classes, FxHashSet::default());
        assert_eq!(targets.ids, FxHashSet::default());
        assert_eq!(targets.tags, names(&["div", "aside"]));
    }

    #[test]
    fn l1_surface_refuses_dynamic_and_foreign_ownership_proofs() {
        for source in [
            r#"<div :class="foreign"/><aside/>"#,
            "<div/><svg><circle/></svg><aside/>",
            r#"<div is="vue:Foreign"/><aside/>"#,
        ] {
            let targets = targets(source);
            assert_eq!(targets.unknown_classes, true, "{source}");
            assert_eq!(targets.unknown_ids, true, "{source}");
            assert_eq!(targets.unknown_tags, true, "{source}");
        }
    }

    #[test]
    fn l1_surface_walks_through_structural_template_carriers() {
        let targets = targets(r#"<template v-if="ready"><div class="local"/></template><aside/>"#);
        assert_eq!(targets.unknown_classes, true);
        assert_eq!(targets.unknown_ids, true);
        assert_eq!(targets.unknown_tags, false);
        assert_eq!(targets.classes, names(&["local"]));
        assert_eq!(targets.ids, FxHashSet::default());
        assert_eq!(targets.tags, names(&["div", "aside"]));
    }
}

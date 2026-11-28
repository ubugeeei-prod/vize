//! The walk Nuxt's template diagnostics make over Vue's parsed template.
//!
//! Each step mirrors `nuxt/src/core/plugins/template-diagnostics.ts` as of
//! Nuxt 4.7: elements are visited in document order with the names their
//! `v-for` and `v-slot` declare in scope, a tag the script binds is skipped,
//! and positions are the attribute's start in the whole SFC, in Vue's UTF-16
//! line, column and offset.

use vize_atelier_core::options::ParserOptions;
use vize_atelier_core::parser::Parser;
use vize_atelier_core::{
    AttributeNode, CompilerError, DirectiveNode, ElementNode, ElementType, ExpressionNode,
    PropNode, TemplateChildNode,
};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_l0::{Allocator, String};

use super::values::{Values, scoped_names};

const LINK_TAGS: [&str; 2] = ["NuxtLink", "nuxt-link"];
const ISLAND_TAGS: [&str; 2] = ["NuxtIsland", "nuxt-island"];
const ISLAND_SERIALIZED_PROPS: [&str; 2] = ["props", "context"];
const LINK_TARGET_PROPS: [&str; 3] = ["to", "href", "external"];
const RESERVED_PROPS: [&str; 4] = ["key", "ref", "ref_for", "ref_key"];

/// Which of Nuxt's checks a finding feeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FindingKind {
    /// A static `<NuxtLink>` target; Nuxt matches it against the page routes.
    Link,
    /// `v-model`, or `v-on` other than `@error`, on a tag Nuxt may render as
    /// a server component.
    Event,
    /// A function or class bound as a prop of such a tag.
    Prop,
    /// A function or class in a `<NuxtIsland>` `props`/`context` object.
    Island,
}

impl FindingKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Link => "link",
            Self::Event => "event",
            Self::Prop => "prop",
            Self::Island => "island",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Finding {
    pub kind: FindingKind,
    pub tag: String,
    pub line: u32,
    pub column: u32,
    pub offset: u32,
    pub attribute: Option<String>,
    pub to: Option<String>,
    pub directive: Option<String>,
    pub prop: Option<String>,
    pub value: Option<&'static str>,
    pub island_name: Option<String>,
    pub island_prop: Option<String>,
}

/// Find what Nuxt's template diagnostics check in `source`, for an app whose
/// server components have the tags `server_tags`.
pub(crate) fn nuxt_template_findings(source: &str, server_tags: &[&str]) -> Vec<Finding> {
    let Ok(descriptor) = parse_sfc(source, SfcParseOptions::default()) else {
        return Vec::new();
    };
    let Some(template) = descriptor.template.as_ref() else {
        return Vec::new();
    };
    // Vue reuses the SFC's own template AST only for inline HTML templates.
    if template.src.is_some() || template.lang.as_deref().is_some_and(|lang| lang != "html") {
        return Vec::new();
    }
    let content = template.content.as_ref();
    let allocator = Allocator::new();
    let (root, errors) = Parser::with_options(&allocator, content, parser_options()).parse();
    if !errors.iter().all(CompilerError::is_recoverable) {
        return Vec::new();
    }
    let mut walk = Walk {
        source,
        content,
        base: template.loc.start,
        server_tags,
        values: Values::new(&descriptor),
        scope: Vec::new(),
        findings: Vec::new(),
    };
    walk.children(&root.children);
    walk.findings
}

/// The DOM compiler's tag classification, as Vue's SFC parse applies it.
fn parser_options() -> ParserOptions {
    ParserOptions {
        is_pre_tag: |tag| tag == "pre",
        is_native_tag: Some(vize_l0::is_platform_tag),
        ..ParserOptions::default()
    }
}

struct Walk<'s, 't> {
    source: &'s str,
    content: &'t str,
    base: usize,
    server_tags: &'s [&'s str],
    values: Values<'s>,
    scope: Vec<&'t str>,
    findings: Vec<Finding>,
}

impl<'s, 't> Walk<'s, 't> {
    fn children(&mut self, children: &'t [TemplateChildNode<'t>]) {
        for child in children {
            if let TemplateChildNode::Element(element) = child {
                self.element(element);
            }
        }
    }

    fn element(&mut self, element: &'t ElementNode<'t>) {
        let outer = self.scope.len();
        self.declare(element, "for");
        self.check(element);
        self.declare(element, "slot");
        self.children(&element.children);
        self.scope.truncate(outer);
    }

    /// Bring the names an element's `v-for` alias or `v-slot` props declare
    /// into scope, read from the authored text as Nuxt reads them.
    fn declare(&mut self, element: &'t ElementNode<'t>, directive: &str) {
        for prop in &element.props {
            let PropNode::Directive(dir) = prop else {
                continue;
            };
            if dir.name != directive {
                continue;
            }
            let Some(text) = dir.exp.as_ref().and_then(|exp| self.authored(exp)) else {
                continue;
            };
            let text = if directive == "for" {
                for_alias(text)
            } else {
                text
            };
            self.scope.extend(scoped_names(text));
        }
    }

    fn check(&mut self, element: &'t ElementNode<'t>) {
        // Vue keeps a `v-pre` subtree, and a custom element, plain elements.
        if element.tag_type != ElementType::Component {
            return;
        }
        let Some(tag) = self.authored_tag(element) else {
            return;
        };
        let link = LINK_TAGS.contains(&tag);
        let island = ISLAND_TAGS.contains(&tag);
        let server = self.server_tags.contains(&tag);
        if !(link || island || server) || self.values.is_bound(tag) {
            return;
        }
        if link {
            self.link(element, tag);
        }
        if server {
            self.server_component(element, tag);
        }
        if island {
            self.island(element, tag);
        }
    }

    fn link(&mut self, element: &'t ElementNode<'t>, tag: &str) {
        let mut target: Option<&AttributeNode<'t>> = None;
        for prop in &element.props {
            match prop {
                PropNode::Attribute(attr) if attr.name == "external" => return,
                PropNode::Attribute(attr) if matches!(attr.name, "to" | "href") => {
                    target = Some(attr);
                }
                PropNode::Directive(dir) if dir.name == "bind" => match static_arg(dir) {
                    Some(arg) if !LINK_TARGET_PROPS.contains(&arg) => {}
                    _ => return,
                },
                _ => {}
            }
        }
        let Some((target, value)) =
            target.and_then(|attr| attr.value.as_ref().map(|value| (attr, value)))
        else {
            return;
        };
        let mut finding = self.finding(FindingKind::Link, tag, target.loc.span.start);
        finding.attribute = Some(target.name.into());
        finding.to = Some(value.content.into());
        self.findings.push(finding);
    }

    fn server_component(&mut self, element: &'t ElementNode<'t>, tag: &str) {
        for prop in &element.props {
            let PropNode::Directive(dir) = prop else {
                continue;
            };
            let arg = static_arg(dir);
            if dir.name == "model" || (dir.name == "on" && arg.is_some_and(|arg| arg != "error")) {
                let start = dir.loc.span.start;
                let directive = self.raw_name(start);
                let mut finding = self.finding(FindingKind::Event, tag, start);
                finding.directive = Some(directive.into());
                self.findings.push(finding);
                continue;
            }
            if dir.name != "bind" {
                continue;
            }
            let (Some(exp), Some(arg)) = (expression(dir), arg) else {
                continue;
            };
            if RESERVED_PROPS.contains(&arg) {
                continue;
            }
            let raw = self.authored(exp).unwrap_or_default();
            let Some(value) =
                self.values
                    .describe_expression(simple_content(exp), raw, &self.scope)
            else {
                continue;
            };
            let mut finding = self.finding(FindingKind::Prop, tag, dir.loc.span.start);
            finding.prop = Some(arg.into());
            finding.value = Some(value);
            self.findings.push(finding);
        }
    }

    fn island(&mut self, element: &'t ElementNode<'t>, tag: &str) {
        let name = element.props.iter().find_map(|prop| match prop {
            PropNode::Attribute(attr) if attr.name == "name" => Some(attr),
            _ => None,
        });
        let name = name
            .and_then(|attr| attr.value.as_ref())
            .map(|value| value.content);
        for prop in &element.props {
            let PropNode::Directive(dir) = prop else {
                continue;
            };
            let Some(arg) = static_arg(dir) else {
                continue;
            };
            if dir.name != "bind" || !ISLAND_SERIALIZED_PROPS.contains(&arg) {
                continue;
            }
            let Some(exp) = expression(dir) else {
                continue;
            };
            let entries = self
                .values
                .describe_object_entries(simple_content(exp), &self.scope);
            for (key, value) in entries {
                let mut finding = self.finding(FindingKind::Island, tag, dir.loc.span.start);
                finding.prop = Some(key.map_or_else(|| arg.into(), Into::into));
                finding.value = Some(value);
                finding.island_name = name.map(Into::into);
                finding.island_prop = Some(arg.into());
                self.findings.push(finding);
            }
        }
    }

    fn finding(&self, kind: FindingKind, tag: &str, start: u32) -> Finding {
        let (line, column, offset) = self.position(start);
        Finding {
            kind,
            tag: tag.into(),
            line,
            column,
            offset,
            attribute: None,
            to: None,
            directive: None,
            prop: None,
            value: None,
            island_name: None,
            island_prop: None,
        }
    }

    /// Vue's position of a template byte offset in the whole SFC: one-based
    /// line, one-based column and offset, both counted in UTF-16 units.
    fn position(&self, start: u32) -> (u32, u32, u32) {
        let at = self.base.saturating_add(start as usize);
        let before = self.source.get(..at).unwrap_or(self.source);
        let line_start = before.rfind('\n').map_or(0, |newline| newline + 1);
        let current = before.get(line_start..).unwrap_or_default();
        let units = |text: &str| u32::try_from(text.encode_utf16().count()).unwrap_or(u32::MAX);
        let line = before.bytes().filter(|byte| *byte == b'\n').count();
        (
            u32::try_from(line).unwrap_or(u32::MAX).saturating_add(1),
            units(current).saturating_add(1),
            units(before),
        )
    }

    /// The tag as authored: a static `vue:` cast keeps Vue's tag `component`.
    fn authored_tag(&self, element: &ElementNode<'t>) -> Option<&'t str> {
        let start = (element.loc.span.start as usize).checked_add(1)?;
        let rest = self.content.get(start..)?;
        rest.get(..name_end(rest))
    }

    /// Vue's `rawName`: the whole authored attribute name with its argument
    /// and modifiers.
    fn raw_name(&self, start: u32) -> &'t str {
        let rest = self.content.get(start as usize..).unwrap_or_default();
        rest.get(..name_end(rest)).unwrap_or_default()
    }

    /// The expression as written in the attribute value (`loc.source`).
    fn authored(&self, exp: &ExpressionNode<'t>) -> Option<&'t str> {
        let span = &exp.loc().span;
        self.content.get(span.start as usize..span.end as usize)
    }
}

fn name_end(text: &str) -> usize {
    text.find(|c: char| c.is_ascii_whitespace() || matches!(c, '=' | '/' | '>'))
        .unwrap_or(text.len())
}

/// The argument of a directive when it is static, as Vue parsed it.
fn static_arg<'t>(dir: &DirectiveNode<'t>) -> Option<&'t str> {
    match dir.arg.as_ref()? {
        ExpressionNode::Simple(arg) if arg.is_static => Some(arg.content),
        _ => None,
    }
}

/// The directive's expression as Vue's parser leaves it: the same-name
/// shorthand (`:foo`) has none until a later transform adds it.
fn expression<'d, 't>(dir: &'d DirectiveNode<'t>) -> Option<&'d ExpressionNode<'t>> {
    if dir.shorthand {
        return None;
    }
    dir.exp.as_ref()
}

fn simple_content<'t>(exp: &ExpressionNode<'t>) -> &'t str {
    match exp {
        ExpressionNode::Simple(simple) => simple.content,
        ExpressionNode::Compound(_) => "",
    }
}

/// The alias part of a `v-for` value: the text before the first
/// `/\s+(?:in|of)\s+/` match.
fn for_alias(text: &str) -> &str {
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if !is_js_space(bytes, index) {
            index += 1;
            continue;
        }
        let run = index;
        while index < bytes.len() && is_js_space(bytes, index) {
            index += 1;
        }
        let keyword = bytes.get(index..index + 2);
        if matches!(keyword, Some(b"in" | b"of")) && is_js_space(bytes, index + 2) {
            return text.get(..run).unwrap_or(text);
        }
    }
    text
}

fn is_js_space(bytes: &[u8], index: usize) -> bool {
    bytes
        .get(index)
        .is_some_and(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0B | 0x0C))
}

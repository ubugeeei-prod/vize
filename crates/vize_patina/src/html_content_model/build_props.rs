//! The prop facts composition prunes with (Davinci FP-3): which nodes a bare
//! `v-if` identifier guards, and which props each component usage passes.
//!
//! Recorded only for a composable skeleton ([`super::composable_skeleton`]);
//! the linter's per-file skeleton never pays for it.
//!
//! A guard is recorded only where the identifier is the component's own
//! binding: outside every scope a `v-for`, `v-slot` or `slot-scope` opens
//! (the element's own included), since a scope variable of the same name
//! would shadow it. A usage's passed props are every attribute, argument and
//! model name it writes, camelized; a spread (`v-bind="obj"`, `v-on="obj"`)
//! or a dynamic argument makes the usage opaque — every prop may be passed.

use vize_relief::{ExpressionNode, PropNode};
use vize_s0::CompactString;

use super::skeleton::{PropFacts, Skeleton, component_usage_name};
use crate::markup::MarkupElement;

/// Records [`PropFacts`] while the builder walks the template.
#[derive(Debug, Default)]
pub(super) struct PropRecorder {
    pub(super) facts: PropFacts,
    /// Per entered element: whether it opened a scope.
    scopes: Vec<bool>,
    /// How many entered elements opened a scope.
    depth: u32,
}

impl PropRecorder {
    /// After `element` opened the `opened` skeleton nodes from `first`.
    pub(super) fn enter(
        &mut self,
        element: &MarkupElement<'_>,
        first: u32,
        opened: u8,
        skeleton: &Skeleton,
    ) {
        let Some(node) = element.as_relief() else {
            // Only Vue templates compose; anything else opens a scope so no
            // guard below it is trusted.
            self.push(true);
            return;
        };
        let mut scoped = false;
        let mut guard = None;
        for prop in &node.props {
            match prop {
                PropNode::Attribute(attr) => {
                    scoped |= matches!(attr.name, "slot-scope" | "scope");
                }
                PropNode::Directive(dir) => {
                    scoped |= matches!(dir.name, "for" | "slot");
                    if dir.name == "if" {
                        guard = simple(dir.exp.as_ref()).filter(|exp| is_identifier(exp));
                    }
                }
            }
        }
        let opened = first..first + u32::from(opened);
        if let (Some(guard), false, 0) = (guard, scoped, self.depth)
            && !opened.is_empty()
        {
            self.facts.guards.push((first, CompactString::new(guard)));
        }
        for index in opened {
            if component_usage_name(skeleton.node(index)).is_some() {
                self.passed(node, index);
            }
        }
        self.push(scoped);
    }

    /// After an element's subtree is done.
    pub(super) fn exit(&mut self) {
        if self.scopes.pop() == Some(true) {
            self.depth -= 1;
        }
    }

    fn push(&mut self, scoped: bool) {
        self.scopes.push(scoped);
        self.depth += u32::from(scoped);
    }

    fn passed(&mut self, node: &vize_relief::ElementNode<'_>, usage: u32) {
        for prop in &node.props {
            let name = match prop {
                PropNode::Attribute(attr) => camelize(attr.name, ""),
                PropNode::Directive(dir) => {
                    let arg = match dir.arg.as_ref() {
                        Some(ExpressionNode::Simple(arg)) if arg.is_static => Some(arg.content),
                        Some(_) => None,
                        None if dir.name == "model" => Some("modelValue"),
                        None if matches!(dir.name, "bind" | "on") => None,
                        // `v-html`/`v-text` set a DOM prop of that name; other
                        // directives pass nothing.
                        None => match dir.name {
                            "html" => Some("innerHTML"),
                            "text" => Some("textContent"),
                            _ => continue,
                        },
                    };
                    let Some(arg) = arg else {
                        self.facts.opaque.push(usage);
                        continue;
                    };
                    match dir.name {
                        "on" => camelize(arg, "on"),
                        "bind" | "model" | "html" | "text" => camelize(arg, ""),
                        _ => continue,
                    }
                }
            };
            self.facts.passed.push((usage, name));
        }
    }
}

fn simple<'a>(exp: Option<&ExpressionNode<'a>>) -> Option<&'a str> {
    match exp? {
        ExpressionNode::Simple(simple) if !simple.is_static => Some(simple.content.trim()),
        _ => None,
    }
}

/// A bare JavaScript identifier (ASCII), not a reserved literal.
fn is_identifier(text: &str) -> bool {
    let mut bytes = text.bytes();
    bytes
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || matches!(first, b'_' | b'$'))
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
        && !matches!(text, "true" | "false" | "null" | "undefined" | "this")
}

/// Vue's `camelize` (`copy-text` → `copyText`); with a prefix (`on`), the
/// first letter is capitalized too (`click` → `onClick`).
fn camelize(name: &str, prefix: &str) -> CompactString {
    let mut out = CompactString::new(prefix);
    let mut upper = !prefix.is_empty();
    for ch in name.chars() {
        if ch == '-' {
            upper = true;
        } else if upper {
            out.extend(ch.to_uppercase());
            upper = false;
        } else {
            out.push(ch);
        }
    }
    out
}

//! How the template uses a `<script setup>` local function.
//!
//! A function read only by `v-on` handlers runs when an event fires, after
//! setup has returned. Any other read (an interpolation, a bound attribute, a
//! component tag) runs it while the component renders, or hands it on, so the
//! rule does not decide its owner.
//!
//! The scan errs one way in each direction: an "other" read is matched by
//! name anywhere in an expression, strings included, and a handler read only
//! by a bare identifier outside strings and member accesses. An unparsed
//! template counts every spelling of the name as an other read.

use crate::markup::surface_source::{DecodedSource, directive, element_verbatim};
use crate::rules::script::SfcScriptContext;
use vize_l1::markup::{Element, SurfaceChild, SurfaceTree};

#[derive(Clone, Copy, Default)]
pub(super) struct TemplateUse {
    /// A `v-on` handler reads the name.
    pub(super) handler: bool,
    /// Something other than a `v-on` handler may read the name.
    pub(super) other: bool,
}

/// The template uses of each of `names`, in order.
pub(super) fn template_uses(sfc: &SfcScriptContext<'_>, names: &[&str]) -> Vec<TemplateUse> {
    let mut uses = vec![TemplateUse::default(); names.len()];
    if let Some(root) = sfc.template_surface {
        let mut scan = Scan {
            root,
            names,
            uses: &mut uses,
        };
        scan.walk(&root.children, false);
    } else if let Some(source) = sfc.template_source {
        for (name, found) in names.iter().zip(uses.iter_mut()) {
            found.other = mentions(source, name, false);
        }
    }
    uses
}

struct Scan<'s, 'a> {
    root: &'s SurfaceTree<'a>,
    names: &'s [&'s str],
    uses: &'s mut [TemplateUse],
}

impl<'a> Scan<'_, 'a> {
    fn walk(&mut self, children: &'a [SurfaceChild<'a>], inherited_verbatim: bool) {
        for child in children {
            match child {
                SurfaceChild::Element(element) => self.element(element, inherited_verbatim),
                SurfaceChild::Interpolation(interpolation) if !inherited_verbatim => {
                    if let Some(source) =
                        DecodedSource::interpolation(self.root.source, &interpolation.content)
                    {
                        self.other(source.text());
                    }
                }
                _ => {}
            }
        }
    }

    fn element(&mut self, element: &'a Element<'a>, inherited_verbatim: bool) {
        let verbatim = element_verbatim(element, inherited_verbatim);
        if !verbatim {
            let tag = element.tag();
            for (name, found) in self.names.iter().zip(self.uses.iter_mut()) {
                found.other |= same_component_name(tag, name);
            }
        }
        for attribute in &element.open.attrs {
            let Some(directive) = directive(self.root, attribute, verbatim) else {
                continue;
            };
            if let Some(crate::markup::surface_source::DirectiveArgument::Dynamic(argument)) =
                directive.argument
            {
                self.other(argument);
            }
            let Some(value) = directive.value else {
                continue;
            };
            let text = value.text();
            if directive.name == "on" {
                for (name, found) in self.names.iter().zip(self.uses.iter_mut()) {
                    found.handler |= mentions(text, name, true);
                }
            } else {
                self.other(text);
            }
        }
        self.walk(&element.children, verbatim);
    }

    fn other(&mut self, text: &str) {
        for (name, found) in self.names.iter().zip(self.uses.iter_mut()) {
            found.other |= mentions(text, name, false);
        }
    }
}

/// `<FooBar>`, `<foo-bar>` and `<foobar>` all name the binding `FooBar`.
fn same_component_name(tag: &str, name: &str) -> bool {
    let mut tag = tag.bytes().filter(|byte| *byte != b'-');
    let mut name = name.bytes();
    loop {
        match (tag.next(), name.next()) {
            (None, None) => return true,
            (Some(left), Some(right)) if left.eq_ignore_ascii_case(&right) => {}
            _ => return false,
        }
    }
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$' || byte >= 0x80
}

/// Whether `text` spells the identifier `name`. A `strict` match skips string
/// and template literals and member names (`a.name`), so it only finds reads
/// of the binding itself.
fn mentions(text: &str, name: &str, strict: bool) -> bool {
    let bytes = text.as_bytes();
    let needle = name.as_bytes();
    if needle.is_empty() {
        return false;
    }
    let mut quote: Option<u8> = None;
    let mut previous: Option<u8> = None;
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        if strict {
            if let Some(open) = quote {
                if byte == b'\\' {
                    index += 2;
                    previous = None;
                    continue;
                }
                if byte == open {
                    quote = None;
                }
                index += 1;
                previous = Some(byte);
                continue;
            }
            if matches!(byte, b'\'' | b'"' | b'`') {
                quote = Some(byte);
                index += 1;
                previous = Some(byte);
                continue;
            }
        }
        let starts_token = previous.is_none_or(|previous| !is_identifier_byte(previous));
        if starts_token
            && bytes.get(index..index + needle.len()) == Some(needle)
            && bytes
                .get(index + needle.len())
                .is_none_or(|next| !is_identifier_byte(*next))
            && !(strict && previous == Some(b'.'))
        {
            return true;
        }
        previous = Some(byte);
        index += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{mentions, same_component_name};

    #[test]
    fn strict_mentions_skip_strings_and_members() {
        assert!(mentions("onClick", "onClick", true));
        assert!(mentions("onClick($event)", "onClick", true));
        assert!(mentions("() => onClick(1)", "onClick", true));
        assert!(!mentions("log('onClick')", "onClick", true));
        assert!(!mentions("menu.onClick()", "onClick", true));
        assert!(!mentions("onClickOutside()", "onClick", true));
        assert!(mentions("log('onClick')", "onClick", false));
    }

    #[test]
    fn component_names_compare_across_casings() {
        assert!(same_component_name("foo-bar", "FooBar"));
        assert!(same_component_name("FooBar", "FooBar"));
        assert!(!same_component_name("foo-baz", "FooBar"));
    }
}

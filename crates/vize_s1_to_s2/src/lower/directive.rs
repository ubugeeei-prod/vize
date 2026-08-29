//! Directive-name analysis: the semantic split S1 deliberately leaves to
//! the lowering (P2-7 keeps names as one raw token).
//!
//! The grammar is Vue's authored spelling: `v-name:arg.mod1.mod2`,
//! shorthands `:` / `.` (v-bind), `@` (v-on), `#` (v-slot), and dynamic
//! arguments `[expr]`. Dots inside a dynamic argument's brackets never
//! split modifiers. The split is total — any name that does not parse as
//! a directive is a static attribute.

use alloc::vec::Vec;

/// Which directive a name spells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Head {
    If,
    ElseIf,
    Else,
    For,
    Model,
    Slot,
    Bind,
    On,
    Html,
    Text,
    Show,
    Cloak,
    Once,
    Pre,
    Memo,
    /// A user-defined directive (`v-pin`).
    Custom,
}

/// A directive argument, static or dynamic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Arg<'a> {
    /// `v-on:click` — the authored name.
    Static(&'a str),
    /// `v-on:[event]` — the text inside the brackets.
    Dynamic(&'a str),
}

/// One analyzed directive name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Directive<'a> {
    pub head: Head,
    /// The custom head's name without `v-` (`"pin"`); empty for
    /// built-ins.
    pub name: &'a str,
    pub arg: Option<Arg<'a>>,
    /// Modifier names in authored order, without their leading dots.
    pub modifiers: Vec<&'a str>,
}

/// How one authored attribute name reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AttrForm<'a> {
    /// A plain attribute.
    Static,
    /// A directive spelling.
    Directive(Directive<'a>),
}

/// Split an authored attribute name.
pub(crate) fn classify<'a>(name: &'a str) -> AttrForm<'a> {
    if let Some(rest) = name.strip_prefix(':') {
        return directive(Head::Bind, "", arg_first(rest));
    }
    if let Some(rest) = name.strip_prefix('.') {
        // `.prop="x"` — v-bind's `.prop` shorthand; the leading dot is
        // the spelling, the argument follows it.
        return directive(Head::Bind, "", arg_first(rest));
    }
    if let Some(rest) = name.strip_prefix('@') {
        return directive(Head::On, "", arg_first(rest));
    }
    if let Some(rest) = name.strip_prefix('#') {
        return directive(Head::Slot, "", arg_first(rest));
    }
    let Some(rest) = name.strip_prefix("v-") else {
        return AttrForm::Static;
    };
    // Head runs to the first `:` (argument) or `.` (modifiers).
    let head_end = rest.find([':', '.']).unwrap_or(rest.len());
    let head_text = &rest[..head_end];
    let tail = &rest[head_end..];
    let (head, name) = match head_text {
        "if" => (Head::If, ""),
        "else-if" => (Head::ElseIf, ""),
        "else" => (Head::Else, ""),
        "for" => (Head::For, ""),
        "model" => (Head::Model, ""),
        "slot" => (Head::Slot, ""),
        "bind" => (Head::Bind, ""),
        "on" => (Head::On, ""),
        "html" => (Head::Html, ""),
        "text" => (Head::Text, ""),
        "show" => (Head::Show, ""),
        "cloak" => (Head::Cloak, ""),
        "once" => (Head::Once, ""),
        "pre" => (Head::Pre, ""),
        "memo" => (Head::Memo, ""),
        other => (Head::Custom, other),
    };
    let (arg, modifiers) = match tail.strip_prefix(':') {
        Some(after_colon) => arg_first(after_colon),
        None => (None, split_modifiers(tail.strip_prefix('.').unwrap_or(""))),
    };
    AttrForm::Directive(Directive {
        head,
        name,
        arg,
        modifiers,
    })
}

fn directive<'a>(
    head: Head,
    name: &'a str,
    (arg, modifiers): (Option<Arg<'a>>, Vec<&'a str>),
) -> AttrForm<'a> {
    AttrForm::Directive(Directive {
        head,
        name,
        arg,
        modifiers,
    })
}

/// Parse `text` as an argument followed by modifiers.
fn arg_first<'a>(text: &'a str) -> (Option<Arg<'a>>, Vec<&'a str>) {
    if let Some(inner_and_rest) = text.strip_prefix('[') {
        // Dynamic argument: to the matching `]`, bracket-aware so
        // `[a[0]]` stays one argument. An unterminated bracket takes the
        // rest of the name (total; the tokenizer's diagnostic already
        // covers the malformed spelling).
        let mut depth = 1usize;
        for (index, byte) in inner_and_rest.bytes().enumerate() {
            match byte {
                b'[' => depth += 1,
                b']' => {
                    depth -= 1;
                    if depth == 0 {
                        let inner = &inner_and_rest[..index];
                        let rest = &inner_and_rest[index + 1..];
                        let modifiers = split_modifiers(rest.strip_prefix('.').unwrap_or(""));
                        return (Some(Arg::Dynamic(inner)), modifiers);
                    }
                }
                _ => {}
            }
        }
        return (Some(Arg::Dynamic(inner_and_rest)), Vec::new());
    }
    let arg_end = text.find('.').unwrap_or(text.len());
    let arg = &text[..arg_end];
    let modifiers = split_modifiers(text[arg_end..].strip_prefix('.').unwrap_or(""));
    let arg = if arg.is_empty() {
        None
    } else {
        Some(Arg::Static(arg))
    };
    (arg, modifiers)
}

/// Split dot-separated modifiers, dropping empty segments.
fn split_modifiers(text: &str) -> Vec<&str> {
    if text.is_empty() {
        return Vec::new();
    }
    text.split('.')
        .filter(|segment| !segment.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Arg, AttrForm, Head, classify};
    use alloc::vec;

    fn directive(form: AttrForm<'_>) -> super::Directive<'_> {
        match form {
            AttrForm::Directive(directive) => directive,
            AttrForm::Static => panic!("expected a directive"),
        }
    }

    #[test]
    fn plain_names_are_static() {
        assert_eq!(classify("id"), AttrForm::Static);
        assert_eq!(classify("data-x"), AttrForm::Static);
        assert_eq!(classify("viewBox"), AttrForm::Static);
    }

    #[test]
    fn full_spellings_split_head_arg_modifiers() {
        let d = directive(classify("v-on:click.stop.prevent"));
        assert_eq!(d.head, Head::On);
        assert_eq!(d.arg, Some(Arg::Static("click")));
        assert_eq!(d.modifiers, vec!["stop", "prevent"]);
    }

    #[test]
    fn shorthands_map_to_their_heads() {
        let bind = directive(classify(":id"));
        assert_eq!(bind.head, Head::Bind);
        assert_eq!(bind.arg, Some(Arg::Static("id")));

        let on = directive(classify("@click"));
        assert_eq!(on.head, Head::On);

        let slot = directive(classify("#named"));
        assert_eq!(slot.head, Head::Slot);
        assert_eq!(slot.arg, Some(Arg::Static("named")));

        let prop = directive(classify(".innerHTML"));
        assert_eq!(prop.head, Head::Bind);
        assert_eq!(prop.arg, Some(Arg::Static("innerHTML")));
    }

    #[test]
    fn dynamic_arguments_keep_inner_dots_and_brackets() {
        let d = directive(classify(":[key.path]"));
        assert_eq!(d.arg, Some(Arg::Dynamic("key.path")));
        assert!(d.modifiers.is_empty());

        let nested = directive(classify("v-bind:[a[0]].camel"));
        assert_eq!(nested.arg, Some(Arg::Dynamic("a[0]")));
        assert_eq!(nested.modifiers, vec!["camel"]);

        let unterminated = directive(classify(":[key"));
        assert_eq!(unterminated.arg, Some(Arg::Dynamic("key")));
    }

    #[test]
    fn custom_directives_keep_their_name() {
        let d = directive(classify("v-pin:top.lazy"));
        assert_eq!(d.head, Head::Custom);
        assert_eq!(d.name, "pin");
        assert_eq!(d.arg, Some(Arg::Static("top")));
        assert_eq!(d.modifiers, vec!["lazy"]);
    }

    #[test]
    fn modifiers_without_argument_attach_to_the_head() {
        let d = directive(classify("v-model.lazy.number"));
        assert_eq!(d.head, Head::Model);
        assert_eq!(d.arg, None);
        assert_eq!(d.modifiers, vec!["lazy", "number"]);
    }

    #[test]
    fn the_bare_v_dash_is_a_nameless_custom_directive() {
        let d = directive(classify("v-"));
        assert_eq!(d.head, Head::Custom);
        assert_eq!(d.name, "");
    }
}

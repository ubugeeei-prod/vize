//! What a reader can do about a group of findings: the hint table.
//!
//! A [`Hint`] names one kind of finding several producers report many times
//! over — an attribute a component does not declare, a self-closing HTML
//! element under strict template syntax — and says, once, how to fix the
//! source and which setting changes the behaviour. The digest renderer
//! ([`super::digest`]) prints one block per hint instead of one diagnostic per
//! finding.
//!
//! Every hint is data in [`HINTS`]: the classifier below only picks an entry.
//! Each [`Setting`]'s snippet is the exact text a test writes into that file
//! (`crates/vize/tests/diagnostic_digest_hints_cli.rs`) to prove the finding
//! disappears, and each docs URL is checked against a heading of the page it
//! names, so a hint cannot drift from the option it documents.

/// Where a [`Setting`] goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    /// A member of the object `vite.config.ts` exports (the shared `vue`
    /// section every Vize command reads).
    ViteConfig,
    /// A top-level member of the project's `tsconfig.json`.
    Tsconfig,
    /// Arguments of the command that reported the finding.
    Cli,
}

impl Place {
    /// The name printed before the snippet.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ViteConfig => "vite.config.ts",
            Self::Tsconfig => "tsconfig.json",
            Self::Cli => "command line",
        }
    }
}

/// One way to change the behaviour, as the exact text to write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Setting {
    /// Where the snippet goes.
    pub place: Place,
    /// The text to write there.
    pub snippet: &'static str,
}

/// What to do about one kind of finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hint {
    /// The kind, printed as the block's code: `error[<id>]`.
    pub id: &'static str,
    /// What the findings have in common, after their count.
    pub summary: &'static str,
    /// The representative frame's label; `{subject}` is the authored text
    /// the finding points at.
    pub label: &'static str,
    /// How to fix the source.
    pub fix: &'static str,
    /// What the settings below do, introducing them.
    pub configure: &'static str,
    /// The settings, each verified to remove the finding.
    pub settings: &'static [Setting],
    /// The documentation page and section.
    pub docs: &'static str,
}

/// `id`s, for the classifiers and tests.
pub mod ids {
    /// An attribute passed to a component that neither declares it nor
    /// forwards it to a single root element.
    pub const UNDECLARED_COMPONENT_ATTR: &str = "type/undeclared-component-attr";
    /// An attribute a native HTML or SVG element does not define.
    pub const UNKNOWN_ELEMENT_ATTR: &str = "type/unknown-element-attr";
    /// A listener for an event the component does not declare.
    pub const UNDECLARED_COMPONENT_EVENT: &str = "type/undeclared-component-event";
    /// `<div />` under strict template syntax.
    pub const STRICT_SELF_CLOSING: &str = "template-syntax/self-closing";
}

const UNKNOWN_PROPS_OFF: Setting = Setting {
    place: Place::Tsconfig,
    snippet: r#""vueCompilerOptions": { "checkUnknownProps": false }"#,
};

/// The table.
pub static HINTS: &[Hint] = &[
    Hint {
        id: ids::UNDECLARED_COMPONENT_ATTR,
        summary: "attributes the component neither declares nor forwards",
        label: "`{subject}` is not a prop of this component, and it does not fall through to a single root element",
        fix: "Declare it with `defineProps`, or forward it from a single root element (without `inheritAttrs: false`).",
        configure: "To stop checking attributes a component does not declare:",
        settings: &[UNKNOWN_PROPS_OFF],
        docs: "https://vizejs.dev/guide/configuration-reference#vue-type-resolution",
    },
    Hint {
        id: ids::UNKNOWN_ELEMENT_ATTR,
        summary: "attributes the HTML or SVG element does not define",
        label: "`{subject}` is not an attribute of this element",
        fix: "Use the element's own attribute (check the spelling), or a `data-*` attribute for custom data.",
        configure: "To stop checking attributes an element does not define:",
        settings: &[UNKNOWN_PROPS_OFF],
        docs: "https://vizejs.dev/guide/configuration-reference#vue-type-resolution",
    },
    Hint {
        id: ids::UNDECLARED_COMPONENT_EVENT,
        summary: "listeners for events the component does not declare",
        label: "the component declares no event for `{subject}`",
        fix: "Declare the event with `defineEmits`, or move the listener to the element that emits it.",
        configure: "To stop checking listeners for undeclared events:",
        settings: &[Setting {
            place: Place::Tsconfig,
            snippet: r#""vueCompilerOptions": { "checkUnknownEvents": false }"#,
        }],
        docs: "https://vizejs.dev/guide/configuration-reference#vue-type-resolution",
    },
    Hint {
        id: ids::STRICT_SELF_CLOSING,
        summary: "non-void HTML elements written as self-closing",
        label: "`{subject}` is not valid HTML, and strict template syntax rejects it",
        fix: "Write an explicit end tag: `<div></div>`.",
        configure: "To accept them and compile them as empty elements (the default `standard` mode):",
        settings: &[
            Setting {
                place: Place::ViteConfig,
                snippet: r#"vue: { compiler: { templateSyntax: "standard" } }"#,
            },
            Setting {
                place: Place::Cli,
                snippet: "--template-syntax standard",
            },
        ],
        docs: "https://vizejs.dev/guide/troubleshooting#template-syntax-modes",
    },
];

/// The hint named `id`.
#[must_use]
pub fn hint(id: &str) -> Option<&'static Hint> {
    HINTS.iter().find(|hint| hint.id == id)
}

/// The hint for a type-checker diagnostic reported in a template, from its
/// TypeScript code and message.
///
/// TS2353 is TypeScript's excess-property error. In a template it is reported
/// on the props object Vize's projection builds for one element, so its key
/// is an authored attribute or listener and its type says whose props were
/// checked: `$ComponentCheckProps<…>` for a component, Vue's
/// `…Attributes & ReservedProps` for a native element, and a component's
/// emits for a listener.
#[must_use]
pub fn for_template_type_diagnostic(code: Option<u32>, message: &str) -> Option<&'static Hint> {
    if code != Some(2353) {
        return None;
    }
    let (key, target) = excess_property(message)?;
    let id = if target.contains("$ComponentCheckProps<") {
        ids::UNDECLARED_COMPONENT_ATTR
    } else if target.contains("ReservedProps>") {
        ids::UNKNOWN_ELEMENT_ATTR
    } else if is_listener_key(key) {
        ids::UNDECLARED_COMPONENT_EVENT
    } else {
        return None;
    };
    hint(id)
}

/// The hint for one error a template gate listed (`ErrorCode` variant name and
/// message).
#[must_use]
pub fn for_template_error(code: &str, message: &str) -> Option<&'static Hint> {
    (code == "UnexpectedSolidusInTag"
        && message == "Invalid self-closing syntax on non-void HTML element.")
        .then(|| hint(ids::STRICT_SELF_CLOSING))
        .flatten()
}

/// `(key, type)` of TypeScript's
/// `Object literal may only specify known properties, and 'KEY' does not exist in type 'TYPE'.`
fn excess_property(message: &str) -> Option<(&str, &str)> {
    let rest = message.strip_prefix("Object literal may only specify known properties, and '")?;
    let (key, rest) = rest.split_once("' does not exist in type '")?;
    let target = rest.strip_suffix("'.")?;
    Some((key.trim_matches('"'), target))
}

/// `onClick`, `onUpdate:modelValue`: the prop name a `@event` listener binds.
fn is_listener_key(key: &str) -> bool {
    key.strip_prefix("on")
        .and_then(|event| event.chars().next())
        .is_some_and(|first| first.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::{HINTS, for_template_error, for_template_type_diagnostic, ids};

    fn id(code: u32, message: &str) -> Option<&'static str> {
        for_template_type_diagnostic(Some(code), message).map(|hint| hint.id)
    }

    #[test]
    fn excess_properties_are_classified_by_whose_props_were_checked() {
        assert_eq!(
            id(
                2353,
                "Object literal may only specify known properties, and '\"behavior\"' does not exist in type '$ComponentCheckProps<{}, $PublicComponentAttrs & { abbr?: unknown; }>'."
            ),
            Some(ids::UNDECLARED_COMPONENT_ATTR)
        );
        assert_eq!(
            id(
                2353,
                "Object literal may only specify known properties, and '\"alt\"' does not exist in type 'Partial<HTMLAttributes & ReservedProps>'."
            ),
            Some(ids::UNKNOWN_ELEMENT_ATTR)
        );
        assert_eq!(
            id(
                2353,
                "Object literal may only specify known properties, and '\"onClick\"' does not exist in type '{ onUpload?: unknown; }'."
            ),
            Some(ids::UNDECLARED_COMPONENT_EVENT)
        );
    }

    #[test]
    fn other_diagnostics_keep_their_own_rendering() {
        for (code, message) in [
            (
                2353,
                "Object literal may only specify known properties, and '\"one\"' does not exist in type '{ two: number; }'.",
            ),
            (2353, "Object literal may only specify known properties."),
            (
                2322,
                "Object literal may only specify known properties, and '\"alt\"' does not exist in type 'Partial<HTMLAttributes & ReservedProps>'.",
            ),
        ] {
            assert_eq!(id(code, message), None, "{message}");
        }
        assert_eq!(for_template_type_diagnostic(None, ""), None);
        assert_eq!(
            for_template_error("UnexpectedSolidusInTag", "Something else."),
            None
        );
        assert_eq!(
            for_template_error(
                "UnexpectedSolidusInTag",
                "Invalid self-closing syntax on non-void HTML element."
            )
            .map(|hint| hint.id),
            Some(ids::STRICT_SELF_CLOSING)
        );
    }

    #[test]
    fn every_hint_is_complete_and_unique() {
        for (index, hint) in HINTS.iter().enumerate() {
            assert!(!hint.settings.is_empty(), "{}", hint.id);
            assert_eq!(hint.label.matches("{subject}").count(), 1, "{}", hint.id);
            assert!(
                HINTS
                    .iter()
                    .skip(index + 1)
                    .all(|other| other.id != hint.id),
                "{}",
                hint.id
            );
        }
    }
}

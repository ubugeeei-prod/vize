//! Completion item builders and binding type conversion.
//!
//! Provides helper functions for constructing various kinds of
//! completion items and converting binding types to completion info.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, InsertTextFormat, MarkupContent, MarkupKind,
};
use vize_relief::BindingType;

/// Convert BindingType to completion item information.
pub(crate) fn binding_type_to_completion_info(
    binding_type: BindingType,
) -> (CompletionItemKind, String, String) {
    match binding_type {
        BindingType::SetupRef => (
            CompletionItemKind::VARIABLE,
            " (ref)".to_string(),
            "**Ref**\n\nReactive reference. Auto-unwrapped in template, needs `.value` in script."
                .to_string(),
        ),
        BindingType::SetupMaybeRef => (
            CompletionItemKind::VARIABLE,
            " (maybeRef)".to_string(),
            "**MaybeRef**\n\nPossibly a ref (from toRef/toRefs). Auto-unwrapped in template."
                .to_string(),
        ),
        BindingType::SetupReactiveConst => (
            CompletionItemKind::VARIABLE,
            " (reactive)".to_string(),
            "**Reactive**\n\nReactive object. Direct access without `.value`.".to_string(),
        ),
        BindingType::SetupConst => (
            CompletionItemKind::CONSTANT,
            " (const)".to_string(),
            "**Const**\n\nConstant binding (function, class, or literal).".to_string(),
        ),
        BindingType::SetupLet => (
            CompletionItemKind::VARIABLE,
            " (let)".to_string(),
            "**Let**\n\nMutable variable.".to_string(),
        ),
        BindingType::Props => (
            CompletionItemKind::PROPERTY,
            " (prop)".to_string(),
            "**Prop**\n\nComponent property from defineProps.".to_string(),
        ),
        BindingType::PropsAliased => (
            CompletionItemKind::PROPERTY,
            " (prop alias)".to_string(),
            "**Aliased Prop**\n\nDestructured prop with alias.".to_string(),
        ),
        BindingType::Data => (
            CompletionItemKind::VARIABLE,
            " (data)".to_string(),
            "**Data**\n\nReactive data property (Options API).".to_string(),
        ),
        BindingType::Options => (
            CompletionItemKind::METHOD,
            " (options)".to_string(),
            "**Options**\n\nComputed or method (Options API).".to_string(),
        ),
        BindingType::LiteralConst => (
            CompletionItemKind::CONSTANT,
            " (literal)".to_string(),
            "**Literal**\n\nLiteral constant value.".to_string(),
        ),
        BindingType::ExternalModule => (
            CompletionItemKind::MODULE,
            " (import)".to_string(),
            "**Import**\n\nImported from external module.".to_string(),
        ),
        BindingType::VueGlobal => (
            CompletionItemKind::VARIABLE,
            " (vue)".to_string(),
            "**Vue Global**\n\nVue global ($refs, $emit, etc.).".to_string(),
        ),
        _ => (
            CompletionItemKind::VARIABLE,
            "".to_string(),
            "Binding from script.".to_string(),
        ),
    }
}

/// Create a directive completion item.
#[allow(clippy::disallowed_macros)]
pub(crate) fn directive_item(label: &str, description: &str, snippet: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        detail: Some(description.to_string()),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!(
                "**{}**\n\n{}\n\n[Vue Documentation](https://vuejs.org/api/built-in-directives.html)",
                label, description
            ),
        })),
        ..Default::default()
    }
}

/// Create a @vize: directive completion item.
#[allow(clippy::disallowed_macros)]
pub(crate) fn vize_directive_item(label: &str, snippet: &str, description: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        detail: Some(description.to_string()),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        sort_text: Some(format!("!{}", label)),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!(
                "**`{}`**\n\n{}\n\nUsage: `<!-- {} -->`",
                label, description, label
            ),
        })),
        ..Default::default()
    }
}

/// Create a component completion item.
#[allow(clippy::disallowed_macros)]
pub(crate) fn component_item(label: &str, description: &str, snippet: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::CLASS),
        detail: Some(format!("Vue built-in: {}", description)),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!(
                "**<{}>**\n\n{}\n\n[Vue Documentation](https://vuejs.org/api/built-in-components.html)",
                label, description
            ),
        })),
        ..Default::default()
    }
}

/// Create a snippet completion item.
pub(crate) fn snippet_item(label: &str, description: &str, snippet: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some(description.to_string()),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    }
}

/// Create an API completion item.
#[allow(clippy::disallowed_macros)]
pub(crate) fn api_item(label: &str, signature: &str, description: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        detail: Some(signature.to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!(
                "```typescript\n{}\n```\n\n{}\n\n[Vue Documentation](https://vuejs.org/api/)",
                signature, description
            ),
        })),
        ..Default::default()
    }
}

/// Create a macro completion item.
#[allow(clippy::disallowed_macros)]
pub(crate) fn macro_item(
    label: &str,
    signature: &str,
    description: &str,
    snippet: &str,
) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        detail: Some(format!("Macro: {}", signature)),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!(
                "```typescript\n{}\n```\n\n{}\n\n*Compiler macro - only usable in `<script setup>`*",
                signature, description
            ),
        })),
        ..Default::default()
    }
}

/// Create an import completion item.
pub(crate) fn import_item(label: &str, description: &str, snippet: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::MODULE),
        detail: Some(description.to_string()),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    }
}

/// Create an attribute completion item.
pub(crate) fn attr_item(label: &str, description: &str, snippet: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::PROPERTY),
        detail: Some(description.to_string()),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    }
}

/// Create a CSS completion item.
#[allow(clippy::disallowed_macros)]
pub(crate) fn css_item(
    label: &str,
    signature: &str,
    description: &str,
    snippet: &str,
) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        detail: Some(format!("Vue CSS: {}", signature)),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!(
                "**{}**\n\n{}\n\n[Vue SFC CSS Features](https://vuejs.org/api/sfc-css-features.html)",
                signature, description
            ),
        })),
        ..Default::default()
    }
}

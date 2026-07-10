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
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, InsertTextFormat,
};
use vize_relief::BindingType;

use crate::ide::markup::{self, Markdown};

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
    let example = markup::snippet_for_docs(snippet);
    CompletionItem {
        label: label.to_string(),
        kind: Some(directive_completion_kind(label)),
        detail: Some(description.to_string()),
        label_details: Some(CompletionItemLabelDetails {
            detail: None,
            description: Some("Vue directive".to_string()),
        }),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: Some(
            Markdown::new()
                .title(&format!("`{label}`"))
                .meta("Vue directive")
                .paragraph(description)
                .example(
                    "vue",
                    &format!("<template>\n  <div {example}></div>\n</template>"),
                )
                .docs(
                    "Vue built-in directives",
                    "https://vuejs.org/api/built-in-directives.html",
                )
                .into_documentation(),
        ),
        ..Default::default()
    }
}

fn directive_completion_kind(label: &str) -> CompletionItemKind {
    match label {
        "@" | "v-on" => CompletionItemKind::EVENT,
        ":" | "v-bind" | "v-model" => CompletionItemKind::PROPERTY,
        "#" | "v-slot" => CompletionItemKind::FIELD,
        _ => CompletionItemKind::KEYWORD,
    }
}

/// Create a @vize: directive completion item.
#[allow(clippy::disallowed_macros)]
pub(crate) fn vize_directive_item(label: &str, snippet: &str, description: &str) -> CompletionItem {
    let example = markup::snippet_for_docs(snippet);
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        detail: Some(description.to_string()),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        sort_text: Some(format!("!{}", label)),
        documentation: Some(
            Markdown::new()
                .title(&format!("`{label}`"))
                .meta("Vize comment directive")
                .paragraph(description)
                .example("html", &format!("<!-- {example} -->"))
                .docs(
                    "Vize comment annotations",
                    "https://github.com/ubugeeei-prod/vize/blob/main/docs/content/guide/comment-annotations.md",
                )
                .into_documentation(),
        ),
        ..Default::default()
    }
}

/// Create a component completion item.
#[allow(clippy::disallowed_macros)]
pub(crate) fn component_item(label: &str, description: &str, snippet: &str) -> CompletionItem {
    let example = markup::snippet_for_docs(snippet);
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::CLASS),
        detail: Some(format!("Vue built-in: {}", description)),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: Some(
            Markdown::new()
                .title(&format!("<{label}>"))
                .meta("Vue built-in component")
                .paragraph(description)
                .example("vue", &example)
                .docs(
                    "Vue built-in components",
                    "https://vuejs.org/api/built-in-components.html",
                )
                .into_documentation(),
        ),
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
        documentation: Some(
            Markdown::new()
                .title(label)
                .meta("Vue API")
                .code("typescript", signature)
                .paragraph(description)
                .docs("Vue API", "https://vuejs.org/api/")
                .into_documentation(),
        ),
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
        documentation: Some(
            Markdown::new()
                .title(label)
                .meta("Vue compiler macro")
                .code("typescript", signature)
                .paragraph(description)
                .section("Scope", "Only usable inside `<script setup>`.")
                .into_documentation(),
        ),
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
    let example = markup::snippet_for_docs(snippet);
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::PROPERTY),
        detail: Some(description.to_string()),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: Some(
            Markdown::new()
                .title(label)
                .meta("Template attribute")
                .paragraph(description)
                .example(
                    "vue",
                    &format!("<template>\n  <div {example}></div>\n</template>"),
                )
                .docs(
                    "Vue template syntax",
                    "https://vuejs.org/guide/essentials/template-syntax.html",
                )
                .into_documentation(),
        ),
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
        documentation: Some(
            Markdown::new()
                .title(signature)
                .meta("Vue SFC CSS feature")
                .code("css", snippet)
                .paragraph(description)
                .docs(
                    "Vue SFC CSS features",
                    "https://vuejs.org/api/sfc-css-features.html",
                )
                .into_documentation(),
        ),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{attr_item, directive_item};
    use tower_lsp::lsp_types::Documentation;

    #[test]
    fn directive_completion_docs_include_highlightable_example() {
        let item = directive_item("v-if", "Conditional rendering", "v-if=\"$1\"");
        let doc = markdown_doc(&item.documentation);

        assert!(doc.contains("**Example**"), "got {doc:?}");
        assert!(doc.contains("```vue"), "got {doc:?}");
        assert!(doc.contains("Vue built-in directives"), "got {doc:?}");
    }

    #[test]
    fn attribute_completion_docs_include_template_syntax_link() {
        let item = attr_item("class", "CSS classes", "class=\"$1\"");
        let doc = markdown_doc(&item.documentation);

        assert!(doc.contains("Template attribute"), "got {doc:?}");
        assert!(doc.contains("```vue"), "got {doc:?}");
        assert!(doc.contains("Vue template syntax"), "got {doc:?}");
    }

    fn markdown_doc(doc: &Option<Documentation>) -> &str {
        match doc.as_ref().expect("completion should include docs") {
            Documentation::MarkupContent(content) => &content.value,
            Documentation::String(value) => value,
        }
    }
}

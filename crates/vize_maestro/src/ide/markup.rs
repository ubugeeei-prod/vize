//! Markdown helpers for LSP hover and completion documentation.

use tower_lsp::lsp_types::{Documentation, MarkupContent, MarkupKind};

#[derive(Default)]
pub(crate) struct Markdown {
    sections: Vec<String>,
}

impl Markdown {
    pub(crate) fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    #[expect(
        clippy::disallowed_macros,
        reason = "tower-lsp lsp_types take std String/HashMap values, built with to_string/format!"
    )]
    pub(crate) fn title(mut self, title: &str) -> Self {
        self.sections.push(format!("**{}**", title));
        self
    }

    #[expect(
        clippy::disallowed_macros,
        reason = "tower-lsp lsp_types take std String/HashMap values, built with to_string/format!"
    )]
    pub(crate) fn meta(mut self, text: &str) -> Self {
        self.sections.push(format!("_{}_", text));
        self
    }

    pub(crate) fn paragraph(mut self, text: &str) -> Self {
        self.sections.push(text.to_string());
        self
    }

    pub(crate) fn code(mut self, language: &str, source: &str) -> Self {
        self.sections.push(code_block(language, source));
        self
    }

    #[expect(
        clippy::disallowed_macros,
        reason = "tower-lsp lsp_types take std String/HashMap values, built with to_string/format!"
    )]
    pub(crate) fn section(mut self, heading: &str, body: &str) -> Self {
        self.sections.push(format!("**{}**\n\n{}", heading, body));
        self
    }

    pub(crate) fn example(self, language: &str, source: &str) -> Self {
        self.section("Example", &code_block(language, source))
    }

    pub(crate) fn docs(self, label: &str, url: &str) -> Self {
        self.section("Docs", &link(label, url))
    }

    pub(crate) fn build(self) -> String {
        self.sections.join("\n\n")
    }

    pub(crate) fn into_documentation(self) -> Documentation {
        markdown_documentation(self.build())
    }
}

#[expect(
    clippy::disallowed_macros,
    reason = "tower-lsp lsp_types take std String/HashMap values, built with to_string/format!"
)]
pub(crate) fn code_block(language: &str, source: &str) -> String {
    format!("```{}\n{}\n```", language, source.trim_end())
}

#[expect(
    clippy::disallowed_macros,
    reason = "tower-lsp lsp_types take std String/HashMap values, built with to_string/format!"
)]
pub(crate) fn link(label: &str, url: &str) -> String {
    format!("[{}]({})", label, url)
}

/// Append `text` so that Markdown shows it as written.
///
/// Code spans (text between matching backtick runs) are copied as they are,
/// since their content is already literal. Elsewhere `\\`, `[` and `]` are
/// backslash-escaped, so the text cannot open a link or an image.
pub(crate) fn push_markdown_text(output: &mut String, text: &str) {
    let mut rest = text;
    while let Some(at) = rest.find(['\\', '[', ']', '`']) {
        let (before, from) = rest.split_at(at);
        output.push_str(before);
        if from.starts_with('`') {
            let fence = from.len() - from.trim_start_matches('`').len();
            let (ticks, body) = from.split_at(fence);
            let span_end = body
                .match_indices(ticks)
                .find(|&(end, _)| {
                    !body.get(..end).unwrap_or_default().ends_with('`')
                        && !body.get(end + fence..).unwrap_or_default().starts_with('`')
                })
                .map(|(end, _)| end + fence);
            let Some(span_end) = span_end else {
                // An unclosed run is literal text: keep it and go on.
                output.push_str(ticks);
                rest = body;
                continue;
            };
            let (span, after) = from.split_at(fence + span_end);
            output.push_str(span);
            rest = after;
        } else {
            output.push('\\');
            let (ch, after) = from.split_at(1);
            output.push_str(ch);
            rest = after;
        }
    }
    output.push_str(rest);
}

pub(crate) fn markdown_content(value: String) -> MarkupContent {
    MarkupContent {
        kind: MarkupKind::Markdown,
        value,
    }
}

pub(crate) fn markdown_documentation(value: String) -> Documentation {
    Documentation::MarkupContent(markdown_content(value))
}

pub(crate) fn snippet_for_docs(snippet: &str) -> String {
    let mut output = String::with_capacity(snippet.len());
    let mut rest = snippet;

    while let Some(ch) = rest.chars().next() {
        if let Some(stripped) = rest.strip_prefix('$') {
            let mut after_digit = stripped.chars();
            if let Some(digit) = after_digit.next().filter(char::is_ascii_digit) {
                if digit != '0' {
                    output.push_str("...");
                }
                rest = after_digit.as_str();
                continue;
            }
            if let Some((placeholder, after)) = stripped
                .strip_prefix('{')
                .and_then(|body| body.split_once('}'))
            {
                if let Some((_, default)) = placeholder.split_once(':') {
                    output.push_str(default);
                } else if !placeholder.starts_with('0') {
                    output.push_str("...");
                }
                rest = after;
                continue;
            }
        }

        output.push(ch);
        rest = rest.get(ch.len_utf8()..).unwrap_or_default();
    }

    let trimmed = output.trim();
    if trimmed.is_empty() {
        "...".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{Markdown, push_markdown_text, snippet_for_docs};

    #[test]
    fn snippet_for_docs_keeps_examples_readable() {
        assert_eq!(
            snippet_for_docs(r#"v-for="$1 in $2" :key="${3:item.id}""#),
            r#"v-for="... in ..." :key="item.id""#
        );
    }

    #[test]
    fn markdown_builder_emits_fenced_examples_and_docs() {
        let doc = Markdown::new()
            .title("v-if")
            .example("vue", "<div v-if=\"ready\" />")
            .docs("Vue built-in directives", "https://vuejs.org/api/")
            .build();

        assert!(doc.contains("**Example**"));
        assert!(doc.contains("```vue"));
        assert!(doc.contains("**Docs**"));
    }

    fn shown(text: &str) -> String {
        let mut output = String::new();
        push_markdown_text(&mut output, text);
        output
    }

    #[test]
    fn markdown_text_keeps_brackets_and_tags_as_written() {
        assert_eq!(
            shown("Rename [x](https://example.com) and ![a](b.png)"),
            "Rename \\[x\\](https://example.com) and !\\[a\\](b.png)"
        );
        assert_eq!(shown("Wrap it in <template>"), "Wrap it in <template>");
        assert_eq!(shown(r"C:\dir"), r"C:\\dir");
    }

    #[test]
    fn markdown_text_copies_code_spans() {
        assert_eq!(shown("Use `<a [x]>` here"), "Use `<a [x]>` here");
        assert_eq!(shown("A ``b ` [c]`` d"), "A ``b ` [c]`` d");
        assert_eq!(shown("open ` [x]"), "open ` \\[x\\]");
        assert_eq!(shown("a `b`` [c]` d"), "a `b`` [c]` d");
        assert_eq!(shown("plain text"), "plain text");
    }
}

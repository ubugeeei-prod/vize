use tower_lsp::lsp_types::{Hover, HoverContents, Range};

use crate::ide::markup;

/// Hover content builder for creating rich hover information.
pub struct HoverBuilder {
    sections: Vec<String>,
}

impl HoverBuilder {
    /// Create a new hover builder.
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    /// Add a title.
    #[allow(clippy::disallowed_macros)]
    pub fn title(mut self, title: &str) -> Self {
        self.sections.push(format!("**{}**", title));
        self
    }

    /// Add compact metadata under the title.
    #[allow(clippy::disallowed_macros)]
    pub fn meta(mut self, text: &str) -> Self {
        self.sections.push(format!("_{}_", text));
        self
    }

    /// Add a code block.
    #[allow(clippy::disallowed_macros)]
    pub fn code(mut self, language: &str, code: &str) -> Self {
        self.sections.push(markup::code_block(language, code));
        self
    }

    /// Add a syntax-highlightable example block.
    #[allow(clippy::disallowed_macros)]
    pub fn example(mut self, language: &str, code: &str) -> Self {
        self.sections.push(format!(
            "**Example**\n\n{}",
            markup::code_block(language, code)
        ));
        self
    }

    /// Add a named text section.
    #[allow(clippy::disallowed_macros)]
    pub fn section(mut self, heading: &str, text: &str) -> Self {
        self.sections.push(format!("**{}**\n\n{}", heading, text));
        self
    }

    /// Add a named bullet list.
    #[allow(clippy::disallowed_macros)]
    pub fn bullets(mut self, heading: &str, items: &[&str]) -> Self {
        if items.is_empty() {
            return self;
        }

        let body = items
            .iter()
            .map(|item| format!("- {}", item))
            .collect::<Vec<_>>()
            .join("\n");
        self.sections.push(format!("**{}**\n{}", heading, body));
        self
    }

    /// Add a description.
    pub fn description(mut self, text: &str) -> Self {
        self.sections.push(text.to_string());
        self
    }

    /// Add a documentation link.
    #[allow(clippy::disallowed_macros)]
    pub fn link(mut self, text: &str, url: &str) -> Self {
        self.sections.push(markup::link(text, url));
        self
    }

    /// Add a named documentation link section.
    #[allow(clippy::disallowed_macros)]
    pub fn docs(mut self, text: &str, url: &str) -> Self {
        self.sections
            .push(format!("**Docs**\n\n{}", markup::link(text, url)));
        self
    }

    /// Build the hover.
    pub fn build(self) -> Hover {
        Hover {
            contents: HoverContents::Markup(markup::markdown_content(self.sections.join("\n\n"))),
            range: None,
        }
    }

    /// Build the hover with a range.
    pub fn build_with_range(self, range: Range) -> Hover {
        Hover {
            contents: HoverContents::Markup(markup::markdown_content(self.sections.join("\n\n"))),
            range: Some(range),
        }
    }
}

impl Default for HoverBuilder {
    fn default() -> Self {
        Self::new()
    }
}

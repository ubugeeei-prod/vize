//! JSDoc recovery for native member quick info.
//!
//! TypeScript 7.0 omits a member's JSDoc when the member is read through a
//! mapped type (`Readonly<Props>`, `$PublicSlots<Slots>`), which is how Vize
//! projects props, emits and slots. 7.1 keeps it. Follow the checker's own
//! declarations so both backends show the authored documentation.

use vize_canon::{CorsaBridge, LspHover, LspHoverContents};

/// Declarations tried per hover; unions and overloads rarely need more.
const DECLARATION_LIMIT: usize = 8;

/// Append the first declaration's JSDoc when `hover` has none.
pub(in crate::ide::hover) async fn adopt_declaration_documentation(
    bridge: &CorsaBridge,
    request_uri: &str,
    (line, character): (u32, u32),
    hover: &mut LspHover,
) {
    if documentation(hover).is_some() {
        return;
    }
    let Ok(definitions) = bridge.definition(request_uri, line, character).await else {
        return;
    };
    for definition in definitions.into_iter().take(DECLARATION_LIMIT) {
        let Ok(Some(origin)) = bridge
            .hover(
                &definition.uri,
                definition.range.start.line,
                definition.range.start.character,
            )
            .await
        else {
            continue;
        };
        if let Some(documentation) = documentation(&origin)
            && let LspHoverContents::Markup(content) = &mut hover.contents
        {
            content.value.truncate(content.value.trim_end().len());
            content.value.push_str("\n\n");
            content.value.push_str(documentation);
            return;
        }
    }
}

/// Property and method quick info, the members a mapped type can rewrap.
pub(in crate::ide::hover) fn is_member_quick_info(hover: &LspHover) -> bool {
    let LspHoverContents::Markup(content) = &hover.contents else {
        return false;
    };
    content.kind == "markdown"
        && content
            .value
            .strip_prefix("```typescript\n")
            .is_some_and(|code| code.starts_with("(property) ") || code.starts_with("(method) "))
}

pub(in crate::ide::hover) fn documentation(hover: &LspHover) -> Option<&str> {
    let LspHoverContents::Markup(content) = &hover.contents else {
        return None;
    };
    if content.kind != "markdown" {
        return None;
    }
    let (_, documentation) = content
        .value
        .strip_prefix("```typescript\n")?
        .split_once("\n```")?;
    let documentation = documentation.trim();
    (!documentation.is_empty()).then_some(documentation)
}

#[cfg(test)]
mod tests {
    use vize_canon::{LspHover, LspHoverContents, LspMarkupContent};

    fn markup(value: &str) -> LspHover {
        LspHover {
            contents: LspHoverContents::Markup(LspMarkupContent {
                kind: "markdown".into(),
                value: value.into(),
            }),
            range: None,
        }
    }

    #[test]
    fn only_property_and_method_quick_info_follow_declarations() {
        for (value, member) in [
            ("```typescript\n(property) title: string\n```", true),
            ("```typescript\n(method) item(props: {}): any\n```", true),
            ("```typescript\nconst title: string\n```", false),
            ("```typescript\n(parameter) row: number\n```", false),
            ("(property) title: string", false),
        ] {
            assert_eq!(
                super::is_member_quick_info(&markup(value)),
                member,
                "{value}"
            );
        }
    }

    #[test]
    fn documentation_is_the_text_after_the_signature_fence() {
        assert_eq!(
            super::documentation(&markup(
                "```typescript\n(property) title: string\n```\nThe title shown above.\n"
            )),
            Some("The title shown above.")
        );
        assert_eq!(
            super::documentation(&markup("```typescript\n(property) title: string\n```\n")),
            None
        );
    }
}

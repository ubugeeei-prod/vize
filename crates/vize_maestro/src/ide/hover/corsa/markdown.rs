//! Markdown shaping for hovers answered through the Corsa bridge.

use super::HoverService;

impl HoverService {
    /// Wrap TypeScript type information in a code block for proper markdown rendering.
    pub(in crate::ide::hover) fn wrap_type_info_in_codeblock(text: &str) -> String {
        let text = text.trim();
        // If already wrapped in code block, return as-is
        if text.starts_with("```") {
            return text.to_string();
        }
        // Check if this looks like TypeScript type info
        // Common patterns: (const), (let), (var), (function), (method), (property), type, interface, etc.
        let looks_like_type_info = is_bare_type(text)
            || text.starts_with('(')
            || text.starts_with("type ")
            || text.starts_with("interface ")
            || text.starts_with("class ")
            || text.starts_with("enum ")
            || text.starts_with("function ")
            || text.starts_with("const ")
            || text.starts_with("let ")
            || text.starts_with("var ")
            || text.starts_with("import ")
            || text.contains(": ")
            || text.contains("=>")
            || text.contains(" | ")
            || text.contains(" & ");

        if looks_like_type_info {
            #[expect(
                clippy::disallowed_macros,
                reason = "`format!` builds the std `String` values tower_lsp::lsp_types payloads take"
            )]
            {
                format!("```typescript\n{}\n```", text)
            }
        } else {
            text.to_string()
        }
    }

    /// The hover body is the signature itself. The former
    /// `**TypeScript quick info**` / `_Resolved through Vize virtual
    /// TypeScript_` preamble named an implementation detail no user acts on
    /// and pushed the signature below the fold in small popups (#3894);
    /// Volar and tsserver open with the code block.
    pub(super) fn decorate_corsa_hover_markdown(value: &str) -> String {
        let value = value.trim();
        if let Some(authored) = authored_setup_type_parameter(value) {
            return authored;
        }
        if is_bare_type(value) {
            return Self::wrap_type_info_in_codeblock(value);
        }
        value.to_string()
    }
}

/// TypeScript names the canonical setup function as the owner of a generic
/// parameter and prints its generated return type. Keep the authored part of
/// that quick info while retaining any documentation after the code fence.
fn authored_setup_type_parameter(value: &str) -> Option<String> {
    let fenced = value.strip_prefix("```typescript\n")?;
    let (code, documentation) = fenced.split_once("\n```")?;
    let owner = setup_owner_delimiter(code)?;
    let authored = code.get(..owner)?;
    let generated = code.get(owner + " in __setup<".len()..)?;
    if !authored.starts_with("(type parameter) ")
        || !generated.contains("__vize_setup_props: $SetupProps;")
    {
        return None;
    }

    let mut result = String::with_capacity(value.len());
    result.push_str("```typescript\n");
    result.push_str(authored);
    result.push_str("\n```");
    result.push_str(documentation);
    Some(result)
}

/// Find the generated owner outside authored TypeScript string literals. A
/// constraint may itself contain the delimiter text, and the generated setup
/// signature repeats that constraint after the real owner.
fn setup_owner_delimiter(code: &str) -> Option<usize> {
    const MARKER: &[u8] = b" in __setup<";
    let bytes = code.as_bytes();
    let mut quote = None;
    let mut escaped = false;
    let mut parentheses = 0_u32;
    let mut brackets = 0_u32;
    let mut braces = 0_u32;
    let mut angles = 0_u32;
    let mut offset = 0;

    while offset < bytes.len() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if bytes[offset] == b'\\' {
                escaped = true;
            } else if bytes[offset] == delimiter {
                quote = None;
            }
            offset += 1;
            continue;
        }
        if parentheses == 0
            && brackets == 0
            && braces == 0
            && angles == 0
            && bytes[offset..].starts_with(MARKER)
        {
            return Some(offset);
        }
        match bytes[offset] {
            delimiter @ (b'\'' | b'"' | b'`') => quote = Some(delimiter),
            b'(' => parentheses += 1,
            b')' => parentheses = parentheses.saturating_sub(1),
            b'[' => brackets += 1,
            b']' => brackets = brackets.saturating_sub(1),
            b'{' => braces += 1,
            b'}' => braces = braces.saturating_sub(1),
            b'<' => angles += 1,
            b'>' => angles = angles.saturating_sub(1),
            _ => {}
        }
        offset += 1;
    }
    None
}

fn is_bare_type(text: &str) -> bool {
    matches!(
        text,
        "string" | "number" | "boolean" | "unknown" | "any" | "void" | "never" | "object"
    )
}

#[cfg(test)]
mod tests {
    use super::HoverService;

    #[test]
    fn bare_template_type_gets_a_typescript_fence() {
        assert_eq!(
            HoverService::decorate_corsa_hover_markdown("number"),
            "```typescript\nnumber\n```"
        );
        assert_eq!(
            HoverService::wrap_type_info_in_codeblock("string"),
            "```typescript\nstring\n```"
        );
        assert_eq!(
            HoverService::decorate_corsa_hover_markdown("A component description."),
            "A component description."
        );
    }

    #[test]
    fn setup_type_parameter_hides_generated_owner_and_return_type() {
        let native = "```typescript\n(type parameter) T extends Item in __setup<T extends Item, K extends keyof T>(): {\n    __vize_setup_props: $SetupProps;\n}\n```\n\nThe item type.";
        assert_eq!(
            HoverService::decorate_corsa_hover_markdown(native),
            "```typescript\n(type parameter) T extends Item\n```\n\nThe item type."
        );
    }

    #[test]
    fn setup_owner_text_inside_a_constraint_remains_authored() {
        let native = "```typescript\n(type parameter) T extends \"before in __setup<after\" in __setup<T extends \"before in __setup<after\">(): {\n    __vize_setup_props: $SetupProps;\n}\n```";
        assert_eq!(
            HoverService::decorate_corsa_hover_markdown(native),
            "```typescript\n(type parameter) T extends \"before in __setup<after\"\n```"
        );
    }

    #[test]
    fn nested_setup_owner_text_inside_a_constraint_remains_authored() {
        let native = "```typescript\n(type parameter) T extends { [K in __setup<string>]: K } in __setup<T extends { [K in __setup<string>]: K }>(): {\n    __vize_setup_props: $SetupProps;\n}\n```";
        assert_eq!(
            HoverService::decorate_corsa_hover_markdown(native),
            "```typescript\n(type parameter) T extends { [K in __setup<string>]: K }\n```"
        );
    }

    #[test]
    fn authored_setup_function_hover_is_not_rewritten() {
        let native = "```typescript\n(type parameter) T in __setup<T>(): void\n```";
        assert_eq!(HoverService::decorate_corsa_hover_markdown(native), native);
    }
}

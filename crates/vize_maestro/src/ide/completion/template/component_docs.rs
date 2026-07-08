//! Markdown docs for component prop and slot completions.
#![allow(clippy::disallowed_macros)]

use tower_lsp::lsp_types::Documentation;

use crate::ide::markup::Markdown;

pub(super) fn prop_documentation(
    name: &str,
    type_detail: &str,
    required: bool,
    default_value: Option<&str>,
    insert_name: &str,
    dynamic: bool,
) -> Documentation {
    let requirement = if required { "Required" } else { "Optional" };
    let mut doc = Markdown::new()
        .title(&format!("Prop `{name}`"))
        .meta(&format!("{requirement} component prop"))
        .code(
            "typescript",
            &format!("interface Props {{\n  {name}: {type_detail}\n}}"),
        )
        .section("Requirement", requirement)
        .example(
            "vue",
            &prop_example(name, type_detail, insert_name, dynamic),
        )
        .docs(
            "Vue Component Props",
            "https://vuejs.org/guide/components/props.html",
        );

    if let Some(default) = default_value {
        doc = doc.section("Default", &format!("`{default}`"));
    }

    doc.into_documentation()
}

pub(super) fn slot_documentation(
    name: &str,
    props_type: &str,
    prop_names: Option<&[String]>,
) -> Documentation {
    let destructure = prop_names
        .filter(|names| !names.is_empty())
        .map(|names| names.join(", "))
        .unwrap_or_else(|| "slotProps".to_string());

    Markdown::new()
        .title(&format!("Slot `{name}`"))
        .meta("Component slot")
        .code("typescript", props_type)
        .example(
            "vue",
            &format!(
                "<template #{name}=\"{{ {destructure} }}\">\n  <!-- slot content -->\n</template>"
            ),
        )
        .docs("Vue Slots", "https://vuejs.org/guide/components/slots.html")
        .into_documentation()
}

fn prop_example(name: &str, type_detail: &str, insert_name: &str, dynamic: bool) -> String {
    if dynamic {
        return format!("<Component :{name}=\"value\" />");
    }
    if type_detail == "boolean" {
        return format!("<Component {insert_name} />\n<Component :{name}=\"value\" />");
    }
    format!("<Component {insert_name}=\"...\" />\n<Component :{name}=\"value\" />")
}

#[cfg(test)]
mod tests {
    use super::prop_documentation;
    use tower_lsp::lsp_types::Documentation;

    #[test]
    fn prop_docs_are_markdown_with_vue_examples() {
        let doc = prop_documentation("modelValue", "string", true, None, "model-value", false);
        let Documentation::MarkupContent(content) = doc else {
            panic!("expected markdown docs");
        };

        assert!(content.value.contains("```typescript"));
        assert!(content.value.contains("```vue"));
        assert!(content.value.contains("Vue Component Props"));
    }
}

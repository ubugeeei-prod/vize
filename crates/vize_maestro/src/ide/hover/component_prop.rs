use tower_lsp::lsp_types::Hover;

use super::HoverBuilder;
use crate::ide::IdeContext;
use crate::ide::completion::template::component_metadata;

pub(super) fn hover_attribute(ctx: &IdeContext<'_>) -> Option<Hover> {
    let (attr_name, component_name) =
        crate::ide::definition::helpers::get_attribute_and_component_at_offset(ctx)?;
    if !crate::ide::is_component_tag(&component_name) {
        return None;
    }

    let prop_name = crate::ide::definition::helpers::kebab_to_camel(&attr_name);
    if let Some(metadata) = component_metadata(ctx, &component_name)
        && let Some(prop) = metadata.props.iter().find(|prop| prop.name == prop_name)
    {
        let type_detail = prop.type_detail.as_deref().unwrap_or("unknown");
        let optional = if prop.required { "" } else { "?" };
        let signature = format!("{}{}: {}", prop.name, optional, type_detail);
        let requirement = if prop.required {
            "Required"
        } else {
            "Optional"
        };
        let mut builder = HoverBuilder::new()
            .title(&prop.name)
            .meta("Component prop")
            .code("typescript", &signature)
            .section("Requirement", requirement)
            .example(
                "vue",
                &format!(
                    "<{component_name} {attr_name}=\"...\" />\n<{component_name} :{attr_name}=\"value\" />"
                ),
            )
            .docs(
                "Vue Component Props",
                "https://vuejs.org/guide/components/props.html",
            );

        if let Some(default) = prop.default_value.as_deref() {
            builder = builder.section("Default", &format!("`{default}`"));
        }

        return Some(builder.build());
    }

    let import_path = crate::ide::definition::helpers::find_import_path(ctx, &component_name)?;
    let resolved_path =
        crate::ide::definition::helpers::resolve_import_path(ctx.uri, &import_path)?;
    let component_content = std::fs::read_to_string(&resolved_path).ok()?;
    let descriptor = vize_atelier_sfc::parse_sfc(
        &component_content,
        vize_atelier_sfc::SfcParseOptions {
            filename: resolved_path.to_string_lossy().to_string().into(),
            ..Default::default()
        },
    )
    .ok()?;
    let script_setup = descriptor.script_setup.as_ref()?;
    let script = script_setup.content.as_ref();
    let define_props_pos = script.find("defineProps")?;
    let after_define_props = &script[define_props_pos..];
    let prop_pos =
        crate::ide::definition::helpers::find_prop_in_define_props(after_define_props, &prop_name)?;
    let signature = prop_signature_at(script, define_props_pos + prop_pos, &prop_name);

    Some(
        HoverBuilder::new()
            .title(&prop_name)
            .meta("Component prop")
            .code("typescript", &signature)
            .example("vue", &format!("<{component_name} {attr_name}=\"...\" />"))
            .docs(
                "Vue Component Props",
                "https://vuejs.org/guide/components/props.html",
            )
            .build(),
    )
}

pub(super) fn hover_event(ctx: &IdeContext<'_>) -> Option<Hover> {
    let contract = crate::ide::definition::component_event::contract(ctx)?;
    let signature = format!("{}: {}", contract.name, contract.payload_type);

    Some(
        HoverBuilder::new()
            .title(&format!("@{}", contract.name))
            .meta("Component event")
            .code("typescript", &signature)
            .section("Payload", &format!("`{}`", contract.payload_type))
            .example("vue", &format!("@{}=\"handler\"", contract.name))
            .docs(
                "Vue Component Events",
                "https://vuejs.org/guide/components/events.html",
            )
            .build_with_range(contract.authored_range),
    )
}

fn prop_signature_at(script: &str, offset: usize, prop_name: &str) -> String {
    let line_start = script[..offset]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0);
    let line_end = script[offset..]
        .find('\n')
        .map(|index| offset + index)
        .unwrap_or(script.len());
    let line = script[line_start..line_end]
        .trim()
        .trim_end_matches(',')
        .trim();
    if line.is_empty() {
        prop_name.to_string()
    } else {
        line.to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::super::HoverService;
    use crate::{ide::IdeContext, server::ServerState};
    use tower_lsp::lsp_types::{HoverContents, Url};

    #[test]
    fn hover_component_prop_uses_croquis_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let child_path = dir.path().join("Child.vue");
        let parent_path = dir.path().join("Parent.vue");

        fs::write(
            &child_path,
            r#"<script setup lang="ts">
defineModel<string>({ required: true })
</script>
"#,
        )
        .unwrap();
        let source = r#"<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child model-value="draft" />
</template>
"#;
        fs::write(&parent_path, source).unwrap();

        let uri = Url::from_file_path(&parent_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.find("model-value").unwrap() + "model-value".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let hover = HoverService::hover(&ctx).unwrap();
        let value = hover_markdown(hover);

        assert!(value.contains("modelValue: string"), "got {value:?}");
        assert!(value.contains("Required"), "got {value:?}");
        assert!(
            value.contains("<Child model-value=\"...\" />"),
            "got {value:?}"
        );
        assert!(value.contains("Vue Component Props"), "got {value:?}");
        assert!(value.contains("**Example**"), "got {value:?}");
        assert!(value.contains("```vue"), "got {value:?}");
    }

    fn hover_markdown(hover: tower_lsp::lsp_types::Hover) -> String {
        match hover.contents {
            HoverContents::Markup(content) => content.value,
            HoverContents::Scalar(marked) => match marked {
                tower_lsp::lsp_types::MarkedString::String(value) => value,
                tower_lsp::lsp_types::MarkedString::LanguageString(value) => value.value,
            },
            HoverContents::Array(items) => items
                .into_iter()
                .map(|item| match item {
                    tower_lsp::lsp_types::MarkedString::String(value) => value,
                    tower_lsp::lsp_types::MarkedString::LanguageString(value) => value.value,
                })
                .collect::<Vec<_>>()
                .join("\n\n"),
        }
    }
}

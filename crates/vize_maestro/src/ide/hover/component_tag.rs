use tower_lsp::lsp_types::Hover;
use vize_croquis::croquis::{ComponentUsage, PassedProp};
use vize_croquis::{Drawer, DrawerOptions};

mod items;

use super::{HoverBuilder, HoverService};
use crate::ide::completion::template::component_metadata;
use crate::ide::definition::helpers;
use crate::ide::{IdeContext, is_component_tag};
use crate::virtual_code::{ArtCursorPosition, BlockType};
use items::{event_items, prop_items, slot_items};

impl HoverService {
    pub(super) fn hover_component_tag(ctx: &IdeContext<'_>) -> Option<Hover> {
        let tag_name = helpers::get_tag_at_offset(&ctx.content, ctx.offset)?;
        if !is_component_tag(&tag_name) {
            return None;
        }

        let metadata = component_metadata(ctx, &tag_name);
        if metadata.is_none() && !starts_with_uppercase(&tag_name) {
            return None;
        }

        let usage = component_usage_at_cursor(ctx, &tag_name)?;
        let mut builder = HoverBuilder::new().title(&tag_name).meta("Component usage");

        let props = prop_items(&usage);
        if !props.is_empty() {
            builder = section_from_items(builder, "Passed props", &props);
        }

        let events = event_items(&usage);
        if !events.is_empty() {
            builder = section_from_items(builder, "Listeners", &events);
        }

        let slots = slot_items(&usage);
        if !slots.is_empty() {
            builder = section_from_items(builder, "Slots", &slots);
        }

        if usage.has_spread_attrs {
            builder = builder.section("Spread attrs", "`v-bind` receives an object expression.");
        }

        if let Some(guard) = usage.vif_guard.as_ref() {
            builder = builder.section("Rendered when", &format!("`{}`", guard.as_str()));
        }

        if let Some(metadata) = metadata {
            let has_dynamic_prop_name = usage.props.iter().any(|prop| prop.name_is_dynamic);
            let missing_required = metadata
                .props
                .iter()
                .filter(|prop| prop.required)
                .filter(|prop| {
                    !usage
                        .props
                        .iter()
                        .any(|passed| prop_names_match(passed, prop.name.as_str()))
                })
                .map(|prop| prop.name.clone())
                .collect::<Vec<_>>();
            if !missing_required.is_empty() {
                let heading = if usage.has_spread_attrs || has_dynamic_prop_name {
                    "Required props not statically visible"
                } else {
                    "Required props not passed"
                };
                builder = section_from_items(builder, heading, &missing_required);
                if !usage.has_spread_attrs && !has_dynamic_prop_name {
                    builder = builder.code(
                        "vue",
                        &missing_required_example(tag_name.as_str(), &missing_required),
                    );
                }
            }

            builder = builder.link(
                "Vue Component Props",
                "https://vuejs.org/guide/components/props.html",
            );
        }

        Some(builder.build())
    }
}

fn component_usage_at_cursor(ctx: &IdeContext<'_>, tag_name: &str) -> Option<ComponentUsage> {
    let template = template_view(ctx)?;
    let allocator = vize_s0::Allocator::new();
    let (root, _) = if template.is_document {
        vize_armature::parse_document(&allocator, template.content)
    } else {
        vize_armature::parse(&allocator, template.content)
    };
    let mut drawer = Drawer::with_options(DrawerOptions {
        analyze_template_scopes: true,
        track_usage: true,
        ..Default::default()
    });
    drawer.draw_template(&root);
    let croquis = drawer.finish();

    croquis
        .component_usages
        .iter()
        .filter(|usage| {
            usage.name == tag_name
                && usage.start <= template.relative_offset
                && template.relative_offset <= usage.end
        })
        .min_by_key(|usage| usage.end.saturating_sub(usage.start))
        .cloned()
}

struct TemplateView<'a> {
    content: &'a str,
    relative_offset: u32,
    is_document: bool,
}

fn template_view<'a>(ctx: &'a IdeContext<'_>) -> Option<TemplateView<'a>> {
    match ctx.block_type? {
        BlockType::Template if crate::utils::is_standalone_html_path(ctx.uri.path()) => {
            Some(TemplateView {
                content: &ctx.content,
                relative_offset: ctx.offset.min(ctx.content.len()) as u32,
                is_document: true,
            })
        }
        BlockType::Template => {
            let descriptor = vize_atelier_sfc::parse_sfc(
                &ctx.content,
                vize_atelier_sfc::SfcParseOptions {
                    filename: ctx.uri.path().to_string().into(),
                    ..Default::default()
                },
            )
            .ok()?;
            let template = descriptor.template?;
            Some(TemplateView {
                content: ctx.content.get(template.loc.start..template.loc.end)?,
                relative_offset: ctx.offset.saturating_sub(template.loc.start) as u32,
                is_document: false,
            })
        }
        BlockType::Art(ArtCursorPosition::VariantTemplate(info)) => Some(TemplateView {
            content: ctx.content.get(info.template_start..info.template_end)?,
            relative_offset: info.relative_offset as u32,
            is_document: false,
        }),
        _ => None,
    }
}

fn section_from_items(builder: HoverBuilder, heading: &str, items: &[String]) -> HoverBuilder {
    let refs = items.iter().map(String::as_str).collect::<Vec<_>>();
    builder.bullets(heading, &refs)
}

fn prop_names_match(passed: &PassedProp, declared_name: &str) -> bool {
    !passed.name_is_dynamic
        && (passed.name == declared_name
            || helpers::kebab_to_camel(passed.name.as_str()) == declared_name)
}

fn missing_required_example(tag_name: &str, missing_required: &[String]) -> String {
    let attrs = missing_required
        .iter()
        .map(|name| format!(" :{}=\"...\"", crate::ide::pascal_to_kebab(name)))
        .collect::<String>();
    format!("<{tag_name}{attrs} />")
}

fn starts_with_uppercase(tag_name: &str) -> bool {
    tag_name
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::HoverService;
    use crate::{ide::IdeContext, server::ServerState};
    use tower_lsp::lsp_types::{HoverContents, Url};

    #[test]
    fn hover_component_tag_uses_croquis_usage() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("Parent.vue");
        let source = r#"<script setup lang="ts">
const msg = 'hello'
function save() {}
</script>

<template>
  <Child :message="msg" @save.once="save">
    <template #item="{ row, index }">
      <span>{{ row }}</span>
    </template>
    <span>fallback</span>
  </Child>
</template>
"#;
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.find("<Child").unwrap() + "<Ch".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let hover = HoverService::hover(&ctx).unwrap();
        let value = hover_markdown(hover);

        assert!(value.contains("Component usage"), "got {value:?}");
        assert!(value.contains(":message=\"msg\""), "got {value:?}");
        assert!(value.contains("@save.once=\"save\""), "got {value:?}");
        assert!(value.contains("#item { row, index }"), "got {value:?}");
        assert!(value.contains("#default"), "got {value:?}");
    }

    #[test]
    fn hover_component_tag_does_not_overstate_missing_props_with_spread_attrs() {
        let dir = tempfile::tempdir().unwrap();
        let child_path = dir.path().join("Child.vue");
        fs::write(
            &child_path,
            r#"<script setup lang="ts">
defineProps<{ requiredMessage: string }>()
</script>
<template><div /></template>
"#,
        )
        .unwrap();

        let source_path = dir.path().join("Parent.vue");
        let source = r#"<script setup lang="ts">
import Child from './Child.vue'
const attrs = {}
</script>

<template>
  <Child v-bind="attrs" />
</template>
"#;
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.find("<Child").unwrap() + "<Ch".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let hover = HoverService::hover(&ctx).unwrap();
        let value = hover_markdown(hover);

        assert!(value.contains("Spread attrs"), "got {value:?}");
        assert!(
            value.contains("Required props not statically visible"),
            "got {value:?}"
        );
        assert!(
            !value.contains("Required props not passed"),
            "got {value:?}"
        );
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

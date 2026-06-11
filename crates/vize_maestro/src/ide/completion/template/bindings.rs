//! Analyzed template binding completions, scope-local bindings, CSS module
//! class completions, and generic template snippets.

use std::collections::BTreeSet;

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, Documentation, MarkupContent,
    MarkupKind,
};
use vize_atelier_sfc::croquis::{
    SfcCroquisOptions, analyze_sfc_descriptor, analyze_sfc_descriptor_with_context_legacy_vue2,
    analyze_sfc_descriptor_with_context_options_api,
};
use vize_croquis::ScopeKind;
use vize_relief::BindingType;

use crate::ide::IdeContext;
use crate::ide::completion::items;

/// `<style module>` populates a template-scope `$style` object whose
/// properties are the declared class names. When the cursor sits at
/// `$style.|` we surface those names instead of the usual directive list.
pub(crate) fn css_module_class_completions(ctx: &IdeContext) -> Option<Vec<CompletionItem>> {
    let before = &ctx.content[..ctx.offset.min(ctx.content.len())];
    let trimmed = before.trim_end_matches([' ', '\t']);
    if !trimmed.ends_with("$style.") {
        return None;
    }
    let descriptor = vize_atelier_sfc::parse_sfc(
        &ctx.content,
        vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        },
    )
    .ok()?;
    let mut classes = BTreeSet::new();
    for style in descriptor.styles.iter() {
        let attrs = &style.attrs;
        if !attrs.contains_key("module") {
            continue;
        }
        for class in extract_css_class_names(&style.content) {
            classes.insert(class);
        }
    }
    if classes.is_empty() {
        return None;
    }
    Some(
        classes
            .into_iter()
            .map(|name| {
                #[allow(clippy::disallowed_macros)]
                CompletionItem {
                    label: name.clone(),
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some("CSS module class".to_string()),
                    documentation: Some(Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("`{name}` from `<style module>`."),
                    })),
                    sort_text: Some(format!("0{name}")),
                    ..Default::default()
                }
            })
            .collect(),
    )
}

/// Extract class selector names from raw CSS content. Approximate but good
/// enough for completion — matches `.identifier` outside attribute selectors.
fn extract_css_class_names(css: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = css.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'.' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && is_class_ident_byte(bytes[end]) {
                end += 1;
            }
            if end > start {
                out.push(css[start..end].to_string());
                i = end;
                continue;
            }
        }
        i += 1;
    }
    out.sort();
    out.dedup();
    out
}

fn is_class_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

pub(crate) fn analyzed_template_binding_completions(
    ctx: &IdeContext,
    include_vue3_details: bool,
) -> Vec<CompletionItem> {
    // petite-vue standalone HTML documents have no SFC `<template>` block, so
    // `parse_sfc` yields nothing and the scope chain below stays empty. Route
    // them through the document parser + Croquis so `v-scope` keys resolve as
    // template-scope completions inside `{{ }}` and directive expressions.
    if crate::utils::is_standalone_html_path(ctx.uri.path()) && ctx.dialect().is_petite_vue() {
        return petite_vue_scope_binding_completions(ctx);
    }

    let options = vize_atelier_sfc::SfcParseOptions {
        filename: ctx.uri.path().to_string().into(),
        ..Default::default()
    };

    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) else {
        return Vec::new();
    };

    // Parse the template so v-for / v-slot scopes land in the Croquis scope
    // chain. Without the AST, `analyze_sfc_descriptor` skips template-level
    // analysis and we lose nested binding visibility.
    let template_block = descriptor.template.as_ref();
    let allocator = vize_carton::Bump::new();
    let template_parse =
        template_block.map(|tb| (vize_armature::parse(&allocator, &tb.content), tb.loc.start));

    let croquis_options = SfcCroquisOptions::full();
    let croquis = if ctx.state.legacy_vue2_enabled() {
        analyze_sfc_descriptor_with_context_legacy_vue2(
            &descriptor,
            template_parse.as_ref().map(|((ast, _), _)| ast),
            croquis_options,
        )
        .croquis
    } else if ctx.state.options_api_enabled() {
        analyze_sfc_descriptor_with_context_options_api(
            &descriptor,
            template_parse.as_ref().map(|((ast, _), _)| ast),
            croquis_options,
        )
        .croquis
    } else {
        analyze_sfc_descriptor(
            &descriptor,
            template_parse.as_ref().map(|((ast, _), _)| ast),
            croquis_options,
        )
    };

    let mut items_vec = Vec::new();

    // Scope-aware completion: include bindings introduced by v-for / v-slot /
    // event-handler scopes that contain the cursor. Top-level setup bindings
    // are added by the loop below; we de-dup by name.
    if let Some((_, template_start)) = template_parse.as_ref() {
        let template_local = ctx.offset.saturating_sub(*template_start) as u32;
        for (name, _binding, scope_kind) in croquis.scopes.bindings_visible_at(template_local) {
            if !is_template_scope_kind(scope_kind) {
                continue;
            }
            if croquis.bindings.contains(name) {
                continue;
            }
            items_vec.push(template_scope_completion_item(name, scope_kind));
        }
    }
    let macro_prop_names: BTreeSet<&str> = if include_vue3_details {
        BTreeSet::new()
    } else {
        croquis
            .macros
            .props()
            .iter()
            .map(|prop| prop.name.as_str())
            .collect()
    };

    for (name, binding_type) in croquis.bindings.iter() {
        if !include_vue3_details
            && (!is_legacy_vue2_binding(binding_type)
                || binding_type == BindingType::Props && macro_prop_names.contains(name))
        {
            continue;
        }
        let (kind, type_detail, doc) = items::binding_type_to_completion_info(binding_type);
        #[allow(clippy::disallowed_macros)]
        items_vec.push(CompletionItem {
            label: name.to_string(),
            kind: Some(kind),
            label_details: Some(CompletionItemLabelDetails {
                detail: Some(type_detail.clone()),
                description: None,
            }),
            detail: Some(type_detail),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: doc,
            })),
            sort_text: Some(format!("0{}", name)),
            ..Default::default()
        });
    }

    if include_vue3_details {
        for prop in croquis.macros.props() {
            let prop_type = prop
                .prop_type
                .as_ref()
                .map(|t| t.as_str())
                .unwrap_or("unknown");
            let required = if prop.required { "" } else { "?" };

            #[allow(clippy::disallowed_macros)]
            items_vec.push(CompletionItem {
                label: prop.name.to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                label_details: Some(CompletionItemLabelDetails {
                    detail: Some(format!(": {}{}", prop_type, required)),
                    description: None,
                }),
                detail: Some(format!("prop: {}", prop_type)),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "**Prop** `{}`\n\n```typescript\n{}: {}{}\n```\n\n{}",
                        prop.name,
                        prop.name,
                        prop_type,
                        if prop.required { "" } else { " // optional" },
                        if prop.default_value.is_some() {
                            "Has default value"
                        } else {
                            ""
                        }
                    ),
                })),
                sort_text: Some(format!("0{}", prop.name)),
                ..Default::default()
            });
        }

        for source in croquis.reactivity.sources() {
            // Reactive bindings are already surfaced by the bindings loop
            // above; skip known bindings so an identifier is not offered twice.
            if croquis.bindings.contains(source.name.as_str()) {
                continue;
            }
            let kind_str = source.kind.to_display();
            #[allow(clippy::disallowed_macros)]
            items_vec.push(CompletionItem {
                label: source.name.to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                label_details: Some(CompletionItemLabelDetails {
                    detail: Some(format!(" ({})", kind_str)),
                    description: None,
                }),
                detail: Some(format!("Reactive: {}", kind_str)),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "**{}** `{}`\n\n{}\n\nAuto-unwrapped in template.",
                        kind_str,
                        source.name,
                        if source.kind.needs_value_access() {
                            "Needs `.value` in script"
                        } else {
                            "Direct access (no `.value` needed)"
                        }
                    ),
                })),
                sort_text: Some(format!("0{}", source.name)),
                ..Default::default()
            });
        }
    }

    items_vec
}

/// Template-scope completions for a petite-vue standalone HTML document.
///
/// The whole document is the template, so we parse it with the document parser
/// (`<!DOCTYPE>`-tolerant, raw-text `<script>`/`<style>`) and run Croquis over
/// the resulting AST. `v-scope` keys are modeled as `v-slot`-kind scopes, so
/// they surface through the same `bindings_visible_at` walk the SFC path uses.
/// Offsets are document-absolute here (no `<template>` block offset to
/// subtract), and there is no `<script setup>` binding set to merge.
fn petite_vue_scope_binding_completions(ctx: &IdeContext) -> Vec<CompletionItem> {
    use vize_croquis::{Drawer, DrawerOptions};

    let allocator = vize_carton::Bump::new();
    let (root, _errors) = vize_armature::parse_document(&allocator, &ctx.content);

    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_template(&root);
    let croquis = drawer.finish();

    let offset = ctx.offset.min(ctx.content.len()) as u32;
    let mut items_vec = Vec::new();
    let mut seen = BTreeSet::new();
    for (name, _binding, scope_kind) in croquis.scopes.bindings_visible_at(offset) {
        if !is_template_scope_kind(scope_kind) {
            continue;
        }
        if !seen.insert(name.to_string()) {
            continue;
        }
        items_vec.push(template_scope_completion_item(name, scope_kind));
    }
    items_vec
}

/// True for scopes introduced by template-level constructs that bring new
/// names into expression scope (v-for, v-slot, event handlers, callbacks).
fn is_template_scope_kind(kind: ScopeKind) -> bool {
    matches!(
        kind,
        ScopeKind::VFor | ScopeKind::VSlot | ScopeKind::EventHandler | ScopeKind::Callback
    )
}

#[allow(clippy::disallowed_macros)]
fn template_scope_completion_item(name: &str, scope_kind: ScopeKind) -> CompletionItem {
    let label = template_scope_label(scope_kind);
    CompletionItem {
        label: name.to_string(),
        kind: Some(CompletionItemKind::VARIABLE),
        label_details: Some(CompletionItemLabelDetails {
            detail: Some(format!(" ({label})")),
            description: Some("local".to_string()),
        }),
        detail: Some(format!("Local {label} binding")),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!(
                "**Local** binding from `{label}` scope.\n\nVisible only inside the current `{label}` subtree."
            ),
        })),
        // Inner-scope bindings sort above setup-scope candidates that use
        // a plain `0` prefix.
        sort_text: Some(format!("00{name}")),
        ..Default::default()
    }
}

fn template_scope_label(kind: ScopeKind) -> &'static str {
    match kind {
        ScopeKind::VFor => "v-for",
        ScopeKind::VSlot => "v-slot",
        ScopeKind::EventHandler => "event handler",
        ScopeKind::Callback => "callback",
        _ => "local",
    }
}

fn is_legacy_vue2_binding(binding_type: BindingType) -> bool {
    matches!(
        binding_type,
        BindingType::Data | BindingType::Options | BindingType::Props | BindingType::VueGlobal
    )
}

/// Template snippet completions.
pub(crate) fn template_snippets() -> Vec<CompletionItem> {
    vec![
        items::snippet_item(
            "vfor",
            "v-for loop",
            "<$1 v-for=\"$2 in $3\" :key=\"$4\">\n\t$0\n</$1>",
        ),
        items::snippet_item("vif", "v-if block", "<$1 v-if=\"$2\">\n\t$0\n</$1>"),
        items::snippet_item("vshow", "v-show block", "<$1 v-show=\"$2\">\n\t$0\n</$1>"),
        items::snippet_item(
            "vmodel",
            "v-model input",
            "<input v-model=\"$1\" type=\"$2\" />",
        ),
        items::snippet_item("von", "v-on handler", "<$1 @$2=\"$3\">$0</$1>"),
        items::snippet_item("vbind", "v-bind attribute", "<$1 :$2=\"$3\">$0</$1>"),
    ]
}

#[cfg(test)]
mod options_api_tests {
    use super::analyzed_template_binding_completions;
    use crate::ide::IdeContext;
    use crate::server::ServerState;
    use tower_lsp::lsp_types::Url;

    const OPTIONS_API_SFC: &str = "<script>\nexport default {\n  data() {\n    return { greeting: 'hello' }\n  },\n}\n</script>\n<template>\n  <p>{{ greeting }}</p>\n</template>\n";

    fn binding_labels(options_api: bool) -> Vec<String> {
        let state = ServerState::new();
        // Options API resolution is default-on; write an explicit config so the
        // off case exercises the opt-out path.
        let dir = tempfile::tempdir().unwrap();
        let config = if options_api {
            r#"{ "typeChecker": { "optionsApi": true } }"#
        } else {
            r#"{ "typeChecker": { "optionsApi": false } }"#
        };
        std::fs::write(dir.path().join("vize.config.json"), config).unwrap();
        state.load_workspace_config(dir.path());
        let uri = Url::parse("file:///comp.vue").unwrap();
        state.documents.open(
            uri.clone(),
            OPTIONS_API_SFC.to_string(),
            1,
            "vue".to_string(),
        );
        let offset = OPTIONS_API_SFC.find("greeting }}").unwrap();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        analyzed_template_binding_completions(&ctx, true)
            .into_iter()
            .map(|item| item.label)
            .collect()
    }

    #[test]
    fn options_api_data_binding_completed_when_enabled() {
        let labels = binding_labels(true);
        assert!(
            labels.iter().any(|label| label == "greeting"),
            "the Options API data() binding should be offered as a template completion \
             when optionsApi is enabled; got {labels:?}"
        );
    }

    #[test]
    fn options_api_data_binding_absent_when_opted_out() {
        let labels = binding_labels(false);
        assert!(
            !labels.iter().any(|label| label == "greeting"),
            "with `optionsApi: false` the Options API data() binding must not \
             resolve; got {labels:?}"
        );
    }

    // A vue-class-component SFC. Class members are auto-detected by AST shape,
    // so they populate template completion with NO optionsApi flag.
    const CLASS_COMPONENT_SFC: &str = "<script lang=\"ts\">\nimport { Vue, Component, Prop } from 'vue-property-decorator'\n@Component\nexport default class Counter extends Vue {\n  count = 0\n  @Prop() readonly title!: string\n  get doubled() { return this.count * 2 }\n  inc() { this.count++ }\n}\n</script>\n<template>\n  <p>{{ count }}</p>\n</template>\n";

    fn class_component_labels() -> Vec<String> {
        let state = ServerState::new();
        let uri = Url::parse("file:///counter.vue").unwrap();
        state.documents.open(
            uri.clone(),
            CLASS_COMPONENT_SFC.to_string(),
            1,
            "vue".to_string(),
        );
        let offset = CLASS_COMPONENT_SFC.find("count }}").unwrap();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        // Class components are auto-detected by AST shape and resolve regardless
        // of the optionsApi flag.
        analyzed_template_binding_completions(&ctx, true)
            .into_iter()
            .map(|item| item.label)
            .collect()
    }

    #[test]
    fn class_component_members_completed_without_flag() {
        let labels = class_component_labels();
        for member in ["count", "title", "doubled", "inc"] {
            assert!(
                labels.iter().any(|label| label == member),
                "class-component member `{member}` should be offered as a template \
                 completion with no optionsApi flag; got {labels:?}"
            );
        }
    }
}

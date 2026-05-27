//! Script completion provider.
//!
//! Handles completions within script blocks including Vue Composition API,
//! compiler macros, and import suggestions.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, Documentation, MarkupContent,
    MarkupKind,
};
use vize_croquis::reactivity::ReactiveKind;
use vize_croquis::{Analyzer, AnalyzerOptions};
use vize_relief::BindingType;

use super::items;
use crate::ide::IdeContext;

/// Get completions for script context.
pub(crate) fn complete_script(ctx: &IdeContext, is_setup: bool) -> Vec<CompletionItem> {
    if is_setup
        && ctx.uri.path().ends_with(".art.vue")
        && let Some(items) = crate::ide::musea::define_art_source_completions(ctx)
    {
        return items;
    }

    if let Some(items) = complete_member_access(ctx, is_setup)
        && !items.is_empty()
    {
        return items;
    }

    let mut items_vec = Vec::new();

    // Add Vue Composition API
    items_vec.extend(composition_api_completions());

    // Add Vue macros (script setup only)
    if is_setup {
        items_vec.extend(macro_completions());
    }

    // Add common imports
    items_vec.extend(import_completions());

    // Use vize_croquis for accurate bindings in script
    if let Some(script_content) = script_content_for_context(ctx, is_setup) {
        let mut analyzer = Analyzer::with_options(AnalyzerOptions {
            analyze_script: true,
            ..Default::default()
        });

        if is_setup {
            analyzer.analyze_script_setup(&script_content);
        } else {
            analyzer.analyze_script_plain(&script_content);
        }

        let croquis = analyzer.finish();

        // Add bindings with type information
        for (name, binding_type) in croquis.bindings.iter() {
            let (kind, mut type_detail, mut doc) =
                items::binding_type_to_completion_info(binding_type);
            let reactive_source = croquis.reactivity.lookup(name);
            if let Some(source) = reactive_source
                && let Some((reactive_detail, reactive_doc)) =
                    reactive_completion_info(&script_content, name, source.kind)
            {
                type_detail = reactive_detail;
                doc = reactive_doc;
            }

            // For refs in script, add .value hint
            let needs_value = reactive_source
                .map(|source| source.kind.needs_value_access())
                .unwrap_or_else(|| {
                    matches!(
                        binding_type,
                        BindingType::SetupRef | BindingType::SetupMaybeRef
                    )
                });

            #[allow(clippy::disallowed_macros)]
            items_vec.push(CompletionItem {
                label: name.to_string(),
                kind: Some(kind),
                label_details: Some(CompletionItemLabelDetails {
                    detail: Some(type_detail.clone()),
                    description: if needs_value {
                        Some(".value".to_string())
                    } else {
                        None
                    },
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

        // Add reactive sources
        for source in croquis.reactivity.sources() {
            let needs_value = source.kind.needs_value_access();
            let (type_detail, doc) =
                reactive_completion_info(&script_content, source.name.as_str(), source.kind)
                    .unwrap_or_else(|| {
                        let kind_str = source.kind.to_display().to_string();
                        let doc = if needs_value {
                            "Needs `.value` access in script.".to_string()
                        } else {
                            "Direct access (no `.value` needed).".to_string()
                        };
                        (kind_str, doc)
                    });

            #[allow(clippy::disallowed_macros)]
            items_vec.push(CompletionItem {
                label: source.name.to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                label_details: Some(CompletionItemLabelDetails {
                    detail: Some(type_detail.clone()),
                    description: if needs_value {
                        Some(".value".to_string())
                    } else {
                        None
                    },
                }),
                detail: Some(type_detail),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: doc,
                })),
                sort_text: Some(format!("0{}", source.name)),
                ..Default::default()
            });
        }
    }

    items_vec
}

fn reactive_completion_info(
    script_content: &str,
    name: &str,
    kind: ReactiveKind,
) -> Option<(String, String)> {
    let wrapper = reactive_wrapper_type(kind)?;
    let value_type = infer_reactive_value_type(script_content, name, kind)
        .unwrap_or_else(|| "unknown".to_string());
    let detail = format!("{wrapper}<{value_type}>");
    let access = if kind.needs_value_access() {
        "Access with `.value` in script."
    } else {
        "Direct access in script."
    };
    let doc = format!("```typescript\n{name}: {detail}\n```\n\n{access}");
    Some((detail, doc))
}

fn reactive_wrapper_type(kind: ReactiveKind) -> Option<&'static str> {
    match kind {
        ReactiveKind::Computed => Some("ComputedRef"),
        ReactiveKind::Ref | ReactiveKind::ShallowRef | ReactiveKind::ToRef => Some("Ref"),
        _ => None,
    }
}

fn complete_member_access(ctx: &IdeContext, is_setup: bool) -> Option<Vec<CompletionItem>> {
    let receiver = member_access_receiver(&ctx.content, ctx.offset)?;
    let script_content = script_content_for_context(ctx, is_setup)?;
    let kind = reactive_kind_for_name(&script_content, receiver)?;

    if !kind.needs_value_access() {
        return None;
    }

    let value_type = infer_reactive_value_type(&script_content, receiver, kind)
        .unwrap_or_else(|| "unknown".to_string());
    let readonly = kind == ReactiveKind::Computed;

    Some(vec![value_completion_item(&value_type, readonly)])
}

fn script_content_for_context(ctx: &IdeContext<'_>, is_setup: bool) -> Option<String> {
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: ctx.uri.path().to_string().into(),
        ..Default::default()
    };

    let descriptor = vize_atelier_sfc::parse_sfc(&ctx.content, options).ok()?;
    if is_setup {
        descriptor
            .script_setup
            .map(|script| script.content.into_owned())
    } else {
        descriptor.script.map(|script| script.content.into_owned())
    }
}

fn member_access_receiver(content: &str, offset: usize) -> Option<&str> {
    let before = &content[..offset.min(content.len())];
    let before = before.trim_end();
    let receiver_end = before.strip_suffix('.')?.len();
    let mut receiver_start = receiver_end;

    while receiver_start > 0 {
        let byte = before.as_bytes()[receiver_start - 1];
        if is_ident_byte(byte) {
            receiver_start -= 1;
        } else {
            break;
        }
    }

    if receiver_start == receiver_end {
        return None;
    }

    Some(&before[receiver_start..receiver_end])
}

#[inline]
fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn reactive_kind_for_name(script_content: &str, name: &str) -> Option<ReactiveKind> {
    let mut analyzer = Analyzer::with_options(AnalyzerOptions {
        analyze_script: true,
        ..Default::default()
    });
    analyzer.analyze_script_setup(script_content);
    let croquis = analyzer.finish();

    if let Some(source) = croquis.reactivity.lookup(name) {
        return Some(source.kind);
    }

    infer_reactive_kind_from_source(script_content, name)
}

fn infer_reactive_kind_from_source(script_content: &str, name: &str) -> Option<ReactiveKind> {
    let declaration_starts = [
        format!("const {name} = "),
        format!("let {name} = "),
        format!("var {name} = "),
    ];

    for declaration_start in declaration_starts {
        let Some(start) = script_content.find(declaration_start.as_str()) else {
            continue;
        };
        let initializer = script_content[start + declaration_start.len()..].trim_start();
        let callee = initializer
            .split_once('(')
            .map(|(callee, _)| callee.trim())
            .unwrap_or(initializer);

        if let Some(kind) = ReactiveKind::from_name(callee) {
            return Some(kind);
        }
    }

    None
}

fn infer_reactive_value_type(
    script_content: &str,
    name: &str,
    kind: ReactiveKind,
) -> Option<String> {
    let wrapper = match kind {
        ReactiveKind::Computed => "ComputedRef",
        ReactiveKind::Ref | ReactiveKind::ShallowRef | ReactiveKind::ToRef => "Ref",
        _ => return None,
    };

    let patterns = [
        format!(
            "const {name} = {callee}<",
            callee = reactive_kind_callee(kind)
        ),
        format!(
            "let {name} = {callee}<",
            callee = reactive_kind_callee(kind)
        ),
    ];
    for pattern in patterns {
        if let Some(pos) = script_content.find(pattern.as_str()) {
            let after = &script_content[pos + pattern.len()..];
            if let Some(end) = find_matching_angle(after) {
                return Some(after[..end].trim().to_string());
            }
        }
    }

    let patterns = [
        format!(
            "const {name} = {callee}(",
            callee = reactive_kind_callee(kind)
        ),
        format!(
            "let {name} = {callee}(",
            callee = reactive_kind_callee(kind)
        ),
    ];
    for pattern in patterns {
        if let Some(pos) = script_content.find(pattern.as_str()) {
            let after = &script_content[pos + pattern.len()..];
            return infer_value_type_from_initializer(after, wrapper);
        }
    }

    None
}

fn reactive_kind_callee(kind: ReactiveKind) -> &'static str {
    match kind {
        ReactiveKind::Computed => "computed",
        ReactiveKind::ShallowRef => "shallowRef",
        ReactiveKind::ToRef => "toRef",
        _ => "ref",
    }
}

fn find_matching_angle(s: &str) -> Option<usize> {
    let mut depth = 1;
    for (i, c) in s.chars().enumerate() {
        match c {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn infer_value_type_from_initializer(initializer: &str, wrapper: &str) -> Option<String> {
    let initializer = initializer.trim_start();
    if wrapper == "ComputedRef"
        && let Some(body) = extract_arrow_body(initializer)
    {
        return infer_expression_type(body);
    }

    infer_expression_type(initializer)
}

fn extract_arrow_body(initializer: &str) -> Option<&str> {
    let arrow = initializer.find("=>")?;
    let body = initializer[arrow + 2..].trim_start();

    if let Some(body) = body.strip_prefix('{')
        && let Some(return_pos) = body.find("return")
    {
        let returned = body[return_pos + "return".len()..].trim_start();
        let end = returned.find([';', '}']).unwrap_or(returned.len());
        return Some(returned[..end].trim());
    }

    let end = body.find(['\n', ';']).unwrap_or(body.len());
    Some(body[..end].trim().trim_end_matches(')').trim())
}

fn infer_expression_type(expression: &str) -> Option<String> {
    let expression = expression.trim();

    if expression.starts_with('"') || expression.starts_with('\'') || expression.starts_with('`') {
        return Some("string".to_string());
    }
    if expression.starts_with("true") || expression.starts_with("false") {
        return Some("boolean".to_string());
    }
    if expression.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
        return Some("number".to_string());
    }
    if expression.contains(".toUpperCase(")
        || expression.contains(".toLowerCase(")
        || expression.contains(".trim(")
    {
        return Some("string".to_string());
    }
    if expression.contains("===")
        || expression.contains("!==")
        || expression.contains(">=")
        || expression.contains("<=")
        || expression.contains(" > ")
        || expression.contains(" < ")
    {
        return Some("boolean".to_string());
    }
    if expression.contains('*') || expression.contains('/') || expression.contains(" - ") {
        return Some("number".to_string());
    }

    None
}

#[allow(clippy::disallowed_macros)]
fn value_completion_item(value_type: &str, readonly: bool) -> CompletionItem {
    CompletionItem {
        label: "value".to_string(),
        kind: Some(CompletionItemKind::PROPERTY),
        detail: Some(if readonly {
            format!("readonly value: {value_type}")
        } else {
            format!("value: {value_type}")
        }),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: if readonly {
                format!(
                    "Readonly computed value.\n\n```typescript\nreadonly value: {value_type}\n```"
                )
            } else {
                format!("Inner ref value.\n\n```typescript\nvalue: {value_type}\n```")
            },
        })),
        sort_text: Some("0value".to_string()),
        ..Default::default()
    }
}

/// Vue Composition API completions.
pub(crate) fn composition_api_completions() -> Vec<CompletionItem> {
    vec![
        items::api_item(
            "ref",
            "function ref<T>(value: T): Ref<T>",
            "Create a reactive reference",
        ),
        items::api_item(
            "reactive",
            "function reactive<T>(target: T): T",
            "Create a reactive object",
        ),
        items::api_item(
            "computed",
            "function computed<T>(getter: () => T): ComputedRef<T>",
            "Create a computed property",
        ),
        items::api_item(
            "watch",
            "function watch(source, callback, options?)",
            "Watch reactive sources",
        ),
        items::api_item(
            "watchEffect",
            "function watchEffect(effect: () => void)",
            "Run effect with auto-tracking",
        ),
        items::api_item(
            "onMounted",
            "function onMounted(callback: () => void)",
            "Lifecycle: after mount",
        ),
        items::api_item(
            "onUnmounted",
            "function onUnmounted(callback: () => void)",
            "Lifecycle: after unmount",
        ),
        items::api_item(
            "onBeforeMount",
            "function onBeforeMount(callback: () => void)",
            "Lifecycle: before mount",
        ),
        items::api_item(
            "onBeforeUnmount",
            "function onBeforeUnmount(callback: () => void)",
            "Lifecycle: before unmount",
        ),
        items::api_item(
            "onUpdated",
            "function onUpdated(callback: () => void)",
            "Lifecycle: after update",
        ),
        items::api_item(
            "onBeforeUpdate",
            "function onBeforeUpdate(callback: () => void)",
            "Lifecycle: before update",
        ),
        items::api_item(
            "toRef",
            "function toRef<T>(object: T, key: K): Ref<T[K]>",
            "Create ref from reactive property",
        ),
        items::api_item(
            "toRefs",
            "function toRefs<T>(object: T): ToRefs<T>",
            "Convert reactive to refs",
        ),
        items::api_item(
            "unref",
            "function unref<T>(ref: T | Ref<T>): T",
            "Unwrap a ref",
        ),
        items::api_item(
            "isRef",
            "function isRef(r): r is Ref",
            "Check if value is ref",
        ),
        items::api_item(
            "shallowRef",
            "function shallowRef<T>(value: T): ShallowRef<T>",
            "Shallow reactive reference",
        ),
        items::api_item(
            "shallowReactive",
            "function shallowReactive<T>(target: T): T",
            "Shallow reactive object",
        ),
        items::api_item(
            "readonly",
            "function readonly<T>(target: T): DeepReadonly<T>",
            "Create readonly proxy",
        ),
        items::api_item(
            "nextTick",
            "function nextTick(callback?): Promise<void>",
            "Wait for next DOM update",
        ),
        items::api_item(
            "provide",
            "function provide<T>(key, value: T)",
            "Provide value to descendants",
        ),
        items::api_item(
            "inject",
            "function inject<T>(key, defaultValue?): T",
            "Inject value from ancestor",
        ),
    ]
}

/// Vue macro completions (script setup only).
pub(crate) fn macro_completions() -> Vec<CompletionItem> {
    vec![
        items::macro_item(
            "defineArt",
            "defineArt(source, options)",
            "Declare Musea art metadata",
            "defineArt(\"$1\", {\n\ttitle: \"$2\",\n});",
        ),
        items::macro_item(
            "defineProps",
            "defineProps<T>()",
            "Declare component props",
            "defineProps<{\n\t$1\n}>()",
        ),
        items::macro_item(
            "defineEmits",
            "defineEmits<T>()",
            "Declare component emits",
            "defineEmits<{\n\t$1\n}>()",
        ),
        items::macro_item(
            "defineExpose",
            "defineExpose(exposed)",
            "Expose properties via refs",
            "defineExpose({\n\t$1\n})",
        ),
        items::macro_item(
            "defineOptions",
            "defineOptions(options)",
            "Declare component options",
            "defineOptions({\n\tname: '$1',\n})",
        ),
        items::macro_item(
            "defineSlots",
            "defineSlots<T>()",
            "Declare typed slots",
            "defineSlots<{\n\t$1\n}>()",
        ),
        items::macro_item(
            "defineModel",
            "defineModel<T>(name?, options?)",
            "Declare two-way binding prop",
            "defineModel<$1>()",
        ),
        items::macro_item(
            "withDefaults",
            "withDefaults(props, defaults)",
            "Set prop defaults",
            "withDefaults(defineProps<{\n\t$1\n}>(), {\n\t$2\n})",
        ),
    ]
}

/// Common import completions.
fn import_completions() -> Vec<CompletionItem> {
    vec![
        items::import_item("import vue", "Import from Vue", "import { $1 } from 'vue'"),
        items::import_item(
            "import ref",
            "Import ref from Vue",
            "import { ref } from 'vue'",
        ),
        items::import_item(
            "import reactive",
            "Import reactive from Vue",
            "import { reactive } from 'vue'",
        ),
        items::import_item(
            "import computed",
            "Import computed from Vue",
            "import { computed } from 'vue'",
        ),
        items::import_item(
            "import watch",
            "Import watch from Vue",
            "import { watch, watchEffect } from 'vue'",
        ),
        items::import_item(
            "import lifecycle",
            "Import lifecycle hooks",
            "import { onMounted, onUnmounted } from 'vue'",
        ),
    ]
}

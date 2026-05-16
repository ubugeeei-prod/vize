//! Script hover provider.
//!
//! Provides hover information for Vue Composition API, compiler macros,
//! and script bindings with type analysis.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind};
use vize_croquis::{Analyzer, AnalyzerOptions};
use vize_relief::BindingType;

#[cfg(feature = "native")]
use std::sync::Arc;

#[cfg(feature = "native")]
use vize_canon::CorsaBridge;

use super::HoverService;
use crate::ide::IdeContext;
impl HoverService {
    /// Get hover for script context.
    pub(super) fn hover_script(ctx: &IdeContext, is_setup: bool) -> Option<Hover> {
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset);

        if word.is_empty() {
            return None;
        }

        // Check for Vue Composition API
        if let Some(hover) = Self::hover_vue_api(&word) {
            return Some(hover);
        }

        // Check for Vue macros (script setup only)
        if is_setup {
            if let Some(hover) = Self::hover_vue_macro(&word) {
                return Some(hover);
            }
        }

        // Try to get TypeScript type information from croquis analysis
        if let Some(hover) = Self::hover_ts_binding_in_script(ctx, &word) {
            return Some(hover);
        }

        None
    }

    /// Get hover for script context with Corsa support.
    #[cfg(feature = "native")]
    pub(super) async fn hover_script_with_corsa(
        ctx: &IdeContext<'_>,
        is_setup: bool,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Option<Hover> {
        let word = Self::get_word_at_offset(&ctx.content, ctx.offset);

        if word.is_empty() {
            return None;
        }

        // Check for Vue Composition API and macros first
        if let Some(hover) = Self::hover_vue_api(&word) {
            return Some(hover);
        }

        if is_setup {
            if let Some(hover) = Self::hover_vue_macro(&word) {
                return Some(hover);
            }
        }

        // Try to get type information from Corsa via virtual TypeScript.
        if let Some(bridge) = corsa_bridge {
            if let Some(ref virtual_docs) = ctx.virtual_docs {
                let script_doc = if is_setup {
                    virtual_docs.script_setup.as_ref()
                } else {
                    virtual_docs.script.as_ref()
                };

                if let Some(script) = script_doc {
                    // Calculate position in virtual TS
                    if let Some(vts_offset) = Self::sfc_to_virtual_ts_script_offset(ctx, ctx.offset)
                    {
                        let (line, character) =
                            crate::ide::offset_to_position(&script.content, vts_offset);
                        let suffix = if is_setup { "setup.ts" } else { "script.ts" };

                        // Open/update virtual document
                        if bridge.is_initialized() {
                            #[allow(clippy::disallowed_macros)]
                            let doc_path = format!("{}.{suffix}", ctx.uri.path());
                            let Ok(uri) = bridge
                                .open_or_update_virtual_document(&doc_path, &script.content)
                                .await
                            else {
                                return Self::hover_script(ctx, is_setup);
                            };

                            // Request hover from Corsa.
                            if let Ok(Some(hover)) = bridge.hover(&uri, line, character).await {
                                return Some(Self::convert_lsp_hover(hover));
                            }
                        }
                    }
                }
            }
        }

        // Fall back to croquis analysis
        Self::hover_script(ctx, is_setup)
    }

    /// Get hover for TypeScript binding in script using croquis analysis.
    fn hover_ts_binding_in_script(ctx: &IdeContext, word: &str) -> Option<Hover> {
        // Parse SFC to get script content
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        };

        let descriptor = vize_atelier_sfc::parse_sfc(&ctx.content, options).ok()?;

        // Get the script content for type inference
        let script_content = descriptor
            .script_setup
            .as_ref()
            .map(|s| s.content.as_ref())
            .or_else(|| descriptor.script.as_ref().map(|s| s.content.as_ref()));

        // Create analyzer and analyze script
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());

        if let Some(ref script_setup) = descriptor.script_setup {
            analyzer.analyze_script_setup(&script_setup.content);
        } else if let Some(ref script) = descriptor.script {
            analyzer.analyze_script_plain(&script.content);
        }

        let summary = analyzer.finish();

        // Look up the binding in the analysis summary
        let binding_type = summary.get_binding_type(word)?;

        // Try to infer a more specific type from the script content
        let inferred_type = script_content
            .and_then(|content| Self::infer_type_from_script(content, word, binding_type))
            .unwrap_or_else(|| Self::binding_type_to_ts_display(binding_type).to_string());

        // Format the hover content with reactivity hints for script context
        let kind_desc = Self::binding_type_to_description(binding_type);

        // Add .value hint for refs in script
        #[allow(clippy::disallowed_macros)]
        let value_hint = if summary.needs_value_in_script(word) {
            format!(
                "\n\n**Tip:** Use `{}.value` to access the value in script.",
                word
            )
        } else {
            String::new()
        };

        #[allow(clippy::disallowed_macros)]
        let value = format!(
            "```typescript\n{}: {}\n```\n\n{}{}",
            word, inferred_type, kind_desc, value_hint
        );

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value,
            }),
            range: None,
        })
    }

    /// Get hover for Vue Composition API.
    pub(super) fn hover_vue_api(word: &str) -> Option<Hover> {
        let (signature, description) = match word {
            "ref" => (
                "function ref<T>(value: T): Ref<T>",
                "Takes an inner value and returns a reactive and mutable ref object, which has a single property `.value` that points to the inner value.",
            ),
            "reactive" => (
                "function reactive<T extends object>(target: T): T",
                "Returns a reactive proxy of the object. The reactive conversion is \"deep\": it affects all nested properties.",
            ),
            "computed" => (
                "function computed<T>(getter: () => T): ComputedRef<T>",
                "Takes a getter function and returns a readonly reactive ref object for the returned value from the getter.",
            ),
            "watch" => (
                "function watch<T>(source: WatchSource<T>, callback: WatchCallback<T>): WatchStopHandle",
                "Watches one or more reactive data sources and invokes a callback function when the sources change.",
            ),
            "watchEffect" => (
                "function watchEffect(effect: () => void): WatchStopHandle",
                "Runs a function immediately while reactively tracking its dependencies and re-runs it whenever the dependencies are changed.",
            ),
            "onMounted" => (
                "function onMounted(callback: () => void): void",
                "Registers a callback to be called after the component has been mounted.",
            ),
            "onUnmounted" => (
                "function onUnmounted(callback: () => void): void",
                "Registers a callback to be called after the component has been unmounted.",
            ),
            "onBeforeMount" => (
                "function onBeforeMount(callback: () => void): void",
                "Registers a hook to be called right before the component is to be mounted.",
            ),
            "onBeforeUnmount" => (
                "function onBeforeUnmount(callback: () => void): void",
                "Registers a hook to be called right before a component instance is to be unmounted.",
            ),
            "onUpdated" => (
                "function onUpdated(callback: () => void): void",
                "Registers a callback to be called after the component has updated its DOM tree due to a reactive state change.",
            ),
            "onBeforeUpdate" => (
                "function onBeforeUpdate(callback: () => void): void",
                "Registers a hook to be called right before the component is about to update its DOM tree due to a reactive state change.",
            ),
            "toRef" => (
                "function toRef<T extends object, K extends keyof T>(object: T, key: K): Ref<T[K]>",
                "Creates a ref that is synced with a property of a reactive object.",
            ),
            "toRefs" => (
                "function toRefs<T extends object>(object: T): ToRefs<T>",
                "Converts a reactive object to a plain object where each property is a ref pointing to the corresponding property of the original object.",
            ),
            "unref" => (
                "function unref<T>(ref: T | Ref<T>): T",
                "Returns the inner value if the argument is a ref, otherwise return the argument itself.",
            ),
            "isRef" => (
                "function isRef<T>(r: Ref<T> | unknown): r is Ref<T>",
                "Checks if a value is a ref object.",
            ),
            "shallowRef" => (
                "function shallowRef<T>(value: T): ShallowRef<T>",
                "Shallow version of `ref()`. The inner value is stored and exposed as-is, and will not be made deeply reactive.",
            ),
            "shallowReactive" => (
                "function shallowReactive<T extends object>(target: T): T",
                "Shallow version of `reactive()`. Only the root level is reactive, nested objects are not converted.",
            ),
            "readonly" => (
                "function readonly<T extends object>(target: T): DeepReadonly<T>",
                "Takes an object and returns a readonly proxy of the original.",
            ),
            "nextTick" => (
                "function nextTick(callback?: () => void): Promise<void>",
                "Utility for waiting for the next DOM update flush.",
            ),
            _ => return None,
        };

        #[allow(clippy::disallowed_macros)]
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!(
                    "```typescript\n{}\n```\n\n{}\n\n[Vue Documentation](https://vuejs.org/api/)",
                    signature, description
                ),
            }),
            range: None,
        })
    }

    /// Get hover for Vue macros.
    pub(super) fn hover_vue_macro(word: &str) -> Option<Hover> {
        let (signature, description) = match word {
            "defineProps" => (
                "function defineProps<T>(): T",
                "Compiler macro to declare component props. Only usable inside `<script setup>`.",
            ),
            "defineEmits" => (
                "function defineEmits<T>(): T",
                "Compiler macro to declare component emits. Only usable inside `<script setup>`.",
            ),
            "defineExpose" => (
                "function defineExpose(exposed: Record<string, any>): void",
                "Compiler macro to explicitly expose properties to the parent via template refs.",
            ),
            "defineOptions" => (
                "function defineOptions(options: ComponentOptions): void",
                "Compiler macro to declare component options. Only usable inside `<script setup>`.",
            ),
            "defineSlots" => (
                "function defineSlots<T>(): T",
                "Compiler macro for typed slots. Only usable inside `<script setup>`.",
            ),
            "defineModel" => (
                "function defineModel<T>(name?: string, options?: DefineModelOptions): ModelRef<T>",
                "Compiler macro to declare a two-way binding prop with corresponding update event.",
            ),
            "withDefaults" => (
                "function withDefaults<T>(props: T, defaults: Partial<T>): T",
                "Provides default values for props when using type-only props declaration.",
            ),
            _ => return None,
        };

        #[allow(clippy::disallowed_macros)]
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!(
                    "```typescript\n{}\n```\n\n{}\n\n*Compiler macro - only usable inside `<script setup>`*",
                    signature, description
                ),
            }),
            range: None,
        })
    }

    /// Convert BindingType to TypeScript type display string.
    pub(super) fn binding_type_to_ts_display(binding_type: BindingType) -> &'static str {
        match binding_type {
            BindingType::SetupRef => "Ref<unknown>",
            BindingType::SetupMaybeRef => "MaybeRef<unknown>",
            BindingType::SetupReactiveConst => "Reactive<unknown>",
            BindingType::SetupConst => "const",
            BindingType::SetupLet => "let",
            BindingType::Props => "Props",
            BindingType::PropsAliased => "Props (aliased)",
            BindingType::Data => "data",
            BindingType::Options => "options",
            BindingType::LiteralConst => "literal const",
            BindingType::JsGlobalUniversal => "global (universal)",
            BindingType::JsGlobalBrowser => "global (browser)",
            BindingType::JsGlobalNode => "global (node)",
            BindingType::JsGlobalDeno => "global (deno)",
            BindingType::JsGlobalBun => "global (bun)",
            BindingType::VueGlobal => "Vue global",
            BindingType::ExternalModule => "imported module",
        }
    }

    /// Convert BindingType to human-readable description.
    pub(super) fn binding_type_to_description(binding_type: BindingType) -> &'static str {
        match binding_type {
            BindingType::SetupRef => "Reactive reference created with `ref()`. Access `.value` in script, auto-unwrapped in template.",
            BindingType::SetupMaybeRef => "Value that may be a ref. Use `unref()` or `toValue()` to access in script.",
            BindingType::SetupReactiveConst => "Reactive object created with `reactive()`. Properties are reactive.",
            BindingType::SetupConst => "Constant binding from script setup. Non-reactive unless wrapped.",
            BindingType::SetupLet => "Mutable binding from script setup. Changes won't trigger reactivity.",
            BindingType::Props => "Component prop. Read-only in the component.",
            BindingType::PropsAliased => "Destructured prop with alias. Read-only.",
            BindingType::Data => "Reactive data from Options API `data()` function.",
            BindingType::Options => "Binding from Options API (methods, computed, etc.).",
            BindingType::LiteralConst => "Literal constant value, hoisted for optimization.",
            BindingType::JsGlobalUniversal => "JavaScript global available in all environments.",
            BindingType::JsGlobalBrowser => "Browser-specific global (window, document, etc.).",
            BindingType::JsGlobalNode => "Node.js-specific global (process, Buffer, etc.).",
            BindingType::JsGlobalDeno => "Deno-specific global.",
            BindingType::JsGlobalBun => "Bun-specific global.",
            BindingType::VueGlobal => "Vue template global ($slots, $emit, $attrs, etc.).",
            BindingType::ExternalModule => "Imported from external module.",
        }
    }
}

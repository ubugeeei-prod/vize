//! OXC-based script parser for high-performance AST analysis.
//!
//! Uses OXC parser to extract:
//! - Compiler macros (defineProps, defineEmits, etc.)
//! - Top-level bindings (const, let, function, class)
//! - Import statements
//! - Reactivity wrappers (ref, computed, reactive)
//! - Invalid exports in script setup
//! - Nested function scopes (arrow functions, callbacks)
//!
//! ## Module Structure
//!
//! - [`process`] - Statement and variable processing
//! - [`extract`] - Props/emits extraction and reactivity detection
//! - [`walk`] - Scope walking functions

mod extract;
mod process;
mod typeof_refs;
mod walk;

use std::sync::LazyLock;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::croquis::{BindingMetadata, ComponentRegistration, Croquis};
use crate::croquis::{ImportStatementInfo, InvalidExport, ReExportInfo, TypeExport};
use crate::macros::MacroTracker;
use crate::provide::ProvideInjectTracker;
use crate::race::RaceConditionTracker;
use crate::reactivity::ReactivityTracker;
use crate::scope::{
    JsGlobalScopeData, JsRuntime, NonScriptSetupScopeData, ParamNames, ScopeChain,
    ScriptSetupScopeData, VueGlobalScopeData,
};
use crate::setup_context::SetupContextTracker;
use vize_carton::{CompactString, FxHashMap, FxHashSet, profile};

pub use process::process_statement;

/// Origin of a local binding that already carries a plain, non-reactive value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReactiveValueOrigin {
    PropsDestructure {
        prop_name: CompactString,
    },
    ReactiveProperty {
        source_name: CompactString,
        prop_name: CompactString,
    },
    RefValue {
        source_name: CompactString,
    },
    FunctionArgument {
        source_name: CompactString,
        callee_name: CompactString,
    },
    GetterCall {
        context_name: CompactString,
        getter_name: CompactString,
        source_name: CompactString,
    },
    PlainAlias {
        source_name: CompactString,
    },
}

/// A returned context whose methods are backed by getter arguments.
#[derive(Debug, Clone, Default)]
pub(crate) struct ReactiveGetterContext {
    pub callee_name: CompactString,
    pub getters: FxHashMap<CompactString, CompactString>,
}

/// Result of parsing a script setup block
#[derive(Debug, Default)]
pub struct ScriptParseResult {
    pub bindings: BindingMetadata,
    pub macros: MacroTracker,
    pub reactivity: ReactivityTracker,
    pub race_conditions: RaceConditionTracker,
    pub type_exports: Vec<TypeExport>,
    pub invalid_exports: Vec<InvalidExport>,
    /// Scope chain for tracking nested JavaScript scopes
    pub scopes: ScopeChain,
    /// Provide/Inject tracking
    pub provide_inject: ProvideInjectTracker,
    /// Track inject variable names for indirect destructure detection
    pub(crate) inject_var_names: FxHashSet<CompactString>,
    /// Track aliases for inject function (e.g., const a = inject; a('key'))
    pub(crate) inject_aliases: FxHashSet<CompactString>,
    /// Track aliases for provide function (e.g., const p = provide; p('key', val))
    pub(crate) provide_aliases: FxHashSet<CompactString>,
    /// Track aliases for reactivity APIs (e.g., const r = ref; r(0))
    /// Maps alias name to the original function name
    pub(crate) reactivity_aliases: FxHashMap<CompactString, CompactString>,
    /// Bindings that are known plain snapshots of reactive values.
    pub(crate) reactive_value_origins: FxHashMap<CompactString, ReactiveValueOrigin>,
    /// Call results that were constructed from getter arguments.
    pub(crate) reactive_getter_contexts: FxHashMap<CompactString, ReactiveGetterContext>,
    /// Setup context violation tracking
    pub setup_context: SetupContextTracker,
    /// Flag to track if we're in a non-setup script context
    pub(crate) is_non_setup_script: bool,
    /// Import statement spans in script content
    pub import_statements: Vec<ImportStatementInfo>,
    /// Re-export statement spans (`export { ... } from "..."`)
    pub re_exports: Vec<ReExportInfo>,
    /// Components registered through Options API `components`.
    pub component_registrations: Vec<ComponentRegistration>,
    /// Definition spans for bindings (name -> (start, end) offset in script)
    pub binding_spans: FxHashMap<CompactString, (u32, u32)>,
    /// Value import source by local binding name.
    pub import_sources: FxHashMap<CompactString, CompactString>,
    /// Names referenced via `typeof X` in the body of each `type_exports`
    /// entry, indexed in parallel with `type_exports`. Used by
    /// `resolve_type_export_hoisting` to keep types adjacent to the
    /// setup-scope values they depend on. Pushed to in lockstep with
    /// `type_exports` via `record_type_export`.
    pub(crate) type_export_typeof_refs: Vec<FxHashSet<CompactString>>,
}

/// Options for plain script parsing.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScriptParserOptions {
    /// Resolve Options API template bindings (`data`/`computed`/`methods`/
    /// `inject`/`setup`/`props`). This is officially supported in Vue 3 and is
    /// an opt-in for the standard build — it does **not** require the `legacy`
    /// feature.
    pub options_api: bool,
    /// Additionally treat the component as legacy Vue 2.7 / Nuxt 2: implies
    /// `options_api` binding resolution and adds Nuxt 2 template globals.
    pub legacy_vue2: bool,
}

impl ScriptParseResult {
    /// Record a `TypeExport` together with the `typeof` value-identifier
    /// references found in its body. Must be the only call site that pushes
    /// to `type_exports` so the two vectors stay in lockstep for
    /// `resolve_type_export_hoisting`.
    pub(crate) fn record_type_export(
        &mut self,
        export: TypeExport,
        typeof_refs: FxHashSet<CompactString>,
    ) {
        self.type_exports.push(export);
        self.type_export_typeof_refs.push(typeof_refs);
    }

    /// Demote `TypeExport::hoisted` to `false` for any type whose body
    /// references a setup-scope value binding via `typeof`. The virtual TS
    /// generator only lifts hoisted types to module scope, so demoted types
    /// stay inside the synthetic `__setup` function alongside the values
    /// they depend on — which is the only place TS can resolve them.
    ///
    /// Imports add value bindings at module scope, so a `typeof importedName`
    /// reference is left as hoisted: the import is visible from the module
    /// scope where the type lands.
    pub(crate) fn resolve_type_export_hoisting(&mut self) {
        if self.type_export_typeof_refs.is_empty() {
            return;
        }

        // A `typeof name` ref keeps a type hoisted only when `name` is a
        // module-scoped import. Rather than materialize the full set of
        // imported binding names up front (O(imports × bindings)), test each
        // referenced name's declaration span against the import ranges on
        // demand — `refs` is tiny, so this is O(refs × imports).
        for idx in 0..self.type_export_typeof_refs.len() {
            let touches_setup_value = self.type_export_typeof_refs[idx].iter().any(|name| {
                let key = name.as_str();
                self.bindings.bindings.contains_key(key)
                    && !self.binding_spans.get(key).is_some_and(|(start, end)| {
                        // Bindings whose declaration site falls inside an import
                        // statement are module-scoped imports, not setup values.
                        self.import_statements
                            .iter()
                            .any(|imp| *start >= imp.start && *end <= imp.end)
                    })
            });
            if touches_setup_value && let Some(te) = self.type_exports.get_mut(idx) {
                te.hoisted = false;
            }
        }
    }

    /// Apply script analysis fields to an existing SFC analysis summary.
    ///
    /// This keeps script parsing as the single owner of script-scoped data while
    /// allowing callers to add template analysis before or after the script pass.
    pub fn apply_to_croquis(self, summary: &mut Croquis) {
        summary.bindings = self.bindings;
        summary.macros = self.macros;
        summary.reactivity = self.reactivity;
        summary.race_conditions = self.race_conditions;
        summary.type_exports = self.type_exports;
        summary.invalid_exports = self.invalid_exports;
        summary.scopes = self.scopes;
        summary.provide_inject = self.provide_inject;
        summary.setup_context = self.setup_context;
        summary.import_statements = self.import_statements;
        summary.re_exports = self.re_exports;
        summary.component_registrations = self.component_registrations;
        summary.binding_spans = self.binding_spans;
    }

    /// Convert script analysis into a `Croquis` summary.
    pub fn into_croquis(self) -> Croquis {
        let mut summary = Croquis::new();
        self.apply_to_croquis(&mut summary);
        summary
    }
}

/// Browser-only globals (WHATWG Living Standard + HTML timers).
///
/// Immutable, identical for every file. Kept as a static `&[&str]` so the
/// `CompactString` list (see `BROWSER_GLOBAL_NAMES`) is materialized once
/// instead of re-running the `smallvec!`/`const_new` construction per file.
static BROWSER_GLOBALS: &[&str] = &[
    "alert",
    "Audio",
    "cancelAnimationFrame",
    "cancelIdleCallback",
    "CanvasRenderingContext2D",
    "clearInterval",
    "clearTimeout",
    "close",
    "confirm",
    "customElements",
    "document",
    "Document",
    "DocumentFragment",
    "Element",
    "FocusEvent",
    "getComputedStyle",
    "getSelection",
    "history",
    "HTMLElement",
    "Image",
    "indexedDB",
    "InputEvent",
    "IntersectionObserver",
    "KeyboardEvent",
    "localStorage",
    "location",
    "matchMedia",
    "MediaQueryList",
    "MouseEvent",
    "MutationObserver",
    "navigator",
    "Node",
    "NodeList",
    "open",
    "PerformanceObserver",
    "PointerEvent",
    "print",
    "prompt",
    "queueMicrotask",
    "requestAnimationFrame",
    "requestIdleCallback",
    "ResizeObserver",
    "screen",
    "self",
    "sessionStorage",
    "setInterval",
    "setTimeout",
    "ShadowRoot",
    "TouchEvent",
    "WebGL2RenderingContext",
    "WebGLRenderingContext",
    "WebSocket",
    "window",
    "XMLHttpRequest",
];

/// Server-only globals (WinterCG extensions, ESM-based).
static NODE_GLOBALS: &[&str] = &["Buffer", "clearImmediate", "process", "setImmediate"];

/// Vue globals ($refs, $emit, $slots, $attrs, $el, etc.).
static VUE_GLOBALS: &[&str] = &[
    "$attrs",
    "$data",
    "$el",
    "$emit",
    "$forceUpdate",
    "$nextTick",
    "$options",
    "$parent",
    "$props",
    "$refs",
    "$root",
    "$slots",
    "$watch",
];

/// `ParamNames` (the owned form stored in each global scope) built once from the
/// static name lists. Each `enter_*_scope` consumes an owned list, so we clone
/// per file — but the `CompactString` values (all short enough to stay inline)
/// are constructed only once here rather than per file.
static BROWSER_GLOBAL_NAMES: LazyLock<ParamNames> =
    LazyLock::new(|| build_global_names(BROWSER_GLOBALS));
static NODE_GLOBAL_NAMES: LazyLock<ParamNames> = LazyLock::new(|| build_global_names(NODE_GLOBALS));
static VUE_GLOBAL_NAMES: LazyLock<ParamNames> = LazyLock::new(|| build_global_names(VUE_GLOBALS));

/// Materialize a `&'static [&str]` into the owned `ParamNames` form.
fn build_global_names(names: &'static [&'static str]) -> ParamNames {
    names
        .iter()
        .map(|&name| CompactString::const_new(name))
        .collect()
}

/// Setup global scopes hierarchy:
/// - ~universal (JS globals) - root, @0:0 (meta)
/// - ~vue (Vue globals) - parent: ~universal, @0:0 (meta)
/// - ~mod (module = SFC) - parent: ~universal, covers entire source
fn setup_global_scopes(scopes: &mut ScopeChain, source_len: u32) {
    // Root is already ~js (JsGlobalUniversal) with common globals
    // Current scope is root (~js)

    // !client - Browser-only globals (WHATWG Living Standard + HTML timers)
    // Used as parent for onMounted, onUnmounted, etc.
    scopes.enter_js_global_scope(
        JsGlobalScopeData {
            runtime: JsRuntime::Browser,
            globals: BROWSER_GLOBAL_NAMES.clone(),
        },
        0,
        0,
    );
    scopes.exit_scope(); // Back to ~univ

    // #server - Server-only globals (WinterCG extensions, ESM-based)
    // Reserved for future SSR/Server Components support
    scopes.enter_js_global_scope(
        JsGlobalScopeData {
            runtime: JsRuntime::Node,
            globals: NODE_GLOBAL_NAMES.clone(),
        },
        0,
        0,
    );
    scopes.exit_scope(); // Back to ~univ

    // ~vue - Vue globals (parent: ~univ, meta scope)
    scopes.enter_vue_global_scope(
        VueGlobalScopeData {
            globals: VUE_GLOBAL_NAMES.clone(),
        },
        0,
        0,
    );
    scopes.exit_scope(); // Back to ~univ

    // ~mod - module scope (parent: ~js, covers entire SFC)
    scopes.enter_module_scope(0, source_len);
    // Stay in module scope - setup/plain will be created as children
}

/// Parse script setup source code using OXC parser with an optional generic parameter.
///
/// `generic` is the value from `<script setup generic="T">` attribute, if present.
///
/// This is a high-performance alternative to string-based analysis,
/// providing accurate AST-based detection with proper span tracking.
pub fn parse_script_setup_with_generic(source: &str, generic: Option<&str>) -> ScriptParseResult {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();

    let ret = profile!(
        "croquis.script_setup.oxc_parse",
        Parser::new(&allocator, source, source_type).parse()
    );

    if ret.panicked {
        return ScriptParseResult::default();
    }

    let source_len = source.len() as u32;

    let mut result = ScriptParseResult {
        bindings: BindingMetadata::script_setup(),
        scopes: ScopeChain::with_capacity(16),
        ..Default::default()
    };

    // Setup global scope hierarchy (universal → mod)
    profile!(
        "croquis.script_setup.global_scopes",
        setup_global_scopes(&mut result.scopes, source_len)
    );

    // Enter script setup scope (parent: ~mod)
    result.scopes.enter_script_setup_scope(
        ScriptSetupScopeData {
            is_ts: true,
            is_async: false,
            generic: generic.map(CompactString::new),
        },
        0,
        source_len,
    );

    // Process all statements
    profile!("croquis.script_setup.walk_statements", {
        for stmt in ret.program.body.iter() {
            process::process_statement(&mut result, stmt, source);
        }
    });

    // After every binding is known, demote any `type` / `interface` that
    // references a setup-scope value via `typeof` so the virtual TS keeps
    // it inside `__setup` instead of hoisting it to module scope.
    profile!(
        "croquis.script_setup.resolve_type_hoisting",
        result.resolve_type_export_hoisting()
    );

    result
}

/// Parse script setup source code using OXC parser.
///
/// This is a high-performance alternative to string-based analysis,
/// providing accurate AST-based detection with proper span tracking.
pub fn parse_script_setup(source: &str) -> ScriptParseResult {
    parse_script_setup_with_generic(source, None)
}

/// Parse non-script-setup (Options API) source code using OXC parser.
pub fn parse_script(source: &str) -> ScriptParseResult {
    parse_script_with_options(source, ScriptParserOptions::default())
}

/// Parse non-script-setup source code using OXC parser with explicit options.
pub fn parse_script_with_options(source: &str, options: ScriptParserOptions) -> ScriptParseResult {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();

    let ret = profile!(
        "croquis.script_plain.oxc_parse",
        Parser::new(&allocator, source, source_type).parse()
    );

    if ret.panicked {
        return ScriptParseResult::default();
    }

    let source_len = source.len() as u32;

    let mut result = ScriptParseResult {
        bindings: BindingMetadata::new(), // Not script setup
        scopes: ScopeChain::with_capacity(16),
        is_non_setup_script: true, // Mark as non-setup script for violation detection
        ..Default::default()
    };

    // Setup global scope hierarchy (universal → mod)
    profile!(
        "croquis.script_plain.global_scopes",
        setup_global_scopes(&mut result.scopes, source_len)
    );

    // Enter non-script-setup scope (parent: ~mod)
    result.scopes.enter_non_script_setup_scope(
        NonScriptSetupScopeData {
            is_ts: true,
            has_define_component: false,
        },
        0,
        source_len,
    );

    process::collect_options_api_component_metadata(
        &mut result,
        &ret.program,
        options.options_api,
        options.legacy_vue2,
    );

    // Process all statements
    profile!("croquis.script_plain.walk_statements", {
        for stmt in ret.program.body.iter() {
            process::process_statement(&mut result, stmt, source);
        }
    });

    // Mirror the setup path so non-setup scripts also keep typeof-anchored
    // types adjacent to their value bindings in any downstream emitters.
    profile!(
        "croquis.script_plain.resolve_type_hoisting",
        result.resolve_type_export_hoisting()
    );

    result
}

#[cfg(test)]
mod tests {
    use super::{ScriptParserOptions, parse_script, parse_script_setup, parse_script_with_options};
    use vize_carton::{CompactString, append, cstr};

    #[test]
    fn test_parse_define_props_type() {
        let result = parse_script_setup(
            r#"
            const props = defineProps<{
                msg: string
                count?: number
            }>()
        "#,
        );

        assert_eq!(result.macros.all_calls().len(), 1);
        assert_eq!(result.macros.props().len(), 2);

        let prop_names: Vec<_> = result
            .macros
            .props()
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert!(prop_names.contains(&"msg"));
        assert!(prop_names.contains(&"count"));
    }

    #[test]
    fn test_parse_define_props_runtime() {
        let result = parse_script_setup(
            r#"
            const props = defineProps(['foo', 'bar'])
        "#,
        );

        assert_eq!(result.macros.props().len(), 2);
    }

    #[test]
    fn test_parse_define_emits() {
        let result = parse_script_setup(
            r#"
            const emit = defineEmits(['update', 'delete'])
        "#,
        );

        assert_eq!(result.macros.all_calls().len(), 1);
        assert_eq!(result.macros.emits().len(), 2);
    }

    #[test]
    fn test_parse_define_emits_runtime_args_with_spread() {
        let result = parse_script_setup(
            r#"
            const emit = defineEmits({
                ...emitObject,
            })
            defineEmits([...dialogEmits])
        "#,
        );

        let calls = result.macros.all_calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(
            calls[0].runtime_args.as_deref(),
            Some("{\n                ...emitObject,\n            }")
        );
        assert_eq!(calls[1].runtime_args.as_deref(), Some("[...dialogEmits]"));
    }

    #[test]
    fn test_parse_define_art() {
        let result = parse_script_setup(
            r#"
import Button from "./Button.vue";

defineArt(Button, {
  title: "Button",
  description: "A button component",
  category: "Components",
  tags: ["button", "ui"],
  status: "draft",
  order: 2,
});
"#,
        );

        let art = result.macros.define_art().expect("defineArt metadata");
        assert_eq!(art.component_name.as_str(), "Button");
        assert_eq!(art.component_source.as_deref(), Some("./Button.vue"));
        assert_eq!(art.title.as_deref(), Some("Button"));
        assert_eq!(art.description.as_deref(), Some("A button component"));
        assert_eq!(art.category.as_deref(), Some("Components"));
        assert_eq!(
            art.tags.iter().map(|tag| tag.as_str()).collect::<Vec<_>>(),
            ["button", "ui"]
        );
        assert_eq!(art.status.as_deref(), Some("draft"));
        assert_eq!(art.order, Some(2));
        assert!(result.macros.define_art_call().is_some());
    }

    #[test]
    fn test_parse_define_art_with_source_literal() {
        let result = parse_script_setup(
            r#"
defineArt("./forms/base-button.vue", {
  title: "Base Button",
});
"#,
        );

        let art = result.macros.define_art().expect("defineArt metadata");
        assert_eq!(art.component_name.as_str(), "BaseButton");
        assert_eq!(
            art.component_source.as_deref(),
            Some("./forms/base-button.vue")
        );
        assert!(art.component_source_span.is_some());
        assert!(art.component_source_value_span.is_some());
        assert_eq!(art.title.as_deref(), Some("Base Button"));
    }

    #[test]
    fn test_parse_define_slots() {
        let result = parse_script_setup(
            r#"
defineSlots<{
  default(props: { user: User }): any
  icon: (props: { size: number }) => any
}>()
"#,
        );

        let slots = result.macros.slots();
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0].name.as_str(), "default");
        assert_eq!(slots[0].props_type.as_deref(), Some("{ user: User }"));
        assert_eq!(slots[1].name.as_str(), "icon");
        assert_eq!(slots[1].props_type.as_deref(), Some("{ size: number }"));
    }

    #[test]
    fn test_parse_define_emits_runtime_object() {
        let result = parse_script_setup(
            r#"
            type SavePayload = { id: number }
            const emit = defineEmits({
                save: (payload: SavePayload) => payload.id > 0,
                close() { return true },
                cancel: null,
            })
        "#,
        );

        assert_eq!(result.macros.all_calls().len(), 1);
        assert_eq!(result.macros.emits().len(), 3);

        let save = result
            .macros
            .emits()
            .iter()
            .find(|emit| emit.name == "save")
            .expect("save emit should be extracted");
        assert_eq!(save.payload_type.as_deref(), Some("[payload: SavePayload]"));

        let close = result
            .macros
            .emits()
            .iter()
            .find(|emit| emit.name == "close")
            .expect("close emit should be extracted");
        assert_eq!(close.payload_type.as_deref(), Some("[]"));

        let cancel = result
            .macros
            .emits()
            .iter()
            .find(|emit| emit.name == "cancel")
            .expect("cancel emit should be extracted");
        assert_eq!(cancel.payload_type, None);
    }

    #[test]
    fn test_parse_plain_script_exported_bindings() {
        let result = parse_script(
            r#"
export const foo = 'bar'
export function hello() {}
export class MyClass {}
"#,
        );

        assert!(result.bindings.contains("foo"));
        assert!(result.bindings.contains("hello"));
        assert!(result.bindings.contains("MyClass"));
        assert!(result.invalid_exports.is_empty());
    }

    #[test]
    fn test_parse_reactivity() {
        let result = parse_script_setup(
            r#"
            const count = ref(0)
            const doubled = computed(() => count.value * 2)
            const state = reactive({ name: 'hello' })
        "#,
        );

        assert!(result.reactivity.is_reactive("count"));
        assert!(result.reactivity.is_reactive("doubled"));
        assert!(result.reactivity.is_reactive("state"));
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_parse_imports() {
        let result = parse_script_setup(
            r#"
            import { ref, computed } from 'vue'
            import MyComponent from './MyComponent.vue'
        "#,
        );

        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_parse_options_api_component_registrations() {
        let output = options_api_parse_snapshot(
            r#"
            import Style from './style.vue'
            import Basic from './basic.vue'
            import { defineComponent } from 'vue'

            export default defineComponent({
                components: {
                    FourStyle: Style,
                    Basic,
                    'string-name': Basic,
                    Ignored: defineComponent({}),
                },
            })
        "#,
        );

        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_parse_options_api_component_registrations_through_bindings() {
        let output = options_api_parse_snapshot(
            r#"
            import LocalButton from './LocalButton.vue'
            import SharedBadge from './SharedBadge.vue'
            import LateCard from './LateCard.vue'
            import { defineComponent } from 'vue'

            const component = defineComponent(options)
            const sharedComponents = {
                SharedBadge,
            }
            const components = {
                ...sharedComponents,
                PrimaryButton: LocalButton,
                LocalButton,
                'late-card': LateCard as any,
            }
            const options = {
                components,
            }

            export default component
        "#,
        );

        insta::assert_snapshot!(output);
    }

    fn options_api_parse_snapshot(source: &str) -> String {
        let result = parse_script(source);
        let mut output = String::new();

        output.push_str("=== Component Registrations ===\n");
        for registration in &result.component_registrations {
            append!(
                output,
                "{} -> {}\n",
                registration.name,
                registration.local_name
            );
        }

        output.push_str("=== Invalid Exports ===\n");
        for invalid_export in &result.invalid_exports {
            append!(
                output,
                "{}: {:?}\n",
                invalid_export.name,
                invalid_export.kind
            );
        }

        output
    }

    #[test]
    fn test_parse_invalid_exports() {
        let result = parse_script_setup(
            r#"
            export const foo = 'bar'
            export let count = 0
            export function hello() {}
            export class MyClass {}
            export default {}
        "#,
        );

        assert_eq!(result.invalid_exports.len(), 5);
    }

    #[test]
    fn test_parse_type_exports() {
        let result = parse_script_setup(
            r#"
            export type Props = { msg: string }
            export interface Emits {
                (e: 'update', value: string): void
            }
        "#,
        );

        assert_eq!(result.type_exports.len(), 2);
    }

    #[test]
    fn test_macro_span_tracking() {
        let source = "const props = defineProps<{ msg: string }>()";
        let result = parse_script_setup(source);

        let call = result.macros.all_calls().first().unwrap();
        assert!(call.start > 0);
        assert!(call.end > call.start);
        assert!(call.end as usize <= source.len());
    }

    #[test]
    fn test_nested_callback_scopes() {
        let result = parse_script_setup(
            r#"
            const items = computed(() => {
                return list.map(item => item.value)
            })
        "#,
        );

        assert!(
            result.scopes.len() >= 3,
            "Expected at least 3 scopes, got {}",
            result.scopes.len()
        );
    }

    #[test]
    fn test_parse_legacy_vue2_options_api_template_bindings() {
        let source = r#"
export default {
  props: {
    message: String,
    'user-id': Number
  },
  data() {
    return {
      count: 0
    }
  },
  asyncData() {
    return {
      pageTitle: 'Hello'
    }
  },
  computed: {
    doubled() {
      return this.count * 2
    }
  },
  methods: {
    save() {}
  },
  setup() {
    return {
      setupValue: 1
    }
  }
}
"#;
        let result = parse_script_with_options(
            source,
            ScriptParserOptions {
                options_api: false,
                legacy_vue2: true,
            },
        );

        for name in [
            "message",
            "userId",
            "count",
            "pageTitle",
            "doubled",
            "save",
            "setupValue",
            "$route",
            "$nuxt",
        ] {
            assert!(result.bindings.contains(name), "missing binding {name}");
        }
    }

    #[test]
    fn test_deeply_nested_callbacks() {
        let result = parse_script_setup(
            r#"
            onMounted(() => {
                watch(
                    () => state.value,
                    (newVal, oldVal) => {
                        console.log(newVal)
                    }
                )
            })
        "#,
        );

        assert!(
            result.scopes.len() >= 4,
            "Expected at least 4 scopes for deeply nested callbacks, got {}",
            result.scopes.len()
        );
    }

    #[test]
    fn test_closure_params_extracted() {
        use crate::scope::{ScopeData, ScopeKind};

        let result = parse_script_setup(
            r#"
            const doubled = list.map((item, index) => item * index)
        "#,
        );

        let closure_scope = result.scopes.iter().find(|s| s.kind == ScopeKind::Closure);

        assert!(closure_scope.is_some(), "Should have a closure scope");

        if let ScopeData::Closure(data) = closure_scope.unwrap().data() {
            assert!(
                data.param_names.contains(&CompactString::new("item")),
                "Closure scope should have 'item' param"
            );
            assert!(
                data.param_names.contains(&CompactString::new("index")),
                "Closure scope should have 'index' param"
            );
            assert!(data.is_arrow, "Should be an arrow function");
        } else {
            panic!("Expected closure scope data");
        }
    }

    #[test]
    fn test_binding_spans_captured() {
        let source = r#"
import { ref } from 'vue'
const count = ref(0)
function increment() {}
class MyClass {}
"#;
        let result = parse_script_setup(source);

        // ref is an import specifier
        assert!(
            result.binding_spans.contains_key("ref"),
            "Should capture import specifier span"
        );

        // count is a variable declaration
        assert!(
            result.binding_spans.contains_key("count"),
            "Should capture variable declaration span"
        );
        let (start, end) = result.binding_spans["count"];
        assert_eq!(&source[start as usize..end as usize], "count");

        // increment is a function declaration
        assert!(
            result.binding_spans.contains_key("increment"),
            "Should capture function declaration span"
        );
        let (start, end) = result.binding_spans["increment"];
        assert_eq!(&source[start as usize..end as usize], "increment");

        // MyClass is a class declaration
        assert!(
            result.binding_spans.contains_key("MyClass"),
            "Should capture class declaration span"
        );
        let (start, end) = result.binding_spans["MyClass"];
        assert_eq!(&source[start as usize..end as usize], "MyClass");
    }

    #[test]
    fn test_binding_spans_imports() {
        let source = r#"
import { ref, computed } from 'vue'
import MyComp from './MyComp.vue'
import * as utils from './utils'
"#;
        let result = parse_script_setup(source);

        for name in &["ref", "computed", "MyComp", "utils"] {
            assert!(
                result.binding_spans.contains_key(*name),
                "Should capture span for import '{}'",
                name
            );
            let (start, end) = result.binding_spans[*name];
            assert_eq!(&source[start as usize..end as usize], *name);
        }
    }

    #[test]
    fn test_binding_spans_stay_byte_aligned_with_unicode_comments() {
        let source = r#"
const before = 1
// あいうえおかきくけこさしすせそたちつてとなにぬねの
const heightLimit = "65vh"
// はひふへほまみむめもやいゆえよらりるれろわをん
"#;
        let result = parse_script_setup(source);

        let (start, end) = result.binding_spans["heightLimit"];
        assert_eq!(&source[start as usize..end as usize], "heightLimit");
    }

    // === Snapshot Tests ===

    #[test]
    fn test_parse_result_snapshot() {
        use insta::assert_snapshot;

        let result = parse_script_setup(
            r#"
import { ref, computed, watch } from 'vue'
import MyComponent from './MyComponent.vue'

const props = defineProps<{
    msg: string
    count?: number
}>()

const emit = defineEmits(['update', 'delete'])

const counter = ref(0)
const doubled = computed(() => counter.value * 2)

watch(counter, (newVal) => {
    console.log(newVal)
})

function increment() {
    counter.value++
}

const MyAlias = MyComponent
"#,
        );

        // Create a summary of the parse result for snapshot
        let bindings: Vec<_> = result.bindings.iter().collect();
        let mut bindings_sorted: Vec<_> = bindings
            .iter()
            .map(|(name, ty)| cstr!("{name}: {ty:?}"))
            .collect();
        bindings_sorted.sort();

        let mut output = String::new();
        output.push_str("=== Bindings ===\n");
        for b in &bindings_sorted {
            output.push_str(b);
            output.push('\n');
        }

        output.push_str("\n=== Macros ===\n");
        append!(output, "Props count: {}\n", result.macros.props().len());
        for p in result.macros.props() {
            append!(output, "  - {} (required: {})\n", p.name, p.required);
        }
        append!(output, "Emits count: {}\n", result.macros.emits().len());
        for e in result.macros.emits() {
            append!(output, "  - {}\n", e.name);
        }

        output.push_str("\n=== Reactivity ===\n");
        append!(
            output,
            "counter: reactive={}\n",
            result.reactivity.is_reactive("counter")
        );
        append!(
            output,
            "doubled: reactive={}\n",
            result.reactivity.is_reactive("doubled")
        );

        assert_snapshot!(output);
    }

    #[test]
    fn test_reactivity_loss_snapshot() {
        use insta::assert_snapshot;

        let result = parse_script_setup(
            r#"
const state = reactive({ count: 0, name: 'test' })
const { count, name } = state

const countRef = ref(0)
const value = countRef.value

const copy = { ...state }
"#,
        );

        let mut output = String::new();
        output.push_str("=== Reactivity Losses ===\n");
        append!(
            output,
            "Total losses: {}\n\n",
            result.reactivity.losses().len()
        );

        for (i, loss) in result.reactivity.losses().iter().enumerate() {
            append!(output, "Loss #{}: {:?}\n", i + 1, loss.kind);
            append!(output, "  span: {}..{}\n", loss.start, loss.end);
        }

        assert_snapshot!(output);
    }

    #[test]
    fn test_props_snapshot_crossing_call_and_getter_context() {
        use crate::reactivity::ReactivityLossKind;

        let result = parse_script_setup(
            r#"
const { count } = defineProps<{ count: number }>()

const ctx = useMyComposable(count)

const ctx2 = useMyComposable(() => count)
const a = ctx2.count()
"#,
        );

        assert!(result.reactivity.losses().iter().any(|loss| matches!(
            &loss.kind,
            ReactivityLossKind::FunctionArgumentExtract {
                source_name,
                argument_name,
                callee_name,
            } if source_name == "count"
                && argument_name == "count"
                && callee_name == "useMyComposable"
        )));
        assert!(result.reactivity.losses().iter().any(|loss| matches!(
            &loss.kind,
            ReactivityLossKind::GetterCallExtract {
                context_name,
                getter_name,
                target_name,
                callee_name,
                source_name,
            } if context_name == "ctx2"
                && getter_name == "count"
                && target_name == "a"
                && callee_name == "useMyComposable"
                && source_name == "count"
        )));
    }

    #[test]
    fn test_plain_reactive_values_inside_call_arguments() {
        use crate::reactivity::ReactivityLossKind;

        let result = parse_script_setup(
            r#"
const props = defineProps<{ count: number }>()
const { count: localCount } = props
const countRef = ref(0)

useMyComposable({ count: localCount })
useMyComposable(props.count)
useMyComposable(countRef.value)
watch(() => localCount, () => {})
"#,
        );

        let losses = result.reactivity.losses();
        assert!(losses.iter().any(|loss| matches!(
            &loss.kind,
            ReactivityLossKind::FunctionArgumentExtract {
                source_name,
                argument_name,
                callee_name,
            } if source_name == "props.count"
                && argument_name == "localCount"
                && callee_name == "useMyComposable"
        )));
        assert!(losses.iter().any(|loss| matches!(
            &loss.kind,
            ReactivityLossKind::FunctionArgumentExtract {
                source_name,
                argument_name,
                callee_name,
            } if source_name == "props.count"
                && argument_name == "props.count"
                && callee_name == "useMyComposable"
        )));
        assert!(losses.iter().any(|loss| matches!(
            &loss.kind,
            ReactivityLossKind::FunctionArgumentExtract {
                source_name,
                argument_name,
                callee_name,
            } if source_name == "countRef.value"
                && argument_name == "countRef.value"
                && callee_name == "useMyComposable"
        )));
        assert!(!losses.iter().any(|loss| matches!(
            &loss.kind,
            ReactivityLossKind::FunctionArgumentExtract {
                argument_name,
                callee_name,
                ..
            } if argument_name == "localCount" && callee_name == "watch"
        )));
    }

    #[test]
    fn test_plain_reactive_values_ignore_value_sink_calls() {
        use crate::reactivity::ReactivityLossKind;

        let result = parse_script_setup(
            r#"
const { count } = defineProps<{ count: number }>()
const emit = defineEmits<{ (e: 'update', value: number): void }>()

console.log(count)
console.warn({ count })
emit('update', count)
Math.max(count, 1)
Number(count)
JSON.stringify({ count })

watch(count, () => {})
useMyComposable(count)
"#,
        );

        let losses = result.reactivity.losses();
        for ignored_callee in ["log", "warn", "emit", "max", "Number", "stringify"] {
            assert!(!losses.iter().any(|loss| matches!(
                &loss.kind,
                ReactivityLossKind::FunctionArgumentExtract {
                    callee_name,
                    ..
                } if callee_name == ignored_callee
            )));
        }
        assert!(losses.iter().any(|loss| matches!(
            &loss.kind,
            ReactivityLossKind::FunctionArgumentExtract {
                argument_name,
                callee_name,
                ..
            } if argument_name == "count" && callee_name == "watch"
        )));
        assert!(losses.iter().any(|loss| matches!(
            &loss.kind,
            ReactivityLossKind::FunctionArgumentExtract {
                argument_name,
                callee_name,
                ..
            } if argument_name == "count" && callee_name == "useMyComposable"
        )));
    }

    #[test]
    fn test_plain_reactive_alias_chain_crosses_calls_and_getters() {
        use crate::reactivity::ReactivityLossKind;

        let result = parse_script_setup(
            r#"
const { count } = defineProps<{ count: number }>()

const alias = count
const second = alias
let assigned
assigned = second

useMyComposable(second)
useMyComposable(assigned)

const ctx = useMyComposable(() => second)
const a = ctx.second()
"#,
        );

        let losses = result.reactivity.losses();
        assert!(
            !losses
                .iter()
                .any(|loss| matches!(&loss.kind, ReactivityLossKind::PropsDestructure { .. }))
        );
        assert!(losses.iter().any(|loss| matches!(
            &loss.kind,
            ReactivityLossKind::PlainValueAlias {
                source_name,
                alias_name,
                target_name,
            } if source_name == "count" && alias_name == "count" && target_name == "alias"
        )));
        assert!(losses.iter().any(|loss| matches!(
            &loss.kind,
            ReactivityLossKind::PlainValueAlias {
                source_name,
                alias_name,
                target_name,
            } if source_name == "count" && alias_name == "alias" && target_name == "second"
        )));
        assert!(losses.iter().any(|loss| matches!(
            &loss.kind,
            ReactivityLossKind::PlainValueAlias {
                source_name,
                alias_name,
                target_name,
            } if source_name == "count" && alias_name == "second" && target_name == "assigned"
        )));
        assert!(losses.iter().any(|loss| matches!(
            &loss.kind,
            ReactivityLossKind::FunctionArgumentExtract {
                source_name,
                argument_name,
                callee_name,
            } if source_name == "count"
                && argument_name == "second"
                && callee_name == "useMyComposable"
        )));
        assert!(losses.iter().any(|loss| matches!(
            &loss.kind,
            ReactivityLossKind::FunctionArgumentExtract {
                source_name,
                argument_name,
                callee_name,
            } if source_name == "count"
                && argument_name == "assigned"
                && callee_name == "useMyComposable"
        )));
        assert!(losses.iter().any(|loss| matches!(
            &loss.kind,
            ReactivityLossKind::GetterCallExtract {
                context_name,
                getter_name,
                source_name,
                ..
            } if context_name == "ctx" && getter_name == "second" && source_name == "count"
        )));
    }

    #[test]
    fn test_scope_structure_snapshot() {
        use crate::scope::ScopeKind;
        use insta::assert_snapshot;

        let result = parse_script_setup(
            r#"
const items = ref([1, 2, 3])

const processed = items.value.map((item, index) => {
    return item * index
})

onMounted(() => {
    watch(() => items.value, (newVal) => {
        console.log(newVal)
    })
})

function processItem(item) {
    return item * 2
}
"#,
        );

        let mut output = String::new();
        output.push_str("=== Scope Structure ===\n");
        append!(output, "Total scopes: {}\n\n", result.scopes.len());

        // Count scopes by kind
        let mut closure_count = 0;
        let mut client_only_count = 0;
        let mut external_module_count = 0;
        let mut script_setup_count = 0;
        let mut module_count = 0;
        let mut js_global_count = 0;

        for scope in result.scopes.iter() {
            match scope.kind {
                ScopeKind::Closure => closure_count += 1,
                ScopeKind::ClientOnly => client_only_count += 1,
                ScopeKind::ExternalModule => external_module_count += 1,
                ScopeKind::ScriptSetup => script_setup_count += 1,
                ScopeKind::Module => module_count += 1,
                ScopeKind::JsGlobalUniversal
                | ScopeKind::JsGlobalBrowser
                | ScopeKind::JsGlobalNode => js_global_count += 1,
                _ => {}
            }
        }

        append!(output, "Closure scopes: {closure_count}\n");
        append!(output, "ClientOnly scopes: {client_only_count}\n");
        append!(output, "ExternalModule scopes: {external_module_count}\n");
        append!(output, "ScriptSetup scopes: {script_setup_count}\n");
        append!(output, "Module scopes: {module_count}\n");
        append!(output, "JsGlobal scopes: {js_global_count}\n");

        assert_snapshot!(output);
    }
}

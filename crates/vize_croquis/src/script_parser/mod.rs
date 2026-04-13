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
mod walk;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::analysis::BindingMetadata;
use crate::analysis::{ImportStatementInfo, InvalidExport, ReExportInfo, TypeExport};
use crate::macros::MacroTracker;
use crate::provide::ProvideInjectTracker;
use crate::reactivity::ReactivityTracker;
use crate::scope::{
    JsGlobalScopeData, JsRuntime, NonScriptSetupScopeData, ScopeChain, ScriptSetupScopeData,
    VueGlobalScopeData,
};
use crate::setup_context::SetupContextTracker;
use vize_carton::{profile, CompactString, FxHashMap, FxHashSet};

pub use process::process_statement;

/// Result of parsing a script setup block
#[derive(Debug, Default)]
pub struct ScriptParseResult {
    pub bindings: BindingMetadata,
    pub macros: MacroTracker,
    pub reactivity: ReactivityTracker,
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
    /// Setup context violation tracking
    pub setup_context: SetupContextTracker,
    /// Flag to track if we're in a non-setup script context
    pub(crate) is_non_setup_script: bool,
    /// Import statement spans in script content
    pub import_statements: Vec<ImportStatementInfo>,
    /// Re-export statement spans (`export { ... } from "..."`)
    pub re_exports: Vec<ReExportInfo>,
    /// Definition spans for bindings (name -> (start, end) offset in script)
    pub binding_spans: FxHashMap<CompactString, (u32, u32)>,
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
            globals: vize_carton::smallvec![
                CompactString::const_new("alert"),
                CompactString::const_new("Audio"),
                CompactString::const_new("cancelAnimationFrame"),
                CompactString::const_new("cancelIdleCallback"),
                CompactString::const_new("CanvasRenderingContext2D"),
                CompactString::const_new("clearInterval"),
                CompactString::const_new("clearTimeout"),
                CompactString::const_new("close"),
                CompactString::const_new("confirm"),
                CompactString::const_new("customElements"),
                CompactString::const_new("document"),
                CompactString::const_new("Document"),
                CompactString::const_new("DocumentFragment"),
                CompactString::const_new("Element"),
                CompactString::const_new("FocusEvent"),
                CompactString::const_new("getComputedStyle"),
                CompactString::const_new("getSelection"),
                CompactString::const_new("history"),
                CompactString::const_new("HTMLElement"),
                CompactString::const_new("Image"),
                CompactString::const_new("indexedDB"),
                CompactString::const_new("InputEvent"),
                CompactString::const_new("IntersectionObserver"),
                CompactString::const_new("KeyboardEvent"),
                CompactString::const_new("localStorage"),
                CompactString::const_new("location"),
                CompactString::const_new("matchMedia"),
                CompactString::const_new("MediaQueryList"),
                CompactString::const_new("MouseEvent"),
                CompactString::const_new("MutationObserver"),
                CompactString::const_new("navigator"),
                CompactString::const_new("Node"),
                CompactString::const_new("NodeList"),
                CompactString::const_new("open"),
                CompactString::const_new("PerformanceObserver"),
                CompactString::const_new("PointerEvent"),
                CompactString::const_new("print"),
                CompactString::const_new("prompt"),
                CompactString::const_new("queueMicrotask"),
                CompactString::const_new("requestAnimationFrame"),
                CompactString::const_new("requestIdleCallback"),
                CompactString::const_new("ResizeObserver"),
                CompactString::const_new("screen"),
                CompactString::const_new("self"),
                CompactString::const_new("sessionStorage"),
                CompactString::const_new("setInterval"),
                CompactString::const_new("setTimeout"),
                CompactString::const_new("ShadowRoot"),
                CompactString::const_new("TouchEvent"),
                CompactString::const_new("WebGL2RenderingContext"),
                CompactString::const_new("WebGLRenderingContext"),
                CompactString::const_new("WebSocket"),
                CompactString::const_new("window"),
                CompactString::const_new("XMLHttpRequest"),
            ],
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
            globals: vize_carton::smallvec![
                CompactString::const_new("Buffer"),
                CompactString::const_new("clearImmediate"),
                CompactString::const_new("process"),
                CompactString::const_new("setImmediate"),
            ],
        },
        0,
        0,
    );
    scopes.exit_scope(); // Back to ~univ

    // ~vue - Vue globals (parent: ~univ, meta scope)
    scopes.enter_vue_global_scope(
        VueGlobalScopeData {
            globals: vize_carton::smallvec![
                CompactString::const_new("$attrs"),
                CompactString::const_new("$data"),
                CompactString::const_new("$el"),
                CompactString::const_new("$emit"),
                CompactString::const_new("$forceUpdate"),
                CompactString::const_new("$nextTick"),
                CompactString::const_new("$options"),
                CompactString::const_new("$parent"),
                CompactString::const_new("$props"),
                CompactString::const_new("$refs"),
                CompactString::const_new("$root"),
                CompactString::const_new("$slots"),
                CompactString::const_new("$watch"),
            ],
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

    // Process all statements
    profile!("croquis.script_plain.walk_statements", {
        for stmt in ret.program.body.iter() {
            process::process_statement(&mut result, stmt, source);
        }
    });

    result
}

#[cfg(test)]
mod tests {
    use super::parse_script_setup;
    use vize_carton::{append, cstr, CompactString};

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

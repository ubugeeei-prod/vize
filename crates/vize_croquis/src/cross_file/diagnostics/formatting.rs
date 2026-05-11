//! Rich formatting for cross-file diagnostics.
//!
//! Provides the [`to_markdown`](super::CrossFileDiagnostic::to_markdown) method
//! that renders a human-readable Markdown representation of each diagnostic.

use super::{CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity};
use vize_carton::append;
use vize_carton::String;

impl CrossFileDiagnostic {
    /// Generate rich markdown diagnostic message.
    pub fn to_markdown(&self) -> String {
        let mut out = String::with_capacity(512);

        // Severity badge
        let severity_badge = match self.severity {
            DiagnosticSeverity::Error => "🔴 **ERROR**",
            DiagnosticSeverity::Warning => "🟡 **WARNING**",
            DiagnosticSeverity::Info => "🔵 **INFO**",
            DiagnosticSeverity::Hint => "💡 **HINT**",
        };

        append!(out, "{severity_badge} `{}`\n\n", self.code());
        append!(out, "### {}\n\n", self.message);

        // Detailed explanation based on kind
        self.format_kind_details(&mut out);

        // Suggestion
        if let Some(suggestion) = &self.suggestion {
            append!(out, "\n**💡 Suggestion**: {suggestion}\n");
        }

        out
    }

    /// Write kind-specific detailed explanation into the output buffer.
    fn format_kind_details(&self, out: &mut String) {
        match &self.kind {
            CrossFileDiagnosticKind::ReactivityOutsideSetup {
                api_name,
                context_description,
            } => {
                append!(
                    *out,
                    "**Problem**: `{api_name}()` is called outside the setup context ({context_description}).\n\n",
                );
                out.push_str("**Why this is dangerous**:\n\n");
                out.push_str("- 🔄 **State Pollution (CSRP)**: In SSR, module-level state is shared across requests, causing data leaks between users.\n");
                out.push_str("- 💾 **Memory Leak**: Reactive state created outside setup won't be cleaned up when the component unmounts.\n");
                out.push_str("- 🐛 **Unpredictable Behavior**: The reactivity system expects to track dependencies within component context.\n\n");
                out.push_str("**Correct usage**:\n\n");
                out.push_str("```vue\n");
                out.push_str("<script setup>\n");
                append!(
                    *out,
                    "const state = {api_name}(...) // ✅ Called in setup\n",
                );
                out.push_str("</script>\n");
                out.push_str("```\n");
            }
            CrossFileDiagnosticKind::LifecycleOutsideSetup {
                hook_name,
                context_description,
            } => {
                append!(
                    *out,
                    "**Problem**: `{hook_name}` is called outside the setup context ({context_description}).\n\n",
                );
                out.push_str("**Why this fails**:\n\n");
                out.push_str(
                    "- Lifecycle hooks must be called **synchronously** during `setup()`.\n",
                );
                out.push_str("- They rely on the current component instance being set.\n");
                out.push_str("- Calling them elsewhere will throw an error or have no effect.\n\n");
            }
            CrossFileDiagnosticKind::WatcherOutsideSetup {
                api_name,
                context_description,
            } => {
                append!(
                    *out,
                    "**Problem**: `{}()` is called outside the setup context ({}).\n\n",
                    api_name,
                    context_description
                );
                out.push_str("**Why this causes memory leaks**:\n\n");
                out.push_str("- Watchers created in setup are **automatically stopped** when the component unmounts.\n");
                out.push_str(
                    "- Watchers created outside setup **run forever** until manually stopped.\n",
                );
                out.push_str("- Each component mount creates new watchers without cleanup → memory leak.\n\n");
                out.push_str("**If you need a global watcher**, store the stop handle:\n\n");
                out.push_str("```ts\n");
                append!(*out, "const stop = {api_name}(...)\n");
                out.push_str("// Later: stop()\n");
                out.push_str("```\n");
            }
            CrossFileDiagnosticKind::SpreadBreaksReactivity {
                source_name,
                source_type,
            } => {
                append!(
                    *out,
                    "**Problem**: Spreading `{source_name}` (a `{source_type}`) creates a **non-reactive shallow copy**.\n\n",
                );
                out.push_str("**What happens**:\n\n");
                out.push_str("```ts\n");
                append!(
                    *out,
                    "const copy = {{ ...{source_name} }} // ❌ copy is NOT reactive\n",
                );
                append!(
                    *out,
                    "{source_name}.foo = 'bar' // copy.foo is still the old value\n",
                );
                out.push_str("```\n\n");
                out.push_str("**Fix**: Keep the reference, or use `toRefs()`:\n\n");
                out.push_str("```ts\n");
                append!(
                    *out,
                    "const {{ foo, bar }} = toRefs({source_name}) // ✅ foo, bar are refs\n",
                );
                out.push_str("```\n");
            }
            CrossFileDiagnosticKind::ReassignmentBreaksReactivity {
                variable_name,
                original_type,
            } => {
                append!(
                    *out,
                    "**Problem**: Reassigning `{variable_name}` loses the original `{original_type}` reference.\n\n",
                );
                out.push_str("**What happens**:\n\n");
                out.push_str("```ts\n");
                append!(*out, "let {variable_name} = ref(0)\n");
                append!(
                    *out,
                    "{variable_name} = ref(1) // ❌ Template still watches the OLD ref\n",
                );
                out.push_str("```\n\n");
                out.push_str("**Fix**: Mutate the `.value` instead:\n\n");
                out.push_str("```ts\n");
                append!(*out, "const {variable_name} = ref(0)\n");
                append!(
                    *out,
                    "{variable_name}.value = 1 // ✅ Same ref, new value\n",
                );
                out.push_str("```\n");
            }
            CrossFileDiagnosticKind::DestructuringBreaksReactivity {
                source_name,
                destructured_keys,
                suggestion,
            } => {
                append!(
                    *out,
                    "**Problem**: Destructuring `{source_name}` extracts plain values, losing reactivity.\n\n",
                );
                out.push_str("**What happens**:\n\n");
                out.push_str("```ts\n");
                let keys = destructured_keys
                    .iter()
                    .map(|k| k.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                append!(
                    *out,
                    "const {{ {keys} }} = {source_name} // ❌ {keys} are plain values\n",
                );
                out.push_str("```\n\n");
                append!(*out, "**Fix**: Use `{suggestion}()`:\n\n");
                out.push_str("```ts\n");
                append!(
                    *out,
                    "const {{ {keys} }} = {suggestion}({source_name}) // ✅ {keys} are refs\n",
                );
                out.push_str("```\n");
            }
            CrossFileDiagnosticKind::ReactiveReferenceEscapes {
                variable_name,
                escaped_via,
                target_name,
            } => {
                append!(
                    *out,
                    "**Problem**: Reactive reference `{variable_name}` escapes its scope via {escaped_via}.\n\n",
                );
                if let Some(target) = target_name {
                    append!(*out, "**Escaped to**: `{target}`\n\n");
                }
                out.push_str("**Why this is implicit** (like Rust's move semantics):\n\n");
                out.push_str("```\n");
                out.push_str("┌─ setup() ─────────────────────────────┐\n");
                append!(
                    *out,
                    "│  const {variable_name} = reactive({{...}})          │\n",
                );
                append!(
                    *out,
                    "│  someFunction({variable_name})  ←── reference escapes │\n",
                );
                out.push_str("│          │                              │\n");
                out.push_str("│          ▼                              │\n");
                out.push_str("│  ┌─ someFunction() ─────────────────┐  │\n");
                append!(
                    *out,
                    "│  │  // {variable_name} is now accessible here    │  │\n",
                );
                out.push_str("│  │  // mutations affect original     │  │\n");
                out.push_str("│  └────────────────────────────────────┘  │\n");
                out.push_str("└──────────────────────────────────────────┘\n");
                out.push_str("```\n\n");
                out.push_str("**Issues**:\n\n");
                out.push_str("- 🔍 **Hidden Data Flow**: Mutations happen \"at a distance\" - hard to trace.\n");
                out.push_str(
                    "- 🐛 **Unexpected Side Effects**: Function may modify your reactive state.\n",
                );
                out.push_str(
                    "- 📦 **Ownership Unclear**: Who \"owns\" this reactive object now?\n\n",
                );
                out.push_str("**Explicit alternatives**:\n\n");
                out.push_str("```ts\n");
                out.push_str("// Option 1: Pass a readonly version\n");
                append!(*out, "someFunction(readonly({variable_name}))\n\n");
                out.push_str("// Option 2: Pass a snapshot (non-reactive copy)\n");
                append!(*out, "someFunction({{ ...{variable_name} }})\n\n");
                out.push_str("// Option 3: Pass specific values explicitly\n");
                append!(
                    *out,
                    "someFunction({variable_name}.id, {variable_name}.name)\n",
                );
                out.push_str("```\n");
            }
            CrossFileDiagnosticKind::ReactiveObjectMutatedAfterEscape {
                variable_name,
                mutation_site,
                escape_site,
            } => {
                append!(
                    *out,
                    "**Problem**: `{variable_name}` is mutated after escaping its scope.\n\n",
                );
                append!(*out, "- Escaped at offset: {escape_site}\n");
                append!(*out, "- Mutated at offset: {mutation_site}\n\n");
                out.push_str("**Timeline**:\n\n");
                out.push_str("```\n");
                append!(*out, "1. {variable_name} created in setup()\n");
                append!(
                    *out,
                    "2. {variable_name} passed to external function (escape)\n",
                );
                append!(
                    *out,
                    "3. {variable_name} mutated ← mutations may affect escaped reference!\n",
                );
                out.push_str("```\n\n");
                out.push_str("**This is similar to Rust's borrow checker**:\n\n");
                out.push_str("- In Rust: `cannot mutate while borrowed`\n");
                out.push_str("- In Vue: mutations after escape create implicit coupling\n\n");
                out.push_str("**Consider**: Document the mutation contract or use `readonly()`.\n");
            }
            CrossFileDiagnosticKind::CircularReactiveDependency { cycle } => {
                out.push_str("**Problem**: Circular reactive dependency detected.\n\n");
                out.push_str("**Dependency Cycle**:\n\n");
                out.push_str("```\n");
                for (i, node) in cycle.iter().enumerate() {
                    if i == 0 {
                        append!(*out, "┌─→ {node}\n");
                    } else if i == cycle.len() - 1 {
                        append!(*out, "│   ↓\n└── {node} ───┘\n");
                    } else {
                        append!(*out, "│   ↓\n│   {node}\n");
                    }
                }
                out.push_str("```\n\n");
                out.push_str("**Why this is dangerous**:\n\n");
                out.push_str("- 💥 **Infinite Update Loops**: Changes propagate endlessly.\n");
                out.push_str("- 📚 **Stack Overflow Risk**: Deep recursion in reactive updates.\n");
                out.push_str("- 🐌 **Performance Degradation**: Wasted computation cycles.\n\n");
                out.push_str("**How to fix**:\n\n");
                out.push_str("```ts\n");
                out.push_str("// Option 1: Use computed() to break the cycle\n");
                out.push_str("const derived = computed(() => {\n");
                out.push_str("  // Read without triggering write\n");
                out.push_str("  return transform(source.value)\n");
                out.push_str("})\n\n");
                out.push_str("// Option 2: Use watchEffect with explicit dependencies\n");
                out.push_str("watchEffect(() => {\n");
                out.push_str("  // One-way data flow only\n");
                out.push_str("})\n\n");
                out.push_str("// Option 3: Restructure to remove bidirectional dependency\n");
                out.push_str("```\n");
            }
            CrossFileDiagnosticKind::NonReactiveProvideValue { key } => {
                append!(
                    *out,
                    "**Problem**: `provide('{key}', value)` passes a plain value to consumers.\n\n",
                );
                out.push_str("**Why this matters**:\n\n");
                out.push_str("- Consumers that call `inject()` receive a snapshot-like value.\n");
                out.push_str("- Later provider updates will not be visible unless the value is a ref, computed, reactive, readonly, or toRef/toRefs result.\n\n");
                out.push_str("**Safer patterns**:\n\n");
                out.push_str("```ts\n");
                append!(*out, "provide('{key}', ref(value))\n");
                append!(*out, "provide('{key}', reactive({{ ... }}))\n");
                append!(*out, "provide('{key}', computed(() => value))\n");
                out.push_str("```\n");
            }
            CrossFileDiagnosticKind::ProvideInjectWithoutSymbol { key, is_provide } => {
                let action = if *is_provide { "provide" } else { "inject" };
                append!(
                    *out,
                    "**Problem**: `{action}('{key}')` uses a string key instead of Symbol/InjectionKey.\n\n",
                );
                out.push_str("**Why string keys are problematic**:\n\n");
                out.push_str("```\n");
                out.push_str("┌─────────────────────────────────────────────────────────┐\n");
                out.push_str("│  String Keys          │  Symbol/InjectionKey           │\n");
                out.push_str("├───────────────────────┼────────────────────────────────┤\n");
                out.push_str("│  ❌ Name collisions    │  ✅ Guaranteed uniqueness       │\n");
                out.push_str("│  ❌ No type safety     │  ✅ Full TypeScript inference   │\n");
                out.push_str("│  ❌ Refactoring breaks │  ✅ IDE rename support          │\n");
                out.push_str("│  ❌ Hard to trace      │  ✅ Go-to-definition works      │\n");
                out.push_str("└───────────────────────┴────────────────────────────────┘\n");
                out.push_str("```\n\n");
                out.push_str("**Name collision example**:\n\n");
                out.push_str("```ts\n");
                out.push_str("// ComponentA.vue\n");
                append!(*out, "provide('{key}', myData)\n\n");
                out.push_str("// LibraryX (unknown to you)\n");
                append!(*out, "provide('{key}', otherData)  // 💥 Collision!\n");
                out.push_str("```\n\n");
                out.push_str("**Type-safe pattern with InjectionKey**:\n\n");
                out.push_str("```ts\n");
                out.push_str("// injection-keys.ts\n");
                out.push_str("import type { InjectionKey, Ref } from 'vue'\n\n");
                out.push_str("export interface UserState {\n");
                out.push_str("  name: string\n");
                out.push_str("  id: number\n");
                out.push_str("}\n\n");
                out.push_str(
                    "export const UserKey: InjectionKey<Ref<UserState>> = Symbol('user')\n\n",
                );
                out.push_str("// Provider.vue\n");
                out.push_str("import { UserKey } from './injection-keys'\n");
                out.push_str("provide(UserKey, userData)  // ✅ Type-checked\n\n");
                out.push_str("// Consumer.vue\n");
                out.push_str("import { UserKey } from './injection-keys'\n");
                out.push_str(
                    "const user = inject(UserKey)  // ✅ Type: Ref<UserState> | undefined\n",
                );
                out.push_str("```\n");
            }
            CrossFileDiagnosticKind::WatchMutationCanBeComputed {
                watch_source,
                mutated_target,
                suggested_computed,
            } => {
                out.push_str("**Problem**: This `watch` callback only mutates a reactive value based on its source.\n\n");
                out.push_str("**Current code** (imperative, harder to trace):\n\n");
                out.push_str("```ts\n");
                append!(*out, "watch({watch_source}, (newVal) => {{\n");
                append!(*out, "  {mutated_target}.value = transform(newVal)\n");
                out.push_str("})\n");
                out.push_str("```\n\n");
                out.push_str("**Why `computed` is better**:\n\n");
                out.push_str("```\n");
                out.push_str("┌─────────────────────────────────────────────────────────┐\n");
                out.push_str("│  watch + mutation       │  computed                     │\n");
                out.push_str("├─────────────────────────┼───────────────────────────────┤\n");
                out.push_str("│  ❌ Imperative flow      │  ✅ Declarative transformation │\n");
                out.push_str("│  ❌ Two variables        │  ✅ Single derived value       │\n");
                out.push_str("│  ❌ Manual sync needed   │  ✅ Auto-cached and reactive   │\n");
                out.push_str("│  ❌ Side effects possible│  ✅ Pure function guarantee    │\n");
                out.push_str("└─────────────────────────┴───────────────────────────────┘\n");
                out.push_str("```\n\n");
                out.push_str("**Refactored code** (declarative, easier to reason about):\n\n");
                out.push_str("```ts\n");
                append!(*out, "{suggested_computed}\n");
                out.push_str("```\n\n");
                out.push_str("**Note**: Use `watch` only when you need **side effects** (API calls, logging, etc.).\n");
            }
            CrossFileDiagnosticKind::DomAccessWithoutNextTick { api, context } => {
                append!(
                    *out,
                    "**Problem**: `{api}` is accessed in `{context}` without `nextTick()`.\n\n",
                );
                out.push_str("**Why this is dangerous**:\n\n");
                out.push_str("```\n");
                out.push_str("Timeline of Vue component lifecycle:\n");
                out.push_str("┌─────────────────────────────────────────────────────────┐\n");
                out.push_str("│  1. setup() runs        → DOM does NOT exist yet        │\n");
                out.push_str("│  2. Template renders    → Virtual DOM created           │\n");
                out.push_str("│  3. onMounted() fires   → DOM exists now                │\n");
                out.push_str("│  4. nextTick() resolves → DOM is fully updated          │\n");
                out.push_str("└─────────────────────────────────────────────────────────┘\n");
                out.push_str("```\n\n");
                out.push_str("**SSR considerations**:\n\n");
                out.push_str("- On the server, `document` and `window` don't exist at all.\n");
                out.push_str(
                    "- Accessing them throws `ReferenceError: document is not defined`.\n\n",
                );
                out.push_str("**Safe patterns**:\n\n");
                out.push_str("```ts\n");
                out.push_str("// Option 1: Use inside onMounted\n");
                out.push_str("onMounted(() => {\n");
                append!(*out, "  {api}  // ✅ Safe - DOM exists\n");
                out.push_str("})\n\n");
                out.push_str("// Option 2: Use nextTick after state change\n");
                out.push_str("await nextTick()\n");
                append!(*out, "{api}  // ✅ Safe - DOM updated\n");
                out.push('\n');
                out.push_str("// Option 3: Guard for SSR\n");
                out.push_str("if (typeof document !== 'undefined') {\n");
                append!(*out, "  {api}  // ✅ Safe - browser only\n");
                out.push_str("}\n");
                out.push_str("```\n");
            }
            CrossFileDiagnosticKind::AsyncBoundaryCrossing {
                variable_name,
                async_context,
            } => {
                append!(
                    *out,
                    "**Problem**: `{variable_name}` is mutated after `{async_context}` work can be invalidated.\n\n",
                );
                out.push_str("**Why this can race**:\n\n");
                out.push_str("- A slower request can finish after a newer request and overwrite fresh state.\n");
                out.push_str("- The component may unmount before the async callback resumes.\n");
                out.push_str("- Watch invalidation does not cancel promises unless cleanup is wired explicitly.\n\n");
                out.push_str("**Safer pattern**:\n\n");
                out.push_str("```ts\n");
                out.push_str("watch(source, async (_value, _oldValue, onCleanup) => {\n");
                out.push_str("  let cancelled = false\n");
                out.push_str("  onCleanup(() => { cancelled = true })\n");
                out.push_str("  const next = await load()\n");
                out.push_str("  if (!cancelled) state.value = next\n");
                out.push_str("})\n");
                out.push_str("```\n");
            }
            CrossFileDiagnosticKind::InjectedAsyncMutationRace {
                key,
                target_name,
                async_context,
                writer_count,
            } => {
                append!(
                    *out,
                    "**Problem**: Consumer code mutates injected state `{target_name}` for key `{key}` from `{async_context}`.\n\n",
                );
                append!(
                    *out,
                    "**Writer count**: {writer_count} consumer file(s) can write this provider-owned state asynchronously.\n\n",
                );
                out.push_str("**Why this can race**:\n\n");
                out.push_str("- Multiple consumers can complete async work in a different order than it was started.\n");
                out.push_str("- Provider-owned state becomes writable from distant branches of the component tree.\n");
                out.push_str(
                    "- Stale consumer work can overwrite newer provider or sibling updates.\n\n",
                );
                out.push_str("**Safer pattern**: expose a provider-owned action that performs cancellation, request-token checks, or version checks before committing state.\n");
            }
            _ => {
                // Default: just show the message
            }
        }
    }
}

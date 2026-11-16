# Integrated custom-lint API draft

> [!WARNING]
> This is a companion design draft for plugin API v2. It does not change the
> existing plugin API v1, built-in rule behavior, diagnostics, or v1 scope.

Custom lint is a first-class `lint.rule` capability in the integrated plugin
plan. A plugin registers a rule once; the same declaration drives CLI linting,
Vite and Vite+ tasks, Nuxt projects, LSP diagnostics and code actions, packaged
editors, Node, Bun, and native hosts.

The parent [integrated plugin API draft](./integrated-plugin-api-draft.md)
defines backend selection, version negotiation, security, and common caches.
The [custom-block draft](./integrated-plugin-custom-block-api-draft.md) defines
ownership and mappings for rules that inspect custom blocks.

## Compatibility baseline

`@vizejs/plugin-sdk` already ships deterministic rule batches, static visit and
fact demands, namespaced fact providers, host-owned ranges, fingerprints,
cache inputs, fixes, per-plugin costs, and one JavaScript call per plugin and
document. Those API v1 packets stay unchanged behind an adapter.

The v2 host generalizes the same strengths without making a mutable compiler
AST public. Existing v1 plugins keep their current node-span replacement fixes,
ordering, cache keys, and diagnostics. New source domains, richer edits,
type-aware facts, and native implementations require an explicit v2 manifest.

## Registration and configuration

Rules are registered by a plugin and configured through the ordinary Vize
linter section in `vite.config.*`. The config remains the single authority:

```ts
export default defineConfig().withVue({
  plugins: [designSystem()],
  linter: {
    rules: {
      "@acme/design-system/no-raw-color": "error",
      "@acme/design-system/prefer-token": ["warn", { allowFallback: true }],
    },
  },
});
```

```ts
api.lint.rule({
  id: "no-raw-color",
  meta: {
    docs: "Disallow raw colors when a design token exists.",
    defaultLevel: "off",
    fixable: "safe",
    messages: {
      raw: "Use the design token {token} instead of {value}.",
    },
    options: {
      type: api.importType("@acme/design-system", "NoRawColorOptions"),
      schema: api.importSchema("@acme/design-system/schemas", "noRawColorOptions"),
    },
  },
  documents: ["template", "script", "style", "custom-block:art"],
  visit: ["template.attribute", "css.declaration", "art.attribute"],
  demands: ["css.colors", "design.tokens"],
  run(context) {
    for (const node of context.nodes) {
      const token = context.facts("design.tokens").get(node.id);
      if (token) {
        context.report({
          node,
          messageId: "raw",
          data: { token: token.name, value: node.value },
          fix: context.replace(node.valueRange, token.reference),
        });
      }
    }
  },
});
```

Rule IDs are always plugin-namespaced. Options are validated and canonicalized
when the plan is built, before files are parsed. Presets expand to an explicit
stable rule map and never override authored rule entries silently.

Simple options may use the inline portable schema DSL. Complex authoring types
use structured `api.importType` references and may compose public unions,
intersections, generics, objects, tuples, `keyof`, and indexed access. Runtime
validation remains an independent inline or imported canonical schema because
TypeScript types are erased. Backend-specific JS validators cannot silently
stand in for a native schema.

## Public semantic projections

Rules select immutable public records, not private parser or compiler nodes.
The initial document domains are:

- SFC structure and authored attributes;
- template syntax and resolved template scopes;
- script and script-setup syntax plus declared semantic facts;
- style syntax and CSS-oriented facts;
- owner-projected custom-block facts and embedded documents; and
- project facts explicitly declared by a workspace provider.

Every record carries a stable snapshot-local ID, source identity, exact UTF-8
range, kind, and only fields admitted by its schema. Parent and relationship
tables use IDs. Plugins cannot retain host handles after a call or mutate a
record. New fields or kinds are capability-versioned.

`visit` and `demands` are static manifest data. Unknown kinds or facts refuse
the rule at plan construction instead of yielding an empty result. The host
unions enabled demands, computes each fact once, and shares the immutable table
with all admitted consumers.

## Facts and providers

A fact provider declares its input record kinds, prerequisite facts, output
schema, granularity, and cache scope. Provider names are plugin-namespaced.
Dependency cycles and ambiguous exclusive providers fail before linting.

Providers return bounded tables keyed by host record or source IDs. They may
not report diagnostics or edits, which keeps analysis reusable by lint, type
checking, and editor features. Observers can add another namespaced fact table;
they cannot replace an owner table implicitly.

Project-wide facts declare their file globs, resolver inputs, and invalidation
keys. The host supplies files from the admitted project graph; a plugin cannot
walk the workspace behind the cache contract.

## Type-aware rules

Type-aware rules declare bounded semantic queries in the manifest. They never
receive a TypeScript checker, program object, or mutable Content Mapper state.

```ts
typeAware: {
  queries: ["expression.type", "symbol.declaration", "call.signature"],
  required: true,
}
```

Canon batches the requested queries against the same semantic snapshot used by
ordinary checking and maps results to public IDs. A required query that the
host cannot supply refuses the rule; it does not degrade into a syntax-only
answer. Optional queries are explicitly nullable and visible in diagnostics.

Disabled syntax-only rules do not start the type checker. Enabled type-aware
rules share one project update and one query batch, including Content Mapper
and editor sessions. Cache keys include the semantic snapshot and dependency
graph identity.

## Diagnostics and fixes

```ts
interface PluginLintDiagnostic {
  ruleId: string;
  messageId: string;
  data?: Readonly<Record<string, string | number | boolean>>;
  source: SourceIdentity;
  range: ByteRange;
  severity?: "error" | "warning" | "info" | "hint";
  related?: readonly RelatedLocation[];
  tags?: readonly ("deprecated" | "unnecessary")[];
  fix?: PluginLintFix;
  suggestions?: readonly PluginLintFix[];
}

interface PluginLintFix {
  applicability: "safe" | "unsafe";
  edits: readonly ExactSourceEdit[];
}
```

Messages use declared IDs and bounded interpolation data so CLI and LSP output
agree. The host owns source positions, severity overrides, suppression,
ordering, rendering, and rule URLs. A plugin cannot emit an unregistered rule
or edit an undeclared source.

Fixes and suggestions require exact mappings. Multi-file edits are explicit.
The shared conflict planner orders lint fixes, code actions, and formatter edits
deterministically and refuses overlap rather than applying config or discovery
order. Safe fix mode never applies an unsafe edit. The host reparses each pass,
enforces a bounded pass count, detects cycles, and verifies unchanged regions.

## Native and JavaScript execution

Native and JavaScript rules lower to the same manifest, record schemas,
diagnostic packets, and conformance fixtures.

- Native rules receive borrowed, size-prefixed record tables through the stable
  C ABI and return host-allocated diagnostic/edit records. Thread safety is
  declared and validated before parallel execution.
- JavaScript rules use `@vizejs/plugin-sdk` under Node or Bun. The host sends one
  serialized batch per plugin and admitted document batch, never a callback per
  node or rule.
- Optional Wasm rules use the same records with explicit memory, fuel, time, and
  capability limits.

Choosing a backend changes neither rule IDs nor results. `native` fails closed
when unavailable; it does not silently run the JS implementation. An admitted
`auto` mode records its chosen backend in diagnostics, caches, and performance
receipts.

### Rust authoring

Rust lint rules are a first-class native authoring path, not built-ins that must
be upstreamed into Vize. The `vize-plugin` crate generates the stable C ABI
entrypoints and typed views over versioned host tables:

```rust
use vize_plugin::prelude::*;

#[derive(Default)]
struct DesignSystem;

#[vize_plugin(api = 2, backend = "native")]
impl Plugin for DesignSystem {
    fn manifest() -> Manifest {
        Manifest::new("com.acme.design-system", env!("CARGO_PKG_VERSION"))
            .lint_rule(
                LintRule::new("no-raw-color")
                    .visit([NodeKind::CssDeclaration])
                    .demand([FactId::new("design.tokens")]),
            )
    }

    fn lint(
        &self,
        batch: LintBatch<'_>,
        diagnostics: &mut DiagnosticSink<'_>,
    ) -> PluginResult<()> {
        for declaration in batch.css_declarations() {
            if let Some(token) = batch.fact::<DesignToken>(declaration.id())? {
                diagnostics.report(
                    declaration.value_range(),
                    "no-raw-color",
                    "raw",
                    [token.name()],
                )?;
            }
        }
        Ok(())
    }
}
```

The exact Rust surface is generated from the same schemas used by TypeScript.
The host calls a native plugin once per admitted batch and passes borrowed,
size-prefixed record and fact tables; it never crosses the ABI once per node.
Rule options arrive as validated canonical data. Diagnostics and fixes are
written through host-owned sinks so ranges, allocation, sorting, suppression,
and conflict handling remain identical to JS rules.

The metadata npm package selects signed per-platform artifacts in the same
Parcel-inspired layout as other native plugins. Every Rust rule also runs the
shared native/Node/Bun fixture corpus. A package may ship both Rust and JS
implementations, but backend choice is explicit and semantic equivalence is a
promotion gate.

## Scheduling, cache, and determinism

Disabled plugin rules add no traversal, fact, type-check, serialization, or
guest work. For enabled rules, the host builds one stage plan:

1. union record selectors and fact demands;
2. run built-in and admitted providers once in dependency order;
3. run native rules in the fused native lanes;
4. run one batch per JS/Wasm plugin outside the native walk; and
5. map, validate, sort, suppress, and conflict-check all diagnostics once.

Result keys include exact source bytes, Vize build, API and record schemas,
plugin integrity and fingerprint, canonical options, enabled severity, declared
inputs, provider outputs, semantic snapshot, target, and preceding mapped
source. A hit executes no provider or rule callback.

Identical inputs must yield byte-identical diagnostic and fix packets. The host
audits misses, bounds packet sizes and report counts, and attributes nodes,
facts, batch bytes, executions, cache status, CPU, wall time, and allocation to
each plugin and rule family.

## LSP and editor behavior

LSP diagnostics consume the same packets as the CLI. Safe fixes and suggestions
become version-checked code actions; rule documentation becomes hover and code
action metadata. Edits against stale document versions are discarded.

The editor can inspect active rules, backend choice, option provenance, fact
demands, cache state, and measured plugin cost without executing a second lint
implementation. Custom-block diagnostics remain anchored to the inline or
external source defined by the block mapping contract.

## Performance and acceptance gates

Every implementation PR records exact base/head SHAs and complete diagnostic
packet identity, then measures cold and warm wall time, CPU, peak RSS,
allocations, filesystem writes, serialized bytes, callback count, fact count,
and cache hits on Node, Bun, and native backends.

Acceptance requires:

- zero new work and files when no plugin rule is enabled;
- no-plugin p50 and p95 within the common target of at most 1% overhead;
- one crossing per admitted JS plugin batch and no per-node callback;
- identical native/Node/Bun diagnostics, ranges, fixes, and ordering;
- exact SFC/template/script/style/custom-block and Unicode/CRLF mappings;
- type-aware rules sharing one semantic update without syntax-only fallback;
- deterministic fixes, bounded multipass behavior, and cycle detection;
- cache invalidation for every declared input and no undeclared workspace read;
  and
- real-project qualification through CLI, Vite/Vite+, Nuxt, LSP, and packaged
  editors.

## Migration

1. Freeze API v1 packets and run them through an output-identical adapter.
2. Build the v2 plan from existing v1 visit/demand/fingerprint metadata with no
   new source domains or behavior.
3. Add richer diagnostics and edits behind v2 conformance fixtures.
4. Add custom-block facts using the shared block artifact, then public script
   and style projections.
5. Add bounded type queries and native rule registration only after no-plugin
   and v1 differential gates pass.
6. Publish independently maintained JS and native plugins and verify fresh Node
   and Bun consumers before promotion.

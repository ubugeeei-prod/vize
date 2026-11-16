# Integrated plugin priority experiences

> [!WARNING]
> This is a product-priority companion to the experimental plugin API draft.
> It does not add these capabilities to the v1 release scope.

The integrated plugin system should first solve extension needs that already
force Vue teams to maintain disconnected runtime, lint, type, and language-tool
implementations. Breadth is secondary to complete vertical experiences.

## Delivery order

The first wave is:

1. custom lint;
2. custom directive implementation;
3. custom block implementation; and
4. a Vue Router integration with the project-wide power associated with a
   dedicated Volar plugin, without adopting the Volar API.

The next wave is:

5. custom compiler macros;
6. custom format languages and rules; and
7. declarative `app.use` and application-install integration.

Each item must cross every relevant product from one `vite.config.*` plugin
entry. A partial runtime-only or editor-only hook does not complete a slice.

## Custom lint

Custom lint is the first proving ground because Vize already has a versioned
SDK, deterministic batches, fact demands, fixes, cache inputs, and cost
attribution. The v2 work should make that contract available through the common
plugin plan without changing API v1 packets.

The [custom-lint API draft](./integrated-plugin-lint-api-draft.md) specifies:

- rule metadata, option schemas, presets, and namespaced configuration;
- public SFC, template, script, style, and custom-block records;
- reusable native and plugin facts plus bounded type-aware queries;
- exact diagnostics, suggestions, fixes, conflict planning, and LSP actions;
- native, Node, Bun, and optional Wasm execution over one conformance corpus;
  and
- fused traversal, batched callbacks, caching, attribution, and performance
  gates.

The acceptance witness is an independently maintained rule package installed
from a public registry and run unchanged in CLI, Vite+, Nuxt, LSP, and packaged
editors under Node and Bun.

## Custom directive implementation

A directive author should declare the directive contract once instead of
shipping unrelated runtime registration, Vue type augmentation, compiler
handling, lint rules, and editor completion.

```ts
const TrackPayload = api.type.intersection(
  api.importType("@acme/tracking", "TrackPayload", [
    api.importType("@acme/domain", "EventMetadata"),
  ]),
  api.type.object({ source: api.type.literal("vue") }),
);

api.vue.directive("track", {
  registration: {
    kind: "global",
    runtime: api.import("@acme/tracking/runtime", "vTrack"),
  },
  types: {
    value: TrackPayload,
    argument: api.type.union(
      api.type.literal("click"),
      api.type.literal("view"),
      api.importType("@acme/tracking", "CustomEventName"),
    ),
    modifiers: {
      once: {
        optional: true,
        default: false,
        docs: "Report only the first event.",
      },
      passive: {
        optional: true,
        default: true,
        docs: "Do not block the event source.",
      },
    },
    element: api.domElement("HTMLElement"),
  },
  compiler: {
    vdom: "runtime",
    vapor: "runtime",
    ssr: "omit-with-diagnostic",
  },
  lint: ["require-event-name"],
  languageServer: {
    completion: true,
    hover: true,
    definition: true,
    references: true,
  },
});
```

Vue directive modifiers are boolean presence flags, but their contracts also
carry `optional`, `default`, documentation, deprecation, and availability.
`optional` defaults to `true`; the normalized default is `false`. A required
modifier uses `{ optional: false }` and emits an authored template diagnostic
when absent. Required modifiers cannot declare a default. Defaults are canonical
booleans or declared boolean config references, never arbitrary callbacks.

The compiler lowers statically known modifier defaults into VDOM, Vapor, and SSR
output. A runtime-only path uses the same precomputed normalization table once
per directive binding. Type checking, lint, completion, hover, and generated
runtime code therefore agree on absence and defaults. Unknown modifiers and
contradictory `{ optional: false, default: ... }` contracts fail closed.

Complex data belongs in the directive value or argument, where type references
may compose imported types, literals, unions, intersections, objects, tuples,
generics, `keyof`, indexed access, and conditional public aliases.

Imported types are structured references, not raw TypeScript snippets. The host
records package, export, type arguments, and resolution identity, then emits
deduplicated imports into virtual declarations. Hover and definition navigate
to the imported symbol. Unsupported or unresolved references fail before a
source file is checked.

Type projection and runtime validation are separate contracts because a
TypeScript type is erased. Portable plugin options use both when needed:

```ts
api.plugin.options({
  type: api.importType("@acme/tracking", "TrackingPluginOptions"),
  schema: api.importSchema("@acme/tracking/schemas", "trackingPluginOptions"),
});
```

An imported schema is resolved once during config planning and lowered to the
canonical portable schema. Inline schemas remain convenient for simple data.
Arbitrary JS validators or functions are backend-specific callbacks; they
cannot silently cross into a native plugin. A plugin that cannot lower its
options to canonical data, import references, and a negotiated schema refuses
that backend instead of changing validation behavior.

### Single-file directive authoring

A directive implementation does not require a separate source file. The plugin
SDK splits one authored module into the config-time manifest and a browser or
SSR runtime chunk:

```ts
import type { TrackPayload } from "@acme/tracking";
import { reportTrackingEvent } from "@acme/tracking/runtime";
import { defineVizePlugin } from "@vizejs/plugin-sdk";

export default defineVizePlugin((api) => {
  const vTrack = api.vue.runtimeDirective<TrackPayload>({
    mounted(element, binding) {
      reportTrackingEvent(element, binding.value);
    },
  });

  api.vue.directive("track", {
    registration: "global",
    runtime: vTrack,
    types: {
      value: api.importType("@acme/tracking", "TrackPayload"),
    },
  });
});
```

The same declaration may live directly in `vite.config.ts`, a project-local
plugin module, or a published package. Splitting files is an organization and
reuse choice, not an API requirement.

The SDK compiler preserves ordinary module imports and emits a stable virtual
runtime module without writing generated source into the project. Runtime code
cannot implicitly capture config-process state. Values crossing the split use
declared canonical plugin options, import references, or explicit virtual-module
exports; an untracked closure capture fails with a source diagnostic.

The authoring SDK should infer option types and generate manifest, runtime import
records, declaration fixtures, and conformance tests from this declaration. It
must not generate source files into the project or require hand-written module
augmentation.

The host validates name collisions, value/argument/modifier types, client and
SSR behavior, VDOM/Vapor support, source-map ownership, and runtime availability
before compilation. Static contracts may lower to direct imports; dynamic
contracts retain Vue resolution semantics.

One fixture corpus drives compile output, type diagnostics, template completion,
hover, definition, lint, SSR, hydration, and HMR. A package cannot claim support
for an output that lacks an implementation or an explicit safe fallback.

## Custom block implementation

Custom blocks are full embedded documents, not raw strings passed between
bundler plugins. The
[custom-block API draft](./integrated-plugin-custom-block-api-draft.md) covers
exclusive ownership, authored and external sources, parsing, facts, typecheck,
lint, format, runtime modules, mappings, HMR, LSP, and editor behavior.

The implementation experience should include:

- one typed block declaration and generated manifest schema;
- a local conformance harness for inline, self-closing, external, malformed,
  Unicode, CRLF, and multi-block fixtures;
- an inspector showing owner, backend, artifact hash, dependencies, maps, HMR
  decision, caches, and attributed cost;
- the same provider implementation under Vite, Vite+, Nuxt, CLI, and LSP; and
- exact error locations in the block or external file, never only the host SFC.

Existing `art`, `route`, and `i18n` integrations are differential oracles. They
move only after output and behavior parity is proven.

## Project-wide ecosystem integration

The fourth first-wave slice provides the capabilities needed for a Vue Router
language plugin. It is not only an LSP callback API: it owns a bounded project
analysis whose facts are reused by type checking, diagnostics, editor requests,
and optional build/runtime contributions.

```ts
api.project.integration("vue-router", {
  inputs: {
    files: ["src/pages/**/*.vue", "src/router/**/*.{ts,js}"],
    sfcBlocks: ["route"],
    macros: ["definePage"],
  },
  provides: ["router.route-graph", "router.route-names", "router.route-params"],
  virtualDocuments: ["router-types"],
  typecheck: ["route-name", "route-params"],
  languageServer: {
    diagnostics: true,
    completion: true,
    hover: true,
    definition: true,
    references: true,
    rename: true,
    symbols: true,
    codeActions: true,
  },
});
```

The host, not the plugin, expands admitted project globs and resolves modules.
The integration receives versioned snapshots and returns immutable project facts,
mapped virtual documents, and diagnostics. It declares all dependencies and
cannot scan arbitrary files behind the cache.

For the Router witness, one route graph reconciles file routes, `<route>` blocks,
`definePage`, router configuration, generated declarations, and runtime route
metadata. The same graph must power:

- route-name and param type checking;
- completion in templates, scripts, and router calls;
- hover, definition, references, rename, and workspace symbols;
- duplicate, missing, unreachable, and inconsistent-route diagnostics;
- mapped virtual declarations and Content Mapper; and
- invalidation limited to the affected graph component.

This provides Volar-plugin-like product value while keeping Vize ownership,
versioning, cancellation, source maps, cache, backend selection, and performance
contracts. Core results remain unless the integration owns the named project
fact or virtual document.

Native project analysis uses the stable C ABI. JavaScript analysis runs under
Node and Bun in bounded workspace batches, not one callback per route or editor
request. LSP reads the cached project artifact and never starts a second plugin
instance with separate state.

## Next-wave compiler macros

The [compiler-macro API draft](./integrated-plugin-compiler-macro-api-draft.md)
defines one JavaScript/Rust contract for binding-aware recognition, public IR,
structured expansion, runtime artifacts, type projections, Content Mapper,
lint, LSP, exact maps, HMR, Node, and Bun. `definePage` is the first differential
witness across VDOM, Vapor, SSR, Vite, Canon, Maestro, and packaged editors.

## Next-wave custom format

A custom formatter owns one admitted document or embedded-language projection.
It returns exact, non-overlapping authored edits. The host validates mapping,
reparses output, checks semantic preservation, enforces idempotence, and shares
the lint/format edit conflict planner.

Range and on-type formatting run only when the selected range belongs to one
exactly mapped document. Formatting an SFC never implicitly writes an external
custom-block source.

## Next-wave application install

`app.use` integration should be declarative, typed, SSR-safe, and shared by all
bundler adapters:

```ts
api.runtime.install({
  id: "router",
  plugin: api.import("vue-router", "router"),
  apply: "app.use",
  phase: "app:before-mount",
  targets: ["client", "ssr"],
  scope: "app",
  options: api.config("router"),
  hmr: "full-reload",
});
```

The contract records import identity, serialized options, installation phase,
client/SSR targets, app or request scope, idempotence, side effects, state
transfer, and HMR disposition. SSR must never leak an installed instance across
requests. Unsupported targets fail during plan construction.

The user keeps one plugin entry in `vite.config.*`; Vite, Vite+, Nuxt, standalone
builds, tests, and editor-created preview apps consume the same installation
record. Generated code uses stable imports and source maps instead of arbitrary
string concatenation.

## Cross-slice release gates

Every slice must provide:

- a typed JavaScript authoring helper and the corresponding native/Rust shape;
- manifest inspection, conflict diagnostics, and local conformance fixtures;
- exact output and diagnostic identity when disabled;
- cold/warm wall, CPU, RSS, allocations, files, bytes, callbacks, and cache
  measurements with base and head SHAs in every implementation PR;
- Node and Bun installed consumers plus native/JS semantic equivalence;
- VDOM, Vapor, SSR, hydration, Nuxt, Vite, Vite+, CLI, LSP, and packaged-editor
  qualification where the capability applies; and
- no unmanaged temporary files or second configuration authority.

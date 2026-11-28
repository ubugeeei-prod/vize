# @vizejs/plugin-sdk

Author synchronous rules, fact providers, native template transforms and compiled
output hooks. Install `@vizejs/native` **0.474.0 or newer** to execute plugins.
The optional native peer lets this package install independently for authoring;
its helpers alone do not compile or lint Vue files. Node 22 or newer is required.

Each code block below is a complete TypeScript module. CI checks the displayed
source for formatting, lint, and types against the public SDK and native
declarations, then executes it through the real host. The isolated callback example also
requires the container runtime described in its section. The final trace example
uses the workspace-only `@vizejs/devtools` package in the Vize checkout.

## Experimental plugin API v2

> [!WARNING]
> API v2 is incubating ([#13074](https://github.com/ubugeeei-prod/vize/issues/13074)).
> Its manifest, plan and capability schemas may change between releases. API v1
> below is frozen and unaffected.

`@vizejs/plugin-sdk/v2` defines one plugin for every Vize product. A factory
call runs `setup` once and records contributions; it never receives source files.

```ts
import { defineVizePlugin } from "@vizejs/plugin-sdk/v2";

export const designSystem = defineVizePlugin<{ strict: boolean }>({
  apiVersion: 2,
  stability: "experimental",
  name: "@acme/design-system",
  version: "0.1.0",
  cacheInputs: [],
  setup(api, options) {
    api.lint.rule({
      id: "no-raw-color",
      meta: { messages: { raw: "Use a design token." }, defaultLevel: "warn" },
      visit: ["ui.bind"],
      run(context) {
        for (const node of context.nodes) {
          if (options.strict && node.name === "color") context.report({ node, messageId: "raw" });
        }
      },
    });
  },
});
```

The entry carries lowered, plain-data manifest and its callbacks. The Vize host,
not this package, validates the manifest and resolves `vue.plugins` into one
canonical immutable plan: capabilities and their schema versions, the stable
`before`/`after` order, exclusive ownership claims, backend, supported outputs
and runtimes, permissions and cache inputs. Unknown required capabilities,
unsupported schemas, cycles, owner conflicts, missing callbacks, non-canonical
options (only JSON with safe integers), unavailable backends and stale plans all
fail closed with a `[plugin/<code>]` error. Only the `js` backend exists during
incubation; `native` and `wasm` refuse instead of falling back.

`api.sfc.block` and `api.vue.directive` are declared, not yet usable: every
product refuses a plan that contains one until every product consuming it
implements it. Lint and the language server already check directive modifiers,
and `vize build` compiles their defaults into VDOM, SSR and Vapor output. v1
`definePlugin` rule plugins may also appear in `vue.plugins`; the host records
them through an adapter and executes the original plugin unchanged.

## Typed visitor capability descriptors

`@vizejs/plugin-sdk/visitor` defines the stable versioned descriptor boundary for
visitor-style plugins. The v1 capability vocabulary covers SFC, template,
script, style, virtual TypeScript, diagnostics, lint, format and devtools trace
events. Each declared capability requires a matching synchronous callback, and
TypeScript narrows the callback payload to that capability.

```ts
import {
  VISITOR_API_VERSION,
  defineVisitorPlugin,
  negotiateVisitorPlugin,
} from "@vizejs/plugin-sdk/visitor";

const observer = defineVisitorPlugin({
  apiVersion: VISITOR_API_VERSION,
  name: "virtual-ts-observer",
  version: "1.0.0",
  capabilities: ["virtual-ts", "diagnostics"] as const,
  cacheInputs: [],
  visitors: {
    "virtual-ts"(event) {
      console.log(event.payload.virtualFile, event.payload.mappings.length);
    },
    diagnostics(event) {
      console.log(event.payload.diagnostics.length);
    },
  },
});

negotiateVisitorPlugin(observer, {
  apiVersion: VISITOR_API_VERSION,
  capabilities: ["virtual-ts", "diagnostics"],
});
```

Definitions fail closed on unknown or duplicate capabilities, missing or
undeclared callbacks, asynchronous functions, reserved cache-input names and API
version mismatches. `runVisitorFixture` freezes one authored event and enforces
the synchronous return contract for package tests.

This module only creates immutable descriptors and author fixtures. It does not
register callbacks in the native compiler, linter or language server, so importing
it adds no work to those paths. A production host must explicitly negotiate the
descriptor and owns cancellation, deadlines, isolation and event construction.
Virtual TypeScript and devtools trace observers use separate opt-in host
entrypoints; importing this module never registers either callback globally.

## Lint rules and autofixes

```ts
import * as native from "@vizejs/native";
import { definePlugin, applyFixes } from "@vizejs/plugin-sdk";

const source = '<template><button :disabled="true">Save</button></template>';
const rule = definePlugin({
  name: "team-rule",
  version: "1.0.0",
  visit: ["ui.bind"],
  demands: [],
  cacheInputs: [],
  rules: {
    check(ctx) {
      for (const node of ctx.nodes) {
        if (node.name === "disabled" && node.value === "true") {
          ctx.report(node, "Use a literal boolean attribute.", "disabled");
        }
      }
    },
  },
});
const output = native.lintWithPlugins(source, [rule], {
  filename: "Button.vue",
  cache: true,
  validateDeterminism: true,
});
const fixed = applyFixes(source, output.fixes);
console.log(fixed);
```

A report names a visited L2 node. Rust assigns its authored UTF-8 byte range;
a fix replaces that whole range. `applyFixes` preserves other source,
deduplicates identical suggestions and refuses conflicting edits atomically.
Apply suggestions deliberately, then rerun the compiler and linter.

Rules receive immutable batches. `ctx.facts(name)` requires the group in the
static `demands` list; no undeclared table crosses the boundary. Native groups
include `templateScopes` and the registered Croquis producers `bindings`,
`undefined-refs`, `component-usages`, `reactivity`, `provide-inject` and
`race-conditions` and `unused-bindings`. The latter computes its registered
`bindings` dependency. The `bindings` and `unused-bindings` spans are authored
SFC UTF-8 byte ranges, mapped by the shared producer even for reordered split
scripts. Other primary position fields retain their Croquis analysis frame:
undefined references/component usage are template-relative; reactivity,
provide/inject and race positions are script-relative (the merged normal/setup
script when both exist). Reports and fixes always use the host's SFC ranges. Public component interface pages are `component-signature`,
`prop-types`, `emit-types`, `slot-types`, `reactivity-classes` and
`component-references`. Each interface value preserves the producer's schema,
nullable unknowns and type dependency completeness. The signature's
`props_complete` separately records whether all property names were enumerated.
The source-only SDK route has no resolved catalog proof, so it reports false
without reading imported files from disk. Imported types which this
analysis cannot resolve remain explicit unknowns. Primary Croquis facts require
a JavaScript or TypeScript script; other script dialects are refused explicitly.

Plugin visits require inline HTML templates. Authored foreign template dialects
and external template blocks are refused before callbacks; the SDK does not
pretend their source is HTML or invent source maps for a preprocessing pass.

TypeScript infers each native group's exact map keys and deeply readonly values
from `ctx.facts("prop-types")`, including authored interface order and all emit
overloads. The package exports `FactGroups`, `FactKey`, `FactValue`, `FactEntry`
and the individual native contracts. Custom provider groups can use
`ctx.facts<MyFact>("design/labels")`; unknown groups default to `unknown`.

## Fact providers

```ts
import * as native from "@vizejs/native";
import { defineFactProvider, definePlugin } from "@vizejs/plugin-sdk";

const source =
  '<script setup>const label = "Save";</script><template><button>{{ label }}</button></template>';
const labels = defineFactProvider({
  name: "design",
  version: "1",
  provides: ["design/labels"],
  visit: ["ui.element"],
  demands: ["bindings"],
  cacheInputs: [],
  provide(batch) {
    return {
      "design/labels": batch.nodes.flatMap((node) =>
        node.name === undefined ? [] : [[node.id, node.name] as const],
      ),
    };
  },
});
const consumer = definePlugin({
  name: "labels",
  version: "1",
  demands: ["design/labels"],
  cacheInputs: [],
  rules: {
    check(ctx) {
      const labels = ctx.facts("design/labels");
      console.log(labels);
    },
  },
});
native.lintWithPlugins(source, [consumer], { cache: true, factProviders: [labels] });
```

A provider declares every output group as `provider-name/group`, its required
facts and node visits. The host computes the dependency closure and refuses
cycles, collisions, undeclared output and malformed tables before using facts.
Each table contains `[stringOrNumericKey, jsonValue]` rows with unique keys.
Provider dependency stamps enter the consumer cache key automatically.

## Native template transforms

```ts
import * as native from "@vizejs/native";
import { defineTransformPlugin } from "@vizejs/plugin-sdk";

const template = '<button class="legacy-btn">Save</button>';
const design = defineTransformPlugin({
  name: "design-system",
  version: "1",
  cacheInputs: [],
  transform(batch) {
    return batch.nodes
      .filter((node) =>
        node.attrs.some((attr) => attr.name === "class" && attr.value === "legacy-btn"),
      )
      .map((node) => ({
        kind: "replace-static-attribute",
        node: node.id,
        name: "class",
        value: "ds-button",
      }));
  },
});
const transformed = native.compileWithTransformPlugins(template, [design], {
  filename: "Button.vue",
  sourceMap: true,
  cache: true,
  cacheDir: ".vize-cache",
});
console.log(transformed.result.code);
```

The hook runs at the native pre-canonical L2 point. It can replace existing
static HTML attributes from the host's allowed set; dynamic attributes,
structural edits, template carriers and foreign namespaces refuse. Values are
semantic text, not entity spelling. Unknown targets or duplicate edits refuse
the entire response. The compiler applies typed edits to its real arena and
emits code and authored maps; no guest AST JSON is accepted. The result is a
module with prefixed identifiers. `plugins` contains per-plugin costs.

## Formatter and output hooks

```ts
import * as native from "@vizejs/native";
import { defineOutputPlugin } from "@vizejs/plugin-sdk";

const template = "<button>Save</button>";
const options = { filename: "Button.vue", sourceMap: true };
const license = defineOutputPlugin({
  name: "license",
  version: "1",
  family: "output",
  cacheInputs: [],
  output() {
    return [{ placement: "prepend" as const, comment: "Copyright Example" }];
  },
});
const compiled = native.compileWithOutputPlugins(template, [license], options);
const decorated = native.applyOutputPlugins(native.compile(template, options), [license], {
  cache: true,
});
console.log(compiled.result.code, decorated.result.code);
```

`family: "formatter"` returns `{start,end,text}` edits using UTF-8 byte offsets.
Only whitespace formatting which preserves the parsed program and comments is
accepted. `family: "output"` appends or prepends ordinary block comments;
source-map directives and executable additions refuse. The host rebases existing
map anchors and preserves source content. Each pipeline key includes preceding
output. Function-mode results carry `preamble` separately from `code`.

## Determinism, caches and costs

Declare every captured configuration or ambient input in `cacheInputs`, even
when the list is empty. Names beginning `@vize/` are host-reserved. Source,
filename, native build, SDK runtime bytes, plugin version/code, static demands,
visits and inputs participate in keys. A cached lint miss always audits two
identical batches. Providers and transforms audit misses. Cached output hooks also always audit;
uncached output hooks audit by default. Different validated outputs refuse before caching. All hook families
have bounded in-process caches and opt-in `cacheDir` persistence. Disk hits
validate their schema, keys and host-confined outputs before reuse.

Costs include elapsed host work and JavaScript call time, including audit calls.
A cache hit executes no callback; its `jsNs` and batch crossing bytes are zero.
A determinism audit observes repeated output. Trusted callbacks can conceal
ambient reads; their declarations remain the author's responsibility.

## Isolated callback sources

```ts
import * as native from "@vizejs/native";
import {
  createSandboxRunner,
  SANDBOX_IMAGE,
  SandboxRuntimeError,
} from "@vizejs/plugin-sdk/sandbox";

const source = "<template><button>Save</button></template>";
const isolated = createSandboxRunner(
  {
    name: "isolated-rule",
    version: "1",
    cacheInputs: [],
    configuration: { message: "Use our design system" },
    rules: { check: "(ctx, config) => { ctx.report(ctx.nodes[0], config.message); }" },
  },
  { timeoutMs: 5000, maxBytes: 1048576 },
);
console.log(`Required sandbox image: ${SANDBOX_IMAGE}`);
try {
  const output = native.lintWithPlugins(source, [isolated], { cache: true });
  console.log(output.diagnostics);
} catch (error) {
  if (error instanceof SandboxRuntimeError) console.error(error.code);
  throw error;
}
```

Install Docker, start its daemon and pull the exported pinned `SANDBOX_IMAGE`
before use. Callback source and explicit JSON configuration cross stdin; host
closures and imports do not. Each invocation uses a nonroot, read-only container
without host mounts, host environment, external network or capabilities, with
memory/CPU/PID/file-descriptor limits. The host bounds request/output bytes and
execution time, then confirms container removal. Runtime checking and cleanup
each add at most five seconds to `timeoutMs`.

`SandboxRuntimeError.code` distinguishes `runtime_unavailable`,
`execution_stopped`, `execution_failed` and `cleanup_failed`. Unavailable Docker
or a missing image refuses execution; it never runs the guest as trusted code.
The image's own filesystem remains readable. Docker supplies the isolation;
this API does not claim virtual-machine isolation.

`definePlugin` and the other ordinary helpers execute trusted application code
synchronously in the calling Node process and have no execution deadline.
Freezing batches and auditing output do not isolate those callbacks. The measured
proxy handle remains for spike compatibility. `@vizejs/extension-sdk` is the
separate versioned WIT guest interface.

## Virtual TypeScript visitors

`defineVisitorPlugin` descriptors with only the `virtual-ts` capability can be
negotiated for the native type-check host. The ordinary `typeCheck` entrypoint
remains plugin-free; callers opt in through the separate visitor entrypoint.

```ts
import { typeCheckWithVirtualTsVisitors } from "@vizejs/native";
import {
  VISITOR_API_VERSION,
  createVirtualTsVisitorHost,
  defineVisitorPlugin,
} from "@vizejs/plugin-sdk/visitor";

const source = "<template><button>Save</button></template>";
const observer = defineVisitorPlugin({
  apiVersion: VISITOR_API_VERSION,
  name: "virtual-ts-observer",
  version: "1",
  capabilities: ["virtual-ts"],
  cacheInputs: [],
  visitors: {
    "virtual-ts"({ payload }) {
      console.log(payload.virtualFile, payload.code, payload.mappings);
    },
  },
});

typeCheckWithVirtualTsVisitors(source, [createVirtualTsVisitorHost(observer)], {
  filename: "src/App.vue",
});
```

The callback receives immutable generated code and source/generated UTF-8 byte
ranges after the real type-check projection is built. Returning a value,
requesting another capability, or forging an incomplete host identity refuses
the invocation.

## Devtools trace visitors

`devtools-trace` descriptors attach explicitly to a request-local
`@vizejs/devtools` recorder. The recorder serializes each committed trace event
through the negotiated adapter; disabled recorders do not invoke configured
hosts, and ordinary recorders without `visitorHosts` do not inspect plugin
descriptors.

```ts
import { createDevtoolsTraceRecorder } from "@vizejs/devtools";
import {
  VISITOR_API_VERSION,
  createDevtoolsTraceVisitorHost,
  defineVisitorPlugin,
} from "@vizejs/plugin-sdk/visitor";

const observer = defineVisitorPlugin({
  apiVersion: VISITOR_API_VERSION,
  name: "trace-observer",
  version: "1",
  capabilities: ["devtools-trace"],
  cacheInputs: [],
  visitors: {
    "devtools-trace"({ file, payload }) {
      console.log(file, payload.events);
    },
  },
});

const trace = createDevtoolsTraceRecorder({
  sessionId: "request-1",
  visitorHosts: [createDevtoolsTraceVisitorHost(observer)],
});
trace.record({
  kind: "render:end",
  componentId: "SaveButton",
  componentName: "SaveButton",
  source: "src/SaveButton.vue",
  durationMs: 1,
});
```

Every delivered event has a deterministic session/sequence ID, a stage derived
from its trace kind, millisecond timestamps normalized to nanoseconds, and the
remaining recorder fields in immutable `data`. A visitor exception or non-void
result refuses the record before it enters the snapshot.

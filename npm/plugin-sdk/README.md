# @vizejs/plugin-sdk

Author synchronous rules, fact providers, native template transforms and compiled
output hooks. Install `@vizejs/native` **0.429.0 or newer** to execute plugins.
The optional native peer lets this package install independently for authoring;
its helpers alone do not compile or lint Vue files. Node 22 or newer is required.

## Lint rules and autofixes

```js
import * as native from "@vizejs/native";
import { definePlugin, applyFixes } from "@vizejs/plugin-sdk";

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

```js
import { defineFactProvider, definePlugin } from "@vizejs/plugin-sdk";
const labels = defineFactProvider({
  name: "design",
  version: "1",
  provides: ["design/labels"],
  visit: ["ui.element"],
  demands: ["bindings"],
  cacheInputs: [],
  provide(batch) {
    return { "design/labels": batch.nodes.map((node) => [node.id, node.name]) };
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

```js
import { defineTransformPlugin } from "@vizejs/plugin-sdk";
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
```

The hook runs at the native pre-canonical L2 point. It can replace existing
static HTML attributes from the host's allowed set; dynamic attributes,
structural edits, template carriers and foreign namespaces refuse. Values are
semantic text, not entity spelling. Unknown targets or duplicate edits refuse
the entire response. The compiler applies typed edits to its real arena and
emits code and authored maps; no guest AST JSON is accepted. The result is a
module with prefixed identifiers. `plugins` contains per-plugin costs.

## Formatter and output hooks

```js
import { defineOutputPlugin } from "@vizejs/plugin-sdk";
const license = defineOutputPlugin({
  name: "license",
  version: "1",
  family: "output",
  cacheInputs: [],
  output() {
    return [{ placement: "prepend", comment: "Copyright Example" }];
  },
});
const compiled = native.compileWithOutputPlugins(template, [license], options);
const decorated = native.applyOutputPlugins(transformed.result, [license], { cache: true });
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

```js
import {
  createSandboxRunner,
  SANDBOX_IMAGE,
  SandboxRuntimeError,
} from "@vizejs/plugin-sdk/sandbox";
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
native.lintWithPlugins(source, [isolated], { cache: true });
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

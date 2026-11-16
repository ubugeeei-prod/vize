// Plugin API v1 is frozen (#13074): every packet a v1 plugin exchanges with the
// native host must stay byte-identical while API v2 incubates beside it.
// Regenerate only for an intentional, versioned v1 break:
//   VIZE_PLUGIN_API_V1_FREEZE=write node --test tests/tooling/davinci-plugin-api-v1-freeze.test.ts
import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";

import {
  defineFactProvider,
  defineOutputPlugin,
  definePlugin,
  defineTransformPlugin,
} from "../../npm/plugin-sdk/index.js";
import booleanAttributes from "./fixtures/davinci-plugin-sdk/boolean-attributes.mjs";
import { TODO_LIST } from "./fixtures/davinci-plugin-sdk/sources.mjs";
import team from "./fixtures/davinci-plugin-sdk/team-conventions.mjs";
import design from "./fixtures/davinci-plugin-sdk/transforms/design-system.mjs";

const native = createRequire(import.meta.url)("../../npm/native/index.js");
const golden = new URL("./fixtures/davinci-plugin-sdk/api-v1-packets.json", import.meta.url);

const BOOLEAN_FORM = `<template>\r\n  <form>\r\n    <input :disabled="true" :required="true" />\r\n    <!-- 日本語🎨 -->\r\n    <select :multiple="true"><option :selected="true">é</option></select>\r\n  </form>\r\n</template>\r\n`;
const BINDINGS = `<script setup lang="ts">\nimport { ref } from "vue";\nconst count = ref(0);\nconst unused = 1;\n</script>\n<template>\n  <button @click="count++">{{ count }} 🦀</button>\n</template>\n`;

/** Record every crossing without changing the plugin the host receives. */
function recorded<T extends { run(batch: string): string }>(plugin: T, packets: string[][]) {
  return {
    ...plugin,
    run(batch: string) {
      const response = plugin.run(batch);
      packets.push([batch, response]);
      return response;
    },
  };
}

const bindingsRule = definePlugin({
  name: "binding-audit",
  version: "1.0.0",
  visit: ["ui.interpolation", "ui.on"],
  demands: ["bindings", "unused-bindings"],
  cacheInputs: [],
  rules: {
    "report-bindings"(ctx) {
      const bindings = [...ctx.facts("bindings").keys()].map(String).sort();
      const unused = [...ctx.facts("unused-bindings").keys()].map(String).sort();
      for (const node of ctx.nodes) {
        ctx.report(node, `bindings=${bindings.join(",")} unused=${unused.join(",")}`);
      }
    },
  },
});

const labels = defineFactProvider({
  name: "labels",
  version: "1.0.0",
  visit: ["ui.element"],
  demands: ["templateScopes"],
  provides: ["labels/tags"],
  cacheInputs: [],
  provide(batch) {
    return { "labels/tags": batch.nodes.map((node) => [node.id, node.name]) };
  },
});

const labelRule = definePlugin({
  name: "label-consumer",
  version: "1.0.0",
  visit: ["ui.element"],
  demands: ["labels/tags"],
  cacheInputs: [],
  rules: {
    tags(ctx) {
      const tags = ctx.facts("labels/tags");
      for (const node of ctx.nodes) {
        const tag = tags.get(node.id);
        if (tag === "td") ctx.report(node, `tag ${tag}`);
      }
    },
  },
});

const stable = (cost: Record<string, unknown>, keys: string[]) =>
  Object.fromEntries(keys.map((key) => [key, cost[key]]));
const RULE_COST = ["name", "version", "nodes", "batchBytes", "reports", "cached", "executions"];

function lintCase(
  name: string,
  source: string,
  filename: string,
  plugins: unknown[],
  options = {},
) {
  const packets: string[][] = [];
  const providerPackets: string[][] = [];
  const factProviders = (options as { factProviders?: { run(batch: string): string }[] })
    .factProviders;
  const output = native.lintWithPlugins(
    source,
    plugins.map((plugin) => recorded(plugin as { run(batch: string): string }, packets)),
    {
      filename,
      ...options,
      ...(factProviders
        ? { factProviders: factProviders.map((provider) => recorded(provider, providerPackets)) }
        : {}),
    },
  );
  return {
    name,
    family: "lint",
    packets,
    providerPackets,
    output: {
      diagnostics: output.diagnostics,
      fixes: output.fixes,
      filename: output.filename,
      plugins: output.plugins.map((cost: Record<string, unknown>) => stable(cost, RULE_COST)),
      factProviders: output.factProviders.map((cost: Record<string, unknown>) =>
        stable(cost, ["name", "version", "providedGroups", "audited", "cached"]),
      ),
    },
  };
}

function transformCase() {
  const packets: string[][] = [];
  const source = '<main><button class="legacy-btn">é {{ label }}</button><i class="x"></i></main>';
  const output = native.compileWithTransformPlugins(source, [recorded(design, packets)], {
    filename: "src/Design.vue",
  });
  return {
    name: "transform/static-class",
    family: "transform",
    packets,
    output: {
      plugins: output.plugins.map((cost: Record<string, unknown>) =>
        stable(cost, ["name", "version", "edits", "cached"]),
      ),
    },
  };
}

// Output batches embed compiled code, which compiler changes legitimately
// move; the frozen v1 surface is the envelope, the response and the costs.
function outputCase(family: "formatter" | "output") {
  const packets: string[][] = [];
  const hook =
    family === "output"
      ? defineOutputPlugin({
          name: "license",
          version: "1.0.0",
          family,
          cacheInputs: [],
          output: () => [{ placement: "prepend", comment: "Copyright 日本語🎨" }],
        })
      : defineOutputPlugin({
          name: "spacing",
          version: "1.0.0",
          family,
          cacheInputs: [],
          output: () => [],
        });
  const output = native.compileWithOutputPlugins(
    '<button title="🎨" @click="save">{{ label }}</button>',
    [recorded(hook, packets)],
    { mode: "function", prefixIdentifiers: true, filename: "Output.vue" },
  );
  return {
    name: `output/${family}`,
    family,
    packets: packets.map(([batch, response]) => {
      const parsed = JSON.parse(batch);
      return [
        JSON.stringify({
          keys: Object.keys(parsed),
          compiled: Object.keys(parsed.compiled ?? {}),
          schema: parsed.schema,
          family: parsed.family,
          offsetEncoding: parsed.offsetEncoding,
        }),
        response,
      ];
    }),
    output: {
      plugins: output.plugins.map((cost: Record<string, unknown>) =>
        stable(cost, ["name", "version", "operations", "cached"]),
      ),
    },
  };
}

function capture() {
  return [
    lintCase("lint/team-conventions", TODO_LIST, "TodoList.vue", [team]),
    lintCase("lint/boolean-fixes-crlf-unicode", BOOLEAN_FORM, "Form.vue", [booleanAttributes]),
    lintCase("lint/two-plugins-audited", TODO_LIST, "TodoList.vue", [team, booleanAttributes], {
      validateDeterminism: true,
    }),
    lintCase("lint/croquis-facts", BINDINGS, "Counter.vue", [bindingsRule]),
    lintCase("lint/fact-provider", TODO_LIST, "TodoList.vue", [labelRule], {
      factProviders: [labels],
    }),
    transformCase(),
    outputCase("output"),
    outputCase("formatter"),
  ];
}

test("plugin API v1 packets stay byte-identical", () => {
  const actual = `${JSON.stringify(capture(), null, 2)}\n`;
  if (process.env.VIZE_PLUGIN_API_V1_FREEZE === "write") writeFileSync(golden, actual);
  const expected = readFileSync(golden, "utf8");
  const cases = JSON.parse(expected) as { name: string }[];
  const observed = JSON.parse(actual) as { name: string }[];
  assert.deepEqual(
    observed.map(({ name }) => name),
    cases.map(({ name }) => name),
  );
  for (const [index, entry] of cases.entries()) {
    assert.equal(
      JSON.stringify(observed[index]),
      JSON.stringify(entry),
      `${entry.name}: a plugin API v1 packet changed`,
    );
  }
});

test("v1 hook descriptors keep their authored shape", () => {
  const transform = defineTransformPlugin({
    name: "shape",
    version: "1",
    cacheInputs: [],
    transform: () => [],
  });
  assert.deepEqual(Object.keys(team).sort(), [
    "cacheInputs",
    "demands",
    "fingerprint",
    "name",
    "rules",
    "run",
    "version",
    "visit",
  ]);
  assert.deepEqual(Object.keys(transform).sort(), [
    "cacheInputs",
    "fingerprint",
    "name",
    "run",
    "version",
  ]);
  assert.deepEqual(Object.keys(labels).sort(), [
    "cacheInputs",
    "demands",
    "fingerprint",
    "name",
    "provides",
    "run",
    "version",
    "visit",
  ]);
});

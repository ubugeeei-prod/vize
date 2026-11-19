import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "node:test";
import { applyFixes, defineOutputPlugin, definePlugin } from "../../npm/plugin-sdk/index.js";
import design from "./fixtures/davinci-plugin-sdk/transforms/design-system.mjs";
import booleans from "./fixtures/davinci-plugin-sdk/boolean-attributes.mjs";
import team from "./fixtures/davinci-plugin-sdk/team-conventions.mjs";

const native = createRequire(import.meta.url)("../../npm/native/index.js");
const source = `<script setup>const items = [{ id: "a" }]</script>
<template>日本語😀<button v-for="(item, i) in items" :key="i" :disabled="true">Go</button><Child :disabled="true" /></template>`;
const options = { filename: "Buttons.vue", cache: true, validateDeterminism: true };

test("two team rules run cached, deterministic, attributed and host-confined autofixes", () => {
  const first = native.lintWithPlugins(source, [team, booleans], options);
  const hit = native.lintWithPlugins(source, [team, booleans], options);
  assert.deepEqual(
    first.diagnostics.map((row) => row.ruleId),
    ["team-conventions/no-index-key", "boolean-attributes/prefer-boolean-shorthand"],
  );
  assert.deepEqual(hit.diagnostics, first.diagnostics);
  assert.deepEqual(hit.fixes, first.fixes);
  assert.deepEqual(
    first.plugins.map((row) => row.cached),
    [false, false],
  );
  assert.deepEqual(
    hit.plugins.map((row) => [row.cached, row.nodes, row.batchBytes, row.jsNs]),
    [
      [true, 0, 0, 0],
      [true, 0, 0, 0],
    ],
  );
  for (const cost of first.plugins) {
    assert.ok(cost.elapsedNs >= cost.jsNs && cost.jsNs > 0);
    assert.equal(cost.reports, 1);
    assert.ok(cost.contentKey && cost.batchBytes > 0);
  }
  const [fix] = first.fixes;
  assert.deepEqual(fix, {
    ruleId: "boolean-attributes/prefer-boolean-shorthand",
    plugin: "boolean-attributes",
    start: Buffer.byteLength(source.slice(0, source.indexOf(":disabled"))),
    end: Buffer.byteLength(
      source.slice(0, source.indexOf(":disabled") + ':disabled="true"'.length),
    ),
    text: "disabled",
  });
  const fixed = applyFixes(source, first.fixes);
  assert.equal(fixed, source.replace(':disabled="true"', "disabled"));
  assert.deepEqual(native.lintWithPlugins(fixed, [booleans], options).diagnostics, []);
});

test("cached misses audit identical batches before writing a nondeterministic result", () => {
  let calls = 0;
  const plugin = definePlugin({
    name: "changing-output",
    version: "1",
    cacheInputs: [],
    rules: {
      check(ctx) {
        ctx.report(ctx.nodes[0], String(++calls));
      },
    },
  });
  assert.throws(
    () => native.lintWithPlugins(source, [plugin], { ...options, validateDeterminism: false }),
    /identical visit batches produced different diagnostics or fixes/,
  );
  assert.equal(calls, 2);
  assert.throws(
    () => native.lintWithPlugins(source, [plugin], options),
    /identical visit batches produced different diagnostics or fixes/,
  );
  assert.equal(calls, 4);
});

test("uncached determinism audits fixes as well as messages", () => {
  let calls = 0;
  const plugin = definePlugin({
    name: "changing-fix",
    version: "1",
    rules: {
      check(ctx) {
        ctx.report(ctx.nodes[0], "same", String(++calls));
      },
    },
  });
  assert.throws(
    () => native.lintWithPlugins(source, [plugin], { validateDeterminism: true }),
    /different diagnostics or fixes/,
  );
});

test("authored design-system transform and output helper consume real native compiler artifacts", () => {
  const template = '<button class="legacy-btn" title="日本語😀">{{ label }}</button>';
  const options = { filename: "Design.vue", sourceMap: true, cache: true };
  const first = native.compileWithTransformPlugins(template, [design], options);
  const hit = native.compileWithTransformPlugins(template, [design], options);
  const expected = native.compile(template.replace('class="legacy-btn"', 'class="ds-button"'), {
    filename: options.filename,
    mode: "module",
    prefixIdentifiers: true,
    hoistStatic: false,
  });
  assert.equal(first.result.code, expected.code);
  assert.equal(first.result.preamble, expected.preamble);
  assert.deepEqual(first.result.map.sourcesContent, [template]);
  assert.equal(first.plugins[0].edits, 1);
  assert.ok(first.plugins[0].nodes > 0 && first.plugins[0].jsNs > 0);
  assert.ok(first.plugins[0].elapsedNs >= first.plugins[0].jsNs);
  assert.deepEqual(hit.result, first.result);
  assert.equal(hit.plugins[0].cached, true);
  assert.equal(hit.plugins[0].jsNs, 0);
  const license = defineOutputPlugin({
    name: "design-license",
    version: "1",
    family: "output",
    cacheInputs: [],
    output() {
      return [{ placement: "prepend", comment: "Design system" }];
    },
  });
  const output = native.applyOutputPlugins(first.result, [license], { cache: true });
  assert.equal(output.result.code, "/* Design system */\n" + first.result.code);
  assert.deepEqual(output.result.map.sourcesContent, [template]);
  assert.equal(output.plugins[0].operations, 1);
  assert.equal(
    native.applyOutputPlugins(first.result, [license], { cache: true }).plugins[0].jsNs,
    0,
  );
});

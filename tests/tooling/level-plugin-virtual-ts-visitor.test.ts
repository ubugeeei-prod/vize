import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "node:test";
import {
  VISITOR_API_VERSION,
  createVirtualTsVisitorHost,
  defineVisitorPlugin,
  type VisitorEvent,
} from "../../npm/plugin-sdk/visitor.js";

const native = createRequire(import.meta.url)("../../npm/native/index.js");
const source = `<script setup lang="ts">
const message = "hello";
</script>
<template><p>{{ message }}</p></template>`;

test("native type checking publishes generated virtual TypeScript and UTF-8 mappings", () => {
  const events: VisitorEvent<"virtual-ts">[] = [];
  const plugin = defineVisitorPlugin({
    apiVersion: VISITOR_API_VERSION,
    name: "virtual-ts-recorder",
    version: "1",
    capabilities: ["virtual-ts"] as const,
    cacheInputs: [],
    visitors: {
      "virtual-ts"(event) {
        events.push(event);
      },
    },
  });

  const result = native.typeCheckWithVirtualTsVisitors(
    source,
    [createVirtualTsVisitorHost(plugin)],
    { filename: "src/App.vue" },
  );

  assert.equal(events.length, 1);
  const [event] = events;
  assert.equal(event.file, "src/App.vue");
  assert.equal(event.payload.sourceFile, "src/App.vue");
  assert.equal(event.payload.virtualFile, "src/App.vue.ts");
  assert.equal(event.payload.code, result.virtualTs);
  assert.match(event.payload.code, /message/u);
  assert.ok(event.payload.mappings.length > 0);
  for (const mapping of event.payload.mappings) {
    assert.equal(mapping.source.encoding, "utf8");
    assert.equal(mapping.generated.encoding, "utf8");
    assert.ok(mapping.source.start <= mapping.source.end);
    assert.ok(mapping.generated.start <= mapping.generated.end);
  }
  assert.equal(Object.isFrozen(event), true);
  assert.equal(Object.isFrozen(event.payload.mappings), true);
});

test("native virtual TypeScript observers fail closed on forged identities and results", () => {
  const base = {
    name: "forged",
    version: "1",
    fingerprint: "forged-v1",
    run: () => "changed",
  };
  assert.throws(
    () => native.typeCheckWithVirtualTsVisitors(source, [base], { filename: "App.vue" }),
    /observers must return undefined/u,
  );
  assert.throws(
    () =>
      native.typeCheckWithVirtualTsVisitors(source, [{ ...base, name: "", run: () => "" }], {
        filename: "App.vue",
      }),
    /identity must be non-empty/u,
  );
});

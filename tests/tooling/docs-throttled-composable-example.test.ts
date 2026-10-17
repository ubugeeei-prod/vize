import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";
import path from "node:path";

import { repoRoot } from "./_helpers/moonbit.ts";
import { COMPOSABLE_CATALOG } from "../../npm/compose/core/src/catalog.ts";
import {
  composableExamples,
  composableExampleSource,
} from "../../npm/ui/scripts/reference-docs/composable-examples.ts";
import { renderReferenceDocs } from "../../npm/ui/scripts/generate-reference-docs.ts";
import { previewComposableExamples } from "../../docs/previews/ui/build-config.ts";
import { exerciseThrottledExample } from "../../docs/previews/ui/interactions-throttled.ts";

const originalNames = [
  "use-toggle",
  "use-counter",
  "use-debounced",
  "use-field",
  "use-history",
  "use-offset-pagination",
  "use-interval",
  "use-timeout",
  "use-async-state",
  "use-storage",
  "media-query",
  "active-element",
  "use-form",
  "use-stepper",
  "use-selection",
  "use-cycle-list",
  "focus",
  "on-click-outside",
  "on-key-stroke",
  "event-listener",
  "use-clipboard",
  "scroll",
  "resize-observer",
  "intersection-observer",
  "element-bounding",
  "element-hover",
  "on-long-press",
  "on-start-typing",
  "network",
  "color-mode",
  "breakpoints",
  "window-size",
  "pointer",
  "mouse-in-element",
  "draggable",
  "text-selection",
];

void test("the throttled preview appends one public example and retains original enrollment", () => {
  const currentNames = composableExamples.map(({ name }) => name);
  // These reviewed demos precede the frozen 123-example enrollment.
  const successors = ["textarea-autosize", "async-resource", "virtual-list"];
  assert.deepEqual(currentNames.slice(0, successors.length), successors);
  assert.equal(currentNames.length, 126);
  assert.equal(new Set(currentNames).size, 126);
  const names = currentNames.slice(successors.length);
  assert.equal(names.length, 123);
  assert.deepEqual(names.slice(0, 36), originalNames);
  assert.deepEqual(names.slice(36, 37), ["use-throttled"]);
  // Later examples append after the throttled preview.
  assert.deepEqual(names.slice(37, 44), [
    "use-number-format",
    "use-list-format",
    "use-plural-rules",
    "use-machine",
    "use-previous",
    "use-countdown",
    "use-relative-time-format",
  ]);
  assert.equal(new Set(names).size, 123);
  const entry = COMPOSABLE_CATALOG.entries.find(({ subpath }) => subpath === "./use-throttled");
  assert.ok(entry);
  assert.deepEqual(entry.runtimeExports, ["useThrottled"]);
  const utility = COMPOSABLE_CATALOG.utilities.find(({ name }) => name === "useThrottled");
  assert.ok(utility);
  assert.equal(utility.stability, "experimental");
  assert.equal(utility.ssr, "deterministic-fallback");
  assert.equal(utility.hydration, "stable");
});

void test("the throttled reference and preview retain the complete same public-import SFC", () => {
  const source = composableExampleSource(path.join(repoRoot, "npm/compose/core"), "use-throttled");
  const preview = previewComposableExamples.find(({ name }) => name === "use-throttled");
  assert.equal(preview?.source, source);
  assert.equal(preview?.sourceSha256, createHash("sha256").update(source).digest("hex"));
  const rendered = renderReferenceDocs();
  const page = rendered.get("guide/composables/use-throttled.md") ?? "";
  assert.ok(page.includes(source.trim()));
  assert.ok(page.includes("index.html?composable=use-throttled"));
  const example = composableExamples.find(({ name }) => name === "use-throttled");
  assert.ok(example);
  for (const text of [example.purpose, example.observe, example.context])
    assert.ok(page.includes(text));
  for (const prefix of ["", "ja/"]) {
    const hub = rendered.get(`${prefix}guide/composables/index.md`) ?? "";
    assert.ok(hub.includes("use-throttled.md)"));
    assert.match(hub, /vp install @vizejs\/composable/);
  }
  assert.match(source, /from "@vizejs\/composable\/use-throttled"/);
  assert.doesNotMatch(source, /\b(?:setTimeout|setInterval|fetch)\s*\(/);
  assert.doesNotMatch(source, /runOnServer|scheduler:/);
});

void test("the throttled law refuses acceptance without actual DOM access", async () => {
  const boundary = new Error("Actual native DOM access is required");
  const page = {
    getByRole: () => {
      throw boundary;
    },
  } as unknown as Parameters<typeof exerciseThrottledExample>[0];
  await assert.rejects(exerciseThrottledExample(page), (error) => error === boundary);
});

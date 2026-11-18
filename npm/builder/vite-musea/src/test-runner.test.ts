import assert from "node:assert/strict";
import test from "node:test";

import { formatMuseaTestFailure, registerMuseaTests, type MuseaTestResult } from "./test-runner.ts";
import type { ArtFileInfo } from "./types/index.ts";

void test("interaction failures retain the authored case and structured trace", () => {
  const result: MuseaTestResult = {
    artPath: "/project/Counter.art.vue",
    variantName: "Interactive",
    status: "failed",
    duration: 4,
    error: "Expected: Count: 0 to match Count: 1",
    trace: [
      {
        kind: "action",
        name: "click",
        target: "button.increment",
        status: "passed",
        duration: 1,
      },
      {
        kind: "assertion",
        name: "Count: 1 is visible",
        status: "failed",
        duration: 0,
      },
    ],
  };

  const message = formatMuseaTestFailure(result);
  assert.match(message, /Counter\.art\.vue › Interactive/);
  assert.match(message, /✓ click · button\.increment/);
  assert.match(message, /✗ Count: 1 is visible/);
});

void test("the Vitest adapter registers only variants with authored play functions", () => {
  const names: string[] = [];
  const hooks: string[] = [];
  const art: ArtFileInfo = {
    path: "/project/Counter.art.vue",
    metadata: { title: "Counter", tags: [], status: "ready" },
    variants: [
      {
        name: "Interactive",
        template: "<Counter />",
        isDefault: true,
        skipVrt: false,
        play: "async () => {}",
      },
      {
        name: "Static",
        template: "<Counter />",
        isDefault: false,
        skipVrt: false,
      },
    ],
    hasScriptSetup: true,
    hasScript: false,
    styleCount: 0,
  };

  registerMuseaTests(
    {
      describe(name, factory) {
        names.push(name);
        factory();
      },
      test(name) {
        names.push(name);
      },
      beforeAll() {
        hooks.push("beforeAll");
      },
      afterAll() {
        hooks.push("afterAll");
      },
    },
    { artFiles: [art], baseUrl: "http://localhost:5173" },
  );

  assert.deepEqual(names, ["Musea", "Counter › Interactive"]);
  assert.deepEqual(hooks, ["beforeAll", "afterAll"]);
});

import assert from "node:assert/strict";
import test from "node:test";

import {
  formatMuseaTestFailure,
  MuseaTestRunner,
  registerMuseaTests,
  type MuseaTestResult,
} from "./test-runner.ts";
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

void test("the Vitest adapter uses concurrent cases when the installed Vitest supports them", () => {
  const registered: string[] = [];
  const concurrent: string[] = [];
  const register = Object.assign((name: string) => registered.push(name), {
    concurrent: (name: string) => concurrent.push(name),
  });

  registerMuseaTests(
    {
      describe(_name, factory) {
        factory();
      },
      test: register,
      beforeAll() {},
      afterAll() {},
    },
    { artFiles: [playArt(["First", "Second"])], baseUrl: "http://localhost:5173" },
  );

  assert.deepEqual(registered, []);
  assert.deepEqual(concurrent, ["Counter › First", "Counter › Second"]);
});

void test("interaction workers stream completed cases and retain authored result order", async () => {
  class StubRunner extends MuseaTestRunner {
    active = 0;
    maxActive = 0;

    override async runTest(
      art: ArtFileInfo,
      variantName: string,
      _baseUrl: string,
    ): Promise<MuseaTestResult> {
      this.active++;
      this.maxActive = Math.max(this.maxActive, this.active);
      await new Promise((resolve) => setTimeout(resolve, variantName === "First" ? 20 : 1));
      this.active--;
      return {
        artPath: art.path,
        variantName,
        status: "passed",
        duration: 1,
        trace: [],
      };
    }
  }

  const runner = new StubRunner({ workers: 2 });
  const completed: string[] = [];
  const results = await runner.runAllTests(
    [playArt(["First", "Second"])],
    "http://localhost:5173",
    (result) => completed.push(result.variantName),
  );

  assert.equal(runner.maxActive, 2);
  assert.deepEqual(completed, ["Second", "First"]);
  assert.deepEqual(
    results.map((result) => result.variantName),
    ["First", "Second"],
  );
});

function playArt(names: string[]): ArtFileInfo {
  return {
    path: "/project/Counter.art.vue",
    metadata: { title: "Counter", tags: [], status: "ready" },
    variants: names.map((name, index) => ({
      name,
      template: "<Counter />",
      isDefault: index === 0,
      skipVrt: false,
      play: "async () => {}",
    })),
    hasScriptSetup: true,
    hasScript: false,
    styleCount: 0,
  };
}

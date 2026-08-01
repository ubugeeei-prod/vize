import assert from "node:assert/strict";
import test from "node:test";

import { collectNuxtLintDirs } from "@vizejs/nuxt-lint-config";
import { toNuxtLintProjectState } from "./nuxt-state.ts";

void test("Nuxt 3 and 4 layers keep their order and their own source directories", () => {
  assert.deepEqual(
    toNuxtLintProjectState({
      rootDir: "/project",
      srcDir: "/project/app",
      dir: { pages: "routes" },
      _layers: [
        { config: { srcDir: "/project/app" } },
        { config: { srcDir: "/project/layers/base", components: ["./ui"] } },
      ],
    }),
    {
      rootDir: "/project",
      dir: { pages: "routes" },
      layers: [
        { srcDir: "/project/app" },
        { srcDir: "/project/layers/base", components: ["./ui"] },
      ],
    },
  );
});

void test("a Nuxt 2 project with no layers becomes a single srcDir layer", () => {
  assert.deepEqual(toNuxtLintProjectState({ rootDir: "/project", srcDir: "/project/src" }), {
    rootDir: "/project",
    dir: {},
    layers: [{ srcDir: "/project/src" }],
  });
});

void test("a project with neither layers nor srcDir falls back to the root", () => {
  assert.deepEqual(toNuxtLintProjectState({ rootDir: "/project" }), {
    rootDir: "/project",
    dir: {},
    layers: [{ srcDir: "/project" }],
  });
});

void test("a layer without its own srcDir inherits the project srcDir", () => {
  assert.deepEqual(
    toNuxtLintProjectState({
      rootDir: "/project",
      srcDir: "/project/app",
      _layers: [{ config: { components: true } }],
    }),
    {
      rootDir: "/project",
      dir: {},
      layers: [{ srcDir: "/project/app", components: true }],
    },
  );
});

void test("overriding rootDir re-anchors every emitted directory", () => {
  const state = toNuxtLintProjectState(
    {
      rootDir: "/project",
      srcDir: "/project/app",
      _layers: [{ config: { srcDir: "/project/app" } }],
    },
    { rootDir: "/project/tooling" },
  );

  assert.deepEqual(collectNuxtLintDirs(state), {
    pages: ["../app/pages"],
    composables: ["../app/composables", "../app/utils"],
    components: ["../app/components"],
    componentsPrefixed: [],
    layouts: ["../app/layouts"],
    plugins: ["../app/plugins"],
    middleware: ["../app/middleware"],
    modules: ["../app/modules"],
    servers: [],
    root: [],
    src: ["../app"],
  });
});

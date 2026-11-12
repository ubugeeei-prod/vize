/**
 * #12871: Nuxt pre-compiles components in `buildStart` against the `.nuxt`
 * tsconfigs, then rewrites them while the dev server starts. Its watcher
 * ignores the build directory, so no hot update evicts the pre-compiled
 * module, and the first request must not fail on the stale project.
 */

import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";

import { compileAll } from "./precompile-run.ts";
import { resetTypeProjectWatches } from "./type-project-watches.ts";
import { loadSfcModule } from "../test/sfc-module.ts";
import { makeState } from "../test/precompile-cache-project.ts";

const COMPONENT = `<script setup lang="ts">
import type { GreetingProps } from "./greeting";

defineProps<GreetingProps>();
</script>

<template>
  <p>{{ msg }}</p>
</template>
`;

const PREPARED = JSON.stringify({ compilerOptions: { strict: true }, exclude: ["../.output"] });
const DEV = JSON.stringify({ compilerOptions: { strict: true }, exclude: ["dev"] });

function project(t: TestContext) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-stale-type-project-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const config = path.join(root, "tsconfig.json");
  const file = path.join(root, "src", "Greeting.vue");
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(config, PREPARED);
  fs.writeFileSync(
    path.join(root, "src", "greeting.ts"),
    "export interface GreetingProps { msg: string }\n",
  );
  fs.writeFileSync(file, COMPONENT);
  // Nuxt's watcher ignores its build directory: watches are accepted, never reported.
  const state = { ...makeState(root), server: { watcher: { add() {} } } as never };
  return { root, config, file, state };
}

/** Both environments' `buildStart`, as a Nuxt dev server runs them. */
async function buildStart(state: ReturnType<typeof project>["state"]) {
  for (const ssr of [false, true]) {
    resetTypeProjectWatches(state.cache);
    await compileAll(state, [], undefined, () => {}, ssr);
  }
}

function revision(state: ReturnType<typeof project>["state"], file: string) {
  const compiled = state.cache.get(file);
  assert.ok(compiled?.typeProject, "the component compiles against a TypeScript project");
  return compiled.typeProject.revision;
}

function moveMtime(file: string) {
  const later = new Date(fs.statSync(file).mtimeMs + 5000);
  fs.utimesSync(file, later, later);
}

void test("a pre-compiled module recompiles when Nuxt rewrites its tsconfig", async (t) => {
  const { config, file, state } = project(t);
  await buildStart(state);
  const precompiled = revision(state, file);

  // `nuxt dev` writes a different tsconfig than `nuxt prepare` did.
  fs.writeFileSync(config, DEV);
  const result = await loadSfcModule(state, file, { addWatchFile() {} });
  assert.ok(result, "the request is served");
  assert.match(result.code, /msg:\s*\{/, "the imported props type is resolved");
  assert.notEqual(revision(state, file), precompiled, "compiled against the current project");

  // A later request reuses the fresh compile.
  const current = revision(state, file);
  assert.ok(await loadSfcModule(state, file, { addWatchFile() {} }));
  assert.equal(revision(state, file), current);
});

void test("a content-identical tsconfig rewrite keeps the pre-compiled module", async (t) => {
  const { config, file, state } = project(t);
  await buildStart(state);
  const precompiled = revision(state, file);

  // Nitro rewrites its tsconfig with the same bytes on every build.
  fs.writeFileSync(config, PREPARED);
  moveMtime(config);
  const result = await loadSfcModule(state, file, { addWatchFile() {} });
  assert.ok(result, "the request is served");
  assert.match(result.code, /msg:\s*\{/);
  assert.equal(revision(state, file), precompiled, "an identical rewrite is no change");
});

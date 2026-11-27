/**
 * The client hot updates of SFCs that change together compile in native
 * batches: how many native calls a wave makes, and what each file's hook still
 * gets from the batch.
 */

// Must stay the first import: it wraps the native compile functions the plugin captures.
import {
  labelledComponent,
  scratchProject,
  takeNativeCompileCalls,
} from "../test/native-compile-calls.ts";

import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import type { HmrContext, ModuleNode } from "vite";

import { compileFile } from "../compiler.ts";
import { toPluginVisibleVirtualId } from "../virtual.ts";
import { makeState } from "../test/precompile-cache-project.ts";
import { observeConsole } from "../test/startup-scan-project.ts";
import { handleHotUpdateHook } from "./hmr.ts";
import { getCompileOptionsForRequest, type VizePluginState } from "./state.ts";

const names = (count: number) => Array.from({ length: count }, (_, i) => `Item${i}.vue`);
const components = (count: number, edit = "") =>
  Object.fromEntries(names(count).map((name) => [name, labelledComponent(`${name}${edit}`)]));

/** A dev server state whose client cache holds every file of `project` as first compiled. */
function servedState(project: ReturnType<typeof scratchProject>, files: string[]) {
  const state = makeState(project.root);
  for (const name of files) {
    const file = project.file(name);
    compileFile(file, state.cache, getCompileOptionsForRequest(state, false, file));
  }
  takeNativeCompileCalls(project.root);
  return state;
}

/** Each file's loaded client module, and the client hot update of a file. */
function clientGraph(state: VizePluginState) {
  const modules = new Map<string, ModuleNode>();
  const moduleOf = (id: string) => {
    let module = modules.get(id);
    if (!module) modules.set(id, (module = { id, url: id } as ModuleNode));
    return module;
  };
  const errors: unknown[] = [];
  const hotUpdate = (file: string) =>
    handleHotUpdateHook(
      state,
      {
        file,
        modules: [],
        read: async () => fs.readFileSync(file, "utf8"),
        server: {
          moduleGraph: {
            getModuleById: (id: string) =>
              id === toPluginVisibleVirtualId(file) ? moduleOf(id) : undefined,
            getModulesByFile: () => undefined,
            invalidateModule() {},
          },
          ws: { send() {} },
        },
      } as unknown as HmrContext,
      { onRecompileError: (error) => errors.push(error) },
    );
  return { hotUpdate, moduleOf, errors };
}

void test("SFCs edited together compile in one native batch, as each compiles alone", async () => {
  const project = scratchProject(components(6));
  try {
    const state = servedState(project, names(6));
    const { hotUpdate, moduleOf } = clientGraph(state);
    for (const [name, source] of Object.entries(components(6, " edited"))) {
      project.write(name, source);
    }

    const updates = await Promise.all(names(6).map((name) => hotUpdate(project.file(name))));

    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "batch", ssr: false, files: names(6) },
    ]);
    assert.deepEqual(
      updates,
      names(6).map((name) => [moduleOf(toPluginVisibleVirtualId(project.file(name)))]),
    );
    for (const name of names(6)) {
      const file = project.file(name);
      assert.equal(state.pendingHmrUpdateTypes.get(file), "full-reload");
      const alone = compileFile(file, new Map(), getCompileOptionsForRequest(state, false, file));
      assert.deepEqual(state.cache.get(file), alone, `${name} compiles as it does alone`);
    }
  } finally {
    project.cleanup();
  }
});

void test("a single edit still compiles alone", async () => {
  const project = scratchProject(components(3));
  try {
    const state = servedState(project, names(3));
    const { hotUpdate } = clientGraph(state);
    project.write("Item1.vue", labelledComponent("Item1 edited"));

    await hotUpdate(project.file("Item1.vue"));

    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "single", ssr: false, files: ["Item1.vue"] },
    ]);
  } finally {
    project.cleanup();
  }
});

void test("a wave larger than precompileBatchSize compiles in batches of that size", async () => {
  const project = scratchProject(components(6));
  try {
    const state = servedState(project, names(6));
    state.precompileBatchSize = 4;
    const { hotUpdate } = clientGraph(state);
    for (const [name, source] of Object.entries(components(6, " edited"))) {
      project.write(name, source);
    }

    await Promise.all(names(6).map((name) => hotUpdate(project.file(name))));

    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "batch", ssr: false, files: names(6).slice(0, 4) },
      { api: "batch", ssr: false, files: names(6).slice(4) },
    ]);
  } finally {
    project.cleanup();
  }
});

void test("a compiler error fails only its own file's hot update", async () => {
  const project = scratchProject(components(3));
  try {
    const state = servedState(project, names(3));
    const { hotUpdate, moduleOf, errors } = clientGraph(state);
    const previous = state.cache.get(project.file("Item1.vue"));
    project.write("Item0.vue", labelledComponent("Item0 edited"));
    project.write("Item1.vue", "<template><div></template>\n");
    project.write("Item2.vue", labelledComponent("Item2 edited"));

    const observed = await observeConsole(() =>
      Promise.all(names(3).map((name) => hotUpdate(project.file(name)))),
    );

    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "batch", ssr: false, files: names(3) },
    ]);
    const [edited0, failed, edited2] = observed.value ?? [];
    assert.deepEqual(edited0, [moduleOf(toPluginVisibleVirtualId(project.file("Item0.vue")))]);
    assert.equal(failed, undefined, "the failed file keeps Vite's own handling");
    assert.deepEqual(edited2, [moduleOf(toPluginVisibleVirtualId(project.file("Item2.vue")))]);
    assert.equal(errors.length, 1);
    assert.match(String(errors[0]), /Compilation failed in .*Item1\.vue/u);
    assert.equal(state.cache.get(project.file("Item1.vue")), previous);
  } finally {
    project.cleanup();
  }
});

void test("a file reported twice in one wave compiles once and updates once", async () => {
  const project = scratchProject(components(2));
  try {
    const state = servedState(project, names(2));
    const { hotUpdate, moduleOf } = clientGraph(state);
    project.write("Item0.vue", labelledComponent("Item0 edited"));
    project.write("Item1.vue", labelledComponent("Item1 edited"));
    const item0 = project.file("Item0.vue");

    const updates = await Promise.all([
      hotUpdate(item0),
      hotUpdate(project.file("Item1.vue")),
      hotUpdate(item0),
    ]);

    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "batch", ssr: false, files: names(2) },
    ]);
    // Compiled one after the other, the second report found nothing to update.
    assert.deepEqual(updates[0], [moduleOf(toPluginVisibleVirtualId(item0))]);
    assert.deepEqual(updates[2], []);
  } finally {
    project.cleanup();
  }
});

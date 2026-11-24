/**
 * Compiles follow the module graph and share native batches (#13559, #13565):
 * how many native calls a set of requests makes, and what each file's own
 * request still gets when another file's batch compiled it.
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
import type { HmrContext } from "vite";

import { compileFile, VizeSfcCompileError } from "../compiler.ts";
import { makeState } from "../test/precompile-cache-project.ts";
import { loadSfcModule } from "../test/sfc-module.ts";
import { observeConsole } from "../test/startup-scan-project.ts";
import { handleHotUpdateHook } from "./hmr.ts";
import { loadHook } from "./load.ts";
import { getSfcModuleRequest, requestResolvedSfc, transformSfcModule } from "./sfc-module.ts";
import { getCompileOptionsForRequest, getEnvironmentCache } from "./state.ts";

const names = (count: number) => Array.from({ length: count }, (_, i) => `Item${i}.vue`);
const components = (count: number) =>
  Object.fromEntries(names(count).map((name) => [name, labelledComponent(name)]));

void test("one request compiles everything its environment resolved, and nothing else", async () => {
  const project = scratchProject({ ...components(4), "Unused.vue": labelledComponent("unused") });
  try {
    const state = makeState(project.root);
    for (const name of names(4)) requestResolvedSfc(state, project.file(name), false);
    assert.deepEqual(takeNativeCompileCalls(project.root), [], "resolving compiles nothing");

    await loadSfcModule(state, project.file("Item0.vue"));
    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "batch", ssr: false, files: names(4) },
    ]);
    for (const name of names(4).slice(1)) await loadSfcModule(state, project.file(name));
    assert.deepEqual(takeNativeCompileCalls(project.root), [], "siblings reuse the batch");
    assert.deepEqual([...state.cache.keys()].sort(), names(4).map(project.file));
    assert.deepEqual([...state.ssrCache.keys()], [], "the SSR environment requested nothing");
  } finally {
    project.cleanup();
  }
});

void test("a loaded file joins the batch when its resolution was not observed", async () => {
  const project = scratchProject(components(3));
  try {
    const state = makeState(project.root);
    // A rebuild loads modules whose resolution the bundler kept from the last build.
    for (const name of names(3)) assert.notEqual(loadHook(state, project.file(name)), null);
    assert.deepEqual(takeNativeCompileCalls(project.root), [], "loading compiles nothing");
    for (const name of names(3).reverse()) await loadSfcModule(state, project.file(name));
    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "batch", ssr: false, files: ["Item2.vue", "Item0.vue", "Item1.vue"] },
    ]);
  } finally {
    project.cleanup();
  }
});

void test("a batch holds at most precompileBatchSize files", async () => {
  const project = scratchProject(components(5));
  try {
    const state = makeState(project.root);
    state.precompileBatchSize = 2;
    for (const name of names(5)) requestResolvedSfc(state, project.file(name), false);
    for (const name of names(5)) await loadSfcModule(state, project.file(name));
    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "batch", ssr: false, files: ["Item0.vue", "Item1.vue"] },
      { api: "batch", ssr: false, files: ["Item2.vue", "Item3.vue"] },
      { api: "single", ssr: false, files: ["Item4.vue"] },
    ]);
  } finally {
    project.cleanup();
  }
});

void test("concurrent requests for the files of a running batch await it", async () => {
  const project = scratchProject(components(3));
  try {
    const state = makeState(project.root);
    for (const name of names(3)) requestResolvedSfc(state, project.file(name), false);
    // A resolver makes the batch yield before it compiles, as a bundler's does.
    const results = await Promise.all(
      names(3).map((name) =>
        loadSfcModule(state, project.file(name), { resolve: async () => null }),
      ),
    );
    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "batch", ssr: false, files: names(3) },
    ]);
    assert.deepEqual(
      results.map((result) => result?.code.includes('"Item') ?? false),
      [true, true, true],
    );
  } finally {
    project.cleanup();
  }
});

void test("each environment compiles its own output, identical to a standalone compile", async () => {
  const project = scratchProject(components(2));
  try {
    const state = makeState(project.root);
    const served: string[] = [];
    for (const ssr of [false, true]) {
      for (const name of names(2)) requestResolvedSfc(state, project.file(name), ssr);
      const batched = await loadSfcModule(state, project.file("Item1.vue"), { ssr });
      assert.deepEqual(takeNativeCompileCalls(project.root), [
        { api: "batch", ssr, files: ["Item1.vue", "Item0.vue"] },
      ]);
      // Item0 was compiled from disk as part of Item1's batch.
      for (const name of names(2)) {
        const file = project.file(name);
        const standalone = compileFile(
          file,
          new Map(),
          getCompileOptionsForRequest(state, ssr, file),
          fs.readFileSync(file, "utf8"),
        );
        assert.deepEqual(getEnvironmentCache(state, ssr).get(file), standalone, `${name} ${ssr}`);
      }
      takeNativeCompileCalls(project.root);
      for (const name of names(2)) {
        const alone = await loadSfcModule(makeState(project.root), project.file(name), { ssr });
        const fromBatch =
          name === "Item1.vue" ? batched : await loadSfcModule(state, project.file(name), { ssr });
        assert.deepEqual(fromBatch, alone, `${name} is served as its standalone compile is`);
      }
      assert.deepEqual(takeNativeCompileCalls(project.root), [
        { api: "single", ssr, files: ["Item0.vue"] },
        { api: "single", ssr, files: ["Item1.vue"] },
      ]);
      served.push(batched!.code);
    }
    assert.notEqual(served[0], served[1], "client and SSR modules differ");
    assert.deepEqual(
      [state.cache.size, state.ssrCache.size],
      [2, 2],
      "each environment keeps its own modules",
    );
  } finally {
    project.cleanup();
  }
});

void test("a compile error is raised by the failing file's own request", async () => {
  const project = scratchProject({ ...components(1), "Broken.vue": "<!-- TODO: implement -->\n" });
  const diagnostics = [
    "At least one <template> or <script> is required in a single file component.",
  ];
  try {
    const state = makeState(project.root);
    for (const name of ["Item0.vue", "Broken.vue"]) {
      requestResolvedSfc(state, project.file(name), false);
    }
    const sibling = await observeConsole(() => loadSfcModule(state, project.file("Item0.vue")));
    assert.equal(sibling.error, null, "another file's error must not fail this module");
    assert.deepEqual(sibling.calls, { log: [], warn: [], error: [] });
    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "batch", ssr: false, files: ["Item0.vue", "Broken.vue"] },
    ]);

    const broken = await observeConsole(() => loadSfcModule(state, project.file("Broken.vue")));
    assert.ok(broken.error instanceof VizeSfcCompileError);
    assert.equal(broken.error.filePath, project.file("Broken.vue"));
    assert.deepEqual(broken.error.diagnostics, diagnostics);
    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "single", ssr: false, files: ["Broken.vue"] },
    ]);

    // The requested file's own error comes out of the batch it started.
    const fresh = makeState(project.root);
    for (const name of ["Item0.vue", "Broken.vue"]) {
      requestResolvedSfc(fresh, project.file(name), false);
    }
    const requested = await observeConsole(() => loadSfcModule(fresh, project.file("Broken.vue")));
    assert.ok(requested.error instanceof VizeSfcCompileError);
    assert.equal(requested.error.message, broken.error.message);
    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "batch", ssr: false, files: ["Broken.vue", "Item0.vue"] },
    ]);
    assert.equal(fresh.cache.has(project.file("Item0.vue")), true);
  } finally {
    project.cleanup();
  }
});

void test("a batched file's warnings are printed by its own request, as a standalone compile prints them", async () => {
  const warned = `<script setup lang="ts">
import { reactive } from "vue";
const form = reactive({ name: "" });
</script>

<template><input v-model="form" /></template>
`;
  const project = scratchProject({ ...components(1), "Warned.vue": warned });
  try {
    const standalone = await observeConsole(() =>
      loadSfcModule(makeState(project.root), project.file("Warned.vue")),
    );
    assert.equal(standalone.calls.warn.length > 0, true, "the fixture must warn");

    const state = makeState(project.root);
    for (const name of ["Item0.vue", "Warned.vue"]) {
      requestResolvedSfc(state, project.file(name), false);
    }
    takeNativeCompileCalls(project.root);
    const sibling = await observeConsole(() => loadSfcModule(state, project.file("Item0.vue")));
    assert.deepEqual(sibling.calls.warn, [], "warnings wait for the file's own request");
    const own = await observeConsole(() => loadSfcModule(state, project.file("Warned.vue")));
    assert.deepEqual(own.calls.warn, standalone.calls.warn);
    assert.deepEqual(own.value, standalone.value);
    const again = await observeConsole(() => loadSfcModule(state, project.file("Warned.vue")));
    assert.deepEqual(again.calls.warn, [], "a reused module does not repeat its warnings");
    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "batch", ssr: false, files: ["Item0.vue", "Warned.vue"] },
    ]);
  } finally {
    project.cleanup();
  }
});

void test("text a host transform rewrote is compiled by that file's own request", async () => {
  const project = scratchProject(components(2));
  try {
    const state = makeState(project.root);
    for (const name of names(2)) requestResolvedSfc(state, project.file(name), false);
    await loadSfcModule(state, project.file("Item0.vue"));
    takeNativeCompileCalls(project.root);

    const file = project.file("Item1.vue");
    const rewritten = labelledComponent("rewritten by a host transform");
    const result = await transformSfcModule(state, rewritten, getSfcModuleRequest(state, file)!);
    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "single", ssr: false, files: ["Item1.vue"] },
    ]);
    assert.equal(state.cache.get(file)?.source, rewritten);
    assert.equal(result.code.includes("rewritten by a host transform"), true);
  } finally {
    project.cleanup();
  }
});

void test("a router's page macro modules compile the resolved pages in one batch", async () => {
  const page = (name: string) => `<script setup lang="ts">
definePage({ name: "${name}", meta: { requiresAuth: true } });
</script>

<template><main>${name}</main></template>
`;
  const pages = ["Home.vue", "About.vue", "Settings.vue"];
  const project = scratchProject(Object.fromEntries(pages.map((name) => [name, page(name)])));
  try {
    const state = makeState(project.root);
    // A routes module imports each page's macro module and, lazily, the page.
    for (const name of pages) requestResolvedSfc(state, project.file(name), false);
    const records = pages.map((name) => loadHook(state, `\0${project.file(name)}?definePage`));
    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "batch", ssr: false, files: pages },
    ]);
    assert.deepEqual(
      records.map((record) => typeof record === "object" && record?.code.includes("requiresAuth")),
      [true, true, true],
    );
    for (const name of pages) await loadSfcModule(state, project.file(name));
    assert.deepEqual(takeNativeCompileCalls(project.root), [], "the pages reuse the batch");
  } finally {
    project.cleanup();
  }
});

void test("a hot update recompiles the edited file once, and its dependents in one batch", async () => {
  const owner = (name: string) =>
    `<script setup lang="ts">\nconst name = "${name}";\n</script>\n\n<template src="./shared.html"></template>\n`;
  const project = scratchProject({
    "Item0.vue": owner("Item0"),
    "Item1.vue": owner("Item1"),
    "Item2.vue": labelledComponent("Item2"),
    "shared.html": "<p>{{ name }}</p>\n",
  });
  const hotUpdate = (file: string) =>
    handleHotUpdateHook(state, {
      file,
      modules: [],
      read: async () => fs.readFileSync(file, "utf8"),
      server: {
        moduleGraph: { getModulesByFile: () => undefined, invalidateModule() {} },
        ws: { send() {} },
      },
    } as unknown as HmrContext);
  const state = makeState(project.root);
  try {
    for (const ssr of [false, true]) {
      for (const name of names(3)) requestResolvedSfc(state, project.file(name), ssr);
      for (const name of names(3)) await loadSfcModule(state, project.file(name), { ssr });
    }
    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "batch", ssr: false, files: names(3) },
      { api: "batch", ssr: true, files: names(3) },
    ]);

    // An edited component: the client hook compiles it, the SSR request compiles it.
    project.write("Item2.vue", labelledComponent("Item2 edited"));
    await hotUpdate(project.file("Item2.vue"));
    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "single", ssr: false, files: ["Item2.vue"] },
    ]);
    for (const ssr of [false, true]) {
      for (const name of names(3)) await loadSfcModule(state, project.file(name), { ssr });
    }
    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "single", ssr: true, files: ["Item2.vue"] },
    ]);

    // An edited dependency: its owners are evicted, and requested again.
    project.write("shared.html", "<p>{{ name }} edited</p>\n");
    await hotUpdate(project.file("shared.html"));
    assert.deepEqual(takeNativeCompileCalls(project.root), [], "eviction compiles nothing");
    for (const ssr of [false, true]) {
      // A dev server resolves a URL again when it is requested again.
      for (const name of names(3)) requestResolvedSfc(state, project.file(name), ssr);
      for (const name of names(3)) await loadSfcModule(state, project.file(name), { ssr });
    }
    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "batch", ssr: false, files: ["Item0.vue", "Item1.vue"] },
      { api: "batch", ssr: true, files: ["Item0.vue", "Item1.vue"] },
    ]);
    for (const ssr of [false, true]) {
      const compiled = getEnvironmentCache(state, ssr).get(project.file("Item1.vue"));
      assert.equal(compiled?.code.includes("edited"), true, `ssr=${ssr} serves the edit`);
    }
  } finally {
    project.cleanup();
  }
});

void test("a batched component serves the same styles as its standalone compile", async () => {
  const styled = (name: string, attributes: string) =>
    `${labelledComponent(name)}\n<style${attributes}>\n.${name} { color: red; }\n</style>\n`;
  const project = scratchProject({
    "A.vue": styled("a", ""),
    "B.vue": styled("b", " scoped"),
    "C.vue": styled("c", " module"),
  });
  const styleImports = (code: string) =>
    [...code.matchAll(/^import (?:\w+ from )?"([^"]+type=style[^"]+)";$/gm)].map(
      (match) => match[1],
    );
  try {
    for (const production of [false, true]) {
      const batched = makeState(project.root);
      const alone = makeState(project.root);
      for (const state of [batched, alone]) {
        state.isProduction = production;
        state.extractCss = production;
      }
      for (const name of ["A.vue", "B.vue", "C.vue"]) {
        requestResolvedSfc(batched, project.file(name), false);
      }
      // A's request compiles all three; C is never requested, so it is never rendered.
      for (const name of ["A.vue", "B.vue"]) {
        const fromBatch = await loadSfcModule(batched, project.file(name));
        assert.deepEqual(fromBatch, await loadSfcModule(alone, project.file(name)));
        const imports = styleImports(fromBatch!.code);
        assert.equal(imports.length, 1, `${name} imports its style module`);
        const style = loadHook(batched, imports[0]);
        assert.notEqual(style, null);
        assert.deepEqual(style, loadHook(alone, imports[0]));
      }
      assert.deepEqual([...batched.collectedCss], [...alone.collectedCss]);
      assert.equal(batched.collectedCss.has(project.file("C.vue")), false);
    }
  } finally {
    project.cleanup();
  }
});

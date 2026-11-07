import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import type { HmrContext, ModuleNode } from "vite";

import { compileFile } from "../compiler.ts";
import { toPluginVisibleVirtualId } from "../virtual.ts";
import { CompiledModuleCache, ownersOfDependency, ownersOfFile } from "./compiled-module-cache.ts";
import { handleHotUpdateHook } from "./hmr.ts";
import type { VizePluginState } from "./state.ts";

const before = `<template><p>before</p></template>`;
const after = `<template><p>after</p></template>`;

function linkDirectory(target: string, link: string): boolean {
  fs.mkdirSync(path.dirname(link), { recursive: true });
  try {
    fs.symlinkSync(target, link, process.platform === "win32" ? "junction" : "dir");
    return true;
  } catch {
    return false;
  }
}

function createState(root: string, owners: readonly string[]): VizePluginState {
  const cache = new CompiledModuleCache();
  for (const owner of owners) {
    compileFile(owner, cache, { root, sourceMap: false, ssr: false, vapor: false }, before);
  }
  return {
    cache,
    ssrCache: new CompiledModuleCache(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: false,
    mergedOptions: {},
    cssAliasRules: [],
    clientViteBase: "/",
    root,
    filter: (file) => file.replaceAll("\\", "/").includes("/apps/"),
    logger: { log() {}, error() {} },
  } as unknown as VizePluginState;
}

function hotContext(
  file: string,
  modules: ReadonlyMap<string, ModuleNode>,
  invalidated: ModuleNode[],
  read: () => Promise<string>,
): HmrContext {
  return {
    file,
    server: {
      moduleGraph: {
        getModuleById(id: string) {
          return modules.get(id);
        },
        getModulesByFile() {},
        invalidateModule(module: ModuleNode) {
          invalidated.push(module);
        },
      },
      ws: { send() {} },
    },
    read,
  } as unknown as HmrContext;
}

void test("one physical workspace SFC keeps granular HMR for every logical package owner", async (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-monorepo-hmr-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const shared = path.join(root, "packages", "shared");
  fs.mkdirSync(shared, { recursive: true });
  const physical = path.join(shared, "Card.vue");
  fs.writeFileSync(physical, before);
  const links = ["app-a", "app-b"].map((app) =>
    path.join(root, "apps", app, "node_modules", "shared"),
  );
  if (!links.every((link) => linkDirectory(shared, link))) {
    t.skip("directory symlinks are unavailable");
    return;
  }
  const owners = links.map((link) => path.join(link, "Card.vue"));
  const state = createState(root, owners);
  const optionOwners: string[] = [];
  state.fileCompilerOptions = (file) => {
    optionOwners.push(file);
    return {};
  };
  const modules = new Map(
    owners.map((owner) => {
      const id = toPluginVisibleVirtualId(owner);
      return [id, { id, url: id } as ModuleNode] as const;
    }),
  );
  const invalidated: ModuleNode[] = [];
  let reads = 0;

  const updated = await handleHotUpdateHook(
    state,
    hotContext(physical, modules, invalidated, async () => {
      reads += 1;
      return after;
    }),
  );

  assert.deepEqual(new Set(updated), new Set(modules.values()));
  assert.deepEqual(new Set(invalidated), new Set(modules.values()));
  assert.deepEqual(new Set(optionOwners), new Set(owners));
  assert.equal(reads, 1, "all logical owners must share the watcher read");
  for (const owner of owners) {
    assert.equal(state.cache.get(owner)?.source, after);
    assert.equal(state.pendingHmrUpdateTypes.get(owner), "template-only");
  }
});

void test("a physical src dependency invalidates every symlinked monorepo owner", async (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-monorepo-dependency-hmr-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const shared = path.join(root, "packages", "shared");
  fs.mkdirSync(shared, { recursive: true });
  const physicalOwner = path.join(shared, "Card.vue");
  const physicalDependency = path.join(shared, "template.html");
  fs.writeFileSync(physicalOwner, before);
  fs.writeFileSync(physicalDependency, "<p>before</p>");
  const links = ["app-a", "app-b"].map((app) =>
    path.join(root, "apps", app, "node_modules", "shared"),
  );
  if (!links.every((link) => linkDirectory(shared, link))) {
    t.skip("directory symlinks are unavailable");
    return;
  }
  const owners = links.map((link) => path.join(link, "Card.vue"));
  const state = createState(root, owners);
  for (const [owner, link] of owners.map((owner, index) => [owner, links[index]] as const)) {
    state.cache.set(owner, {
      ...state.cache.get(owner)!,
      dependencies: [path.join(link, "template.html")],
    });
  }
  const modules = new Map(
    owners.map((owner) => {
      const id = toPluginVisibleVirtualId(owner);
      return [id, { id, url: id } as ModuleNode] as const;
    }),
  );
  const invalidated: ModuleNode[] = [];

  const updated = await handleHotUpdateHook(
    state,
    hotContext(physicalDependency, modules, invalidated, async () => "<p>after</p>"),
  );

  assert.deepEqual(new Set(updated), new Set(modules.values()));
  assert.deepEqual(new Set(invalidated), new Set(modules.values()));
  for (const owner of owners) {
    assert.equal(state.cache.has(owner), false);
    assert.equal(state.pendingHmrUpdateTypes.get(owner), "full-reload");
  }

  invalidated.length = 0;
  const repeated = await handleHotUpdateHook(
    state,
    hotContext(physicalDependency, modules, invalidated, async () => "<p>again</p>"),
  );
  assert.deepEqual(new Set(repeated), new Set(modules.values()));
  assert.deepEqual(new Set(invalidated), new Set(modules.values()));
});

void test("a remounted workspace link drops old physical identities before indexing its new target", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-monorepo-remount-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const versions = ["v1", "v2"].map((version) => path.join(root, "store", version));
  for (const version of versions) {
    fs.mkdirSync(version, { recursive: true });
    fs.writeFileSync(path.join(version, "Card.vue"), before);
    fs.writeFileSync(path.join(version, "template.html"), "<p />");
  }
  const link = path.join(root, "node_modules", "shared");
  if (!linkDirectory(versions[0], link)) {
    t.skip("directory symlinks are unavailable");
    return;
  }
  const owner = path.join(link, "Card.vue");
  const dependency = path.join(link, "template.html");
  const cache = new CompiledModuleCache();
  const compiled = {
    code: "export default {}",
    scopeId: "shared",
    hasScoped: false,
    dependencies: [dependency],
  };
  cache.set(owner, compiled);
  assert.deepEqual(ownersOfFile(cache, path.join(versions[0], "Card.vue")), [owner]);
  assert.deepEqual(ownersOfDependency(cache, path.join(versions[0], "template.html")), [owner]);

  fs.unlinkSync(link);
  assert.equal(linkDirectory(versions[1], link), true);
  cache.set(owner, compiled);

  assert.deepEqual(ownersOfFile(cache, path.join(versions[0], "Card.vue")), []);
  assert.deepEqual(ownersOfDependency(cache, path.join(versions[0], "template.html")), []);
  assert.deepEqual(ownersOfFile(cache, path.join(versions[1], "Card.vue")), [owner]);
  assert.deepEqual(ownersOfDependency(cache, path.join(versions[1], "template.html")), [owner]);
});

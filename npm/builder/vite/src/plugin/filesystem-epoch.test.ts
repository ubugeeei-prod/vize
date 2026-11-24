/**
 * #13557: every compiled SFC indexes the identities of its dependencies, and
 * 1,000 components typed through one barrel asked for the same `realpath`s
 * 1,000 times over. The plugin's caches share a filesystem epoch, which asks
 * about each distinct path once and ends on every sign of a change.
 */

import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import type { Plugin } from "vite";

import type { CompiledModule } from "../types.ts";
import { startupHost } from "../test/startup-scan-project.ts";
import { CompiledModuleCache, ownersOfDependency, ownersOfFile } from "./compiled-module-cache.ts";
import { FilesystemEpoch, processFilesystemEpoch } from "./filesystem-epoch.ts";
import { vize } from "./index.ts";
import { filesystemPathIdentities } from "./path-identity.ts";

/** Every `fs.realpathSync.native` call `run` makes, per path, as the issue's preload counts them. */
async function realpathCalls(run: () => unknown): Promise<Map<string, number>> {
  const native = fs.realpathSync.native;
  const calls = new Map<string, number>();
  fs.realpathSync.native = ((file: fs.PathLike, options?: fs.EncodingOption) => {
    calls.set(String(file), (calls.get(String(file)) ?? 0) + 1);
    return native(file, options);
  }) as typeof native;
  try {
    await run();
  } finally {
    fs.realpathSync.native = native;
  }
  return calls;
}

function total(calls: ReadonlyMap<string, number>): number {
  return [...calls.values()].reduce((sum, count) => sum + count, 0);
}

function once(paths: readonly string[]): Map<string, number> {
  return new Map(paths.map((file) => [file, 1]));
}

function sorted(calls: ReadonlyMap<string, number>): [string, number][] {
  return [...calls].sort(([left], [right]) => (left < right ? -1 : left > right ? 1 : 0));
}

function temporaryRoot(t: TestContext, name: string): string {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), `vize-epoch-${name}-`)));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  return root;
}

/** The issue's reproduction: components typing their props through a star barrel of modules. */
function barrelProject(t: TestContext, componentCount: number, moduleCount: number) {
  const root = temporaryRoot(t, "barrel");
  const write = (file: string, source: string) => {
    const target = path.join(root, file);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, source);
    return target;
  };
  const config = write(
    "tsconfig.json",
    JSON.stringify({
      compilerOptions: { module: "ESNext", moduleResolution: "bundler", strict: true },
      include: ["src"],
    }),
  );
  const models = Array.from({ length: moduleCount }, (_, i) =>
    write(`src/models/Model${i}.ts`, `export interface Model${i} {\n  field${i}: string;\n}\n`),
  );
  const barrel = write(
    "src/models/index.ts",
    models.map((_, i) => `export * from "./Model${i}";\n`).join(""),
  );
  const types = write(
    "src/types.ts",
    `import type { Model0 } from "./models";\n\nexport interface ItemProps extends Model0 {\n  id: string;\n  label: string;\n}\n`,
  );
  const components = Array.from({ length: componentCount }, (_, i) =>
    write(
      `src/components/Item${i}.vue`,
      `<script setup lang="ts">\nimport type { ItemProps } from "../types";\n\nconst props = defineProps<ItemProps>();\n</script>\n\n<template><div>{{ props.label }} ${i}</div></template>\n`,
    ),
  );
  return { root, config, models, barrel, types, components };
}

function compiled(dependencies: string[], revision?: string): CompiledModule {
  return {
    code: "export default {}",
    scopeId: "epoch",
    hasScoped: false,
    dependencies,
    ...(revision === undefined ? {} : { typeProject: { root: "/work/tsconfig.json", revision } }),
  };
}

function linkDirectory(target: string, link: string): boolean {
  fs.mkdirSync(path.dirname(link), { recursive: true });
  try {
    fs.symlinkSync(target, link, process.platform === "win32" ? "junction" : "dir");
    return true;
  } catch {
    return false;
  }
}

/** Two versions of a package and a workspace link to the first. */
function linkedPackage(t: TestContext) {
  const root = temporaryRoot(t, "link");
  const versions = ["v1", "v2"].map((version) => path.join(root, "store", version));
  for (const version of versions) {
    fs.mkdirSync(version, { recursive: true });
    fs.writeFileSync(path.join(version, "types.ts"), "export interface Shared {}\n");
  }
  const link = path.join(root, "node_modules", "shared");
  const remount = () => {
    fs.unlinkSync(link);
    assert.equal(linkDirectory(versions[1], link), true);
  };
  return {
    linked: linkDirectory(versions[0], link),
    remount,
    dependency: path.join(link, "types.ts"),
    physical: versions.map((version) => path.join(version, "types.ts")),
    owners: ["A.vue", "B.vue", "C.vue"].map((name) => path.join(root, name)),
  };
}

function hook(value: unknown): (this: unknown, ...args: unknown[]) => unknown {
  const handler = typeof value === "function" ? value : (value as { handler?: unknown })?.handler;
  assert.equal(typeof handler, "function");
  return handler as (this: unknown, ...args: unknown[]) => unknown;
}

function mainPlugin(): Plugin {
  const plugin = vize({ configMode: false }).find((value) => value.name === "vite-plugin-vize");
  assert.ok(plugin);
  return plugin;
}

void test("components sharing their dependencies ask about each distinct path once", async (t) => {
  const { root, config, models, barrel, types, components } = barrelProject(t, 40, 12);
  // What the native compiler reports for every component of the reproduction:
  // the type sources it read, and each tsconfig candidate it looked for.
  const absent = ["src/components/tsconfig.json", "src/tsconfig.json"].map((file) =>
    path.join(root, file),
  );
  const dependencies = [...absent, ...models, barrel, types, config];
  const index = (cache: CompiledModuleCache) => {
    for (const component of components) cache.set(component, compiled(dependencies));
    return cache;
  };

  let alone!: CompiledModuleCache;
  const before = await realpathCalls(() => (alone = index(new CompiledModuleCache())));
  // Per component: itself, each dependency, and the parent of each absent one.
  assert.equal(total(before), 40 * (1 + 17 + 2));
  assert.equal(before.size, 59);

  const epoch = new FilesystemEpoch();
  let client!: CompiledModuleCache;
  const after = await realpathCalls(() => {
    client = index(new CompiledModuleCache(epoch));
    // The SSR cache of the same build indexes the same components.
    index(new CompiledModuleCache(epoch));
  });
  assert.deepEqual(
    sorted(after),
    sorted(
      once([
        ...components,
        ...dependencies,
        path.join(root, "src/components"),
        path.join(root, "src"),
      ]),
    ),
  );
  assert.equal(total(after), 59);

  for (const dependency of [...dependencies, ...components]) {
    assert.deepEqual(ownersOfDependency(client, dependency), ownersOfDependency(alone, dependency));
    assert.deepEqual(ownersOfFile(client, dependency), ownersOfFile(alone, dependency));
  }
});

void test("an epoch answers what the filesystem answers, for present, absent and linked paths", (t) => {
  const { linked, dependency, physical } = linkedPackage(t);
  if (!linked) {
    t.skip("directory symlinks are unavailable");
    return;
  }
  const epoch = new FilesystemEpoch();
  for (const file of [
    dependency,
    physical[0],
    path.join(path.dirname(dependency), "absent", "tsconfig.json"),
    path.join(path.dirname(physical[1]), "absent.ts"),
    path.join(dependency, "below-a-file.ts"),
    "./relative.css",
    "\\\\?\\C:\\app\\src\\types.ts",
  ]) {
    const now = filesystemPathIdentities(file);
    assert.deepEqual(epoch.pathIdentities(file), now);
    assert.deepEqual(epoch.pathIdentities(file), now, "and it answers the same again");
  }
});

void test("a remounted workspace link is observed again in the next epoch", (t) => {
  const { linked, remount, dependency, physical, owners } = linkedPackage(t);
  if (!linked) {
    t.skip("directory symlinks are unavailable");
    return;
  }
  const epoch = new FilesystemEpoch();
  const cache = new CompiledModuleCache(epoch);
  cache.set(owners[0], compiled([dependency]));
  assert.deepEqual(ownersOfDependency(cache, physical[0]), [owners[0]]);

  remount();
  // The host reports the remount: a watcher event, `watchChange`, or a hot update.
  epoch.advance();
  cache.set(owners[0], compiled([dependency]));
  cache.set(owners[1], compiled([dependency]));
  assert.deepEqual(ownersOfDependency(cache, physical[0]), []);
  assert.deepEqual(ownersOfDependency(cache, physical[1]), [owners[0], owners[1]]);
  assert.deepEqual(ownersOfDependency(cache, dependency), [owners[0], owners[1]]);
});

void test("a new TypeScript project revision ends the epoch without a host event", async (t) => {
  const { linked, remount, dependency, physical, owners } = linkedPackage(t);
  if (!linked) {
    t.skip("directory symlinks are unavailable");
    return;
  }
  const epoch = new FilesystemEpoch();
  const cache = new CompiledModuleCache(epoch);
  cache.set(owners[0], compiled([dependency], "1"));
  // A second project is not a change of the first.
  const other = { ...compiled([dependency], "7"), typeProject: { root: "/other", revision: "7" } };
  assert.deepEqual(await realpathCalls(() => cache.set(owners[1], other)), once([owners[1]]));

  remount();
  // Same revision: the compile saw the project the epoch already knows.
  cache.set(owners[1], compiled([dependency], "1"));
  assert.deepEqual(ownersOfDependency(cache, physical[0]), [owners[0], owners[1]]);
  // The compiler captured the project again, as it does after Nuxt rewrites an
  // ignored `.nuxt/tsconfig.*.json` (#12871): nothing observed earlier is kept.
  cache.set(owners[2], compiled([dependency], "2"));
  assert.deepEqual(ownersOfDependency(cache, physical[1]), [owners[2]]);
  cache.set(owners[1], compiled([dependency], "2"));
  assert.deepEqual(ownersOfDependency(cache, physical[1]), [owners[2], owners[1]]);
  assert.deepEqual(ownersOfDependency(cache, physical[0]), [owners[0]]);
});

void test("every sign of a filesystem change ends the process epoch", async (t) => {
  const root = temporaryRoot(t, "signals");
  const file = path.join(root, "types.ts");
  fs.writeFileSync(file, "export interface Shared {}\n");
  const asked = () => realpathCalls(() => processFilesystemEpoch.pathIdentities(file));
  const plugin = mainPlugin();
  const watcher = new EventEmitter();
  hook(plugin.configureServer).call(plugin, {
    config: {},
    httpServer: new EventEmitter(),
    watcher,
    middlewares: { use() {} },
  });
  // A hot update looks its own file up, so it reports another one.
  const hotContext = {
    file: path.join(root, "changed.ts"),
    modules: [],
    read: async () => "",
    server: { pluginContainer: {} },
  };
  const environment = { name: "ssr", moduleGraph: {}, hot: { send() {} }, pluginContainer: {} };
  const signals: [string, () => unknown][] = [
    ["a config resolution", () => startupHost(root, { plugins: vize({ configMode: false }) })],
    ["a build without a config resolution", () => hook(mainPlugin().buildStart).call({})],
    ["watchChange", () => hook(plugin.watchChange).call({}, file, { event: "update" })],
    ["a watcher file event", () => watcher.emit("all", "change", file)],
    ["a watcher directory event", () => watcher.emit("all", "addDir", root)],
    ["hotUpdate", () => hook(plugin.hotUpdate).call({ environment }, hotContext)],
    ["handleHotUpdate", () => hook(plugin.handleHotUpdate).call({}, hotContext)],
  ];

  await asked();
  for (const [name, signal] of signals) {
    assert.deepEqual(await asked(), new Map(), `${name}: answered within the epoch`);
    await signal();
    assert.deepEqual(await asked(), once([file]), `${name}: asked again`);
  }
  // The start of a configured build is not a change: its environments share the epoch.
  const host = await startupHost(root, { plugins: vize({ configMode: false, scanPatterns: [] }) });
  await asked();
  await host.start();
  await host.close();
  assert.deepEqual(await asked(), new Map());
});

void test("the environments of one app build ask about each path once, and the next build asks again", async (t) => {
  const { root, models, barrel, types, components } = barrelProject(t, 24, 8);
  const shared = [...components, ...models, barrel, types];
  // `vite build --app` configures a plugin instance per environment, then builds them in turn.
  const appBuild = async () => {
    const environments = [
      await startupHost(root, { plugins: vize({ configMode: false }) }),
      await startupHost(root, { plugins: vize({ configMode: false }), build: { ssr: true } }),
    ];
    return realpathCalls(async () => {
      for (const environment of environments) {
        await environment.start();
        await environment.close();
      }
    });
  };

  for (const build of ["first", "next"]) {
    const calls = await appBuild();
    assert.deepEqual(
      shared.map((file) => calls.get(file)),
      shared.map(() => 1),
      `${build} build: every component and type source is asked about once`,
    );
    assert.equal(total(calls), calls.size, `${build} build: no path is asked about twice`);
  }
});

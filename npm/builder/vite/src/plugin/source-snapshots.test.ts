/**
 * #13561: an SFC the plugin already holds was stat'ed and read once per
 * environment by the pre-compile, read again by `load`, and probed again by
 * `resolveId`; and the main `load` and `transform` hooks were called for every
 * module of the graph. A build now looks at each source once, and the hooks
 * declare the ids they can act on.
 */

import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { build, type Rollup } from "vite";

import type { CompiledModule } from "../types.ts";
import { makeState } from "../test/precompile-cache-project.ts";
import { startupHost } from "../test/startup-scan-project.ts";
import { RESOLVED_CSS_MODULE, toPluginVisibleVirtualId, toVirtualId } from "../virtual.ts";
import { vize } from "./index.ts";
import { LOAD_ID_FILTER, TRANSFORM_ID_FILTER, loadHook, transformHook } from "./load.ts";
import { resolveIdHook } from "./resolve.ts";
import { RESOLVE_ID_FILTER } from "./resolve-filter.ts";
import { getSfcModuleRequest } from "./sfc-module.ts";

type Host = Awaited<ReturnType<typeof startupHost>>;
type Calls = Record<"stat" | "read" | "exists", Map<string, number>>;

/** Every `stat`, read and existence probe of a `.vue` file that `run` makes. */
async function vueFileCalls(run: () => unknown): Promise<Calls> {
  const calls: Calls = { stat: new Map(), read: new Map(), exists: new Map() };
  const count = (kind: keyof Calls, file: unknown) => {
    if (typeof file === "string" && file.endsWith(".vue")) {
      calls[kind].set(file, (calls[kind].get(file) ?? 0) + 1);
    }
  };
  const { statSync, readFileSync, existsSync } = fs;
  fs.statSync = ((file: fs.PathLike, ...rest: never[]) => {
    count("stat", file);
    return statSync(file, ...rest);
  }) as typeof statSync;
  fs.readFileSync = ((file: fs.PathOrFileDescriptor, ...rest: never[]) => {
    count("read", file);
    return readFileSync(file, ...rest);
  }) as typeof readFileSync;
  fs.existsSync = (file) => {
    count("exists", file);
    return existsSync(file);
  };
  try {
    await run();
  } finally {
    Object.assign(fs, { statSync, readFileSync, existsSync });
  }
  return calls;
}

function once(paths: readonly string[]): [string, number][] {
  return paths
    .map((file): [string, number] => [file, 1])
    .sort(([left], [right]) => order(left, right));
}

function sorted(calls: ReadonlyMap<string, number>): [string, number][] {
  return [...calls].sort(([left], [right]) => order(left, right));
}

function order(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

function component(label: string): string {
  return `<script setup lang="ts">\nconst props = defineProps<{ label: string }>();\n</script>\n\n<template><div>{{ props.label }} ${label}</div></template>\n`;
}

function project(t: TestContext, count: number) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-source-snapshot-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.mkdirSync(path.join(root, "src/components"), { recursive: true });
  const components = Array.from({ length: count }, (_, i) => {
    const file = path.join(root, `src/components/Item${i}.vue`);
    fs.writeFileSync(file, component(String(i)));
    return file;
  });
  return { root, components };
}

function hook(value: unknown): (this: unknown, ...args: unknown[]) => unknown {
  const handler = typeof value === "function" ? value : (value as { handler?: unknown })?.handler;
  assert.equal(typeof handler, "function");
  return handler as (this: unknown, ...args: unknown[]) => unknown;
}

function mainPlugin(host: Host) {
  const plugin = host.resolved.plugins.find((value) => value.name === "vite-plugin-vize");
  assert.ok(plugin);
  return plugin;
}

/** The SFC text the main plugin's `load` starts the pipeline with. */
async function loadedText(host: Host, file: string, ssr = false): Promise<string> {
  const loaded = await hook(mainPlugin(host).load).call(
    { addWatchFile() {} },
    toVirtualId(file, ssr),
    {
      ssr,
    },
  );
  assert.ok(loaded !== null && loaded !== undefined);
  return typeof loaded === "string" ? loaded : (loaded as { code: string }).code;
}

/** The opt-in startup scan, and the default: sources are read when the module graph asks. */
const scanModes = [
  { name: "startup scan", scanPatterns: ["**/*.vue"] },
  { name: "no scan", scanPatterns: undefined },
];
type ScanPatterns = (typeof scanModes)[number]["scanPatterns"];

/** `vite build --app`: a plugin instance per environment, all configured before the first builds. */
async function appBuild(root: string, scanPatterns: ScanPatterns): Promise<[Host, boolean][]> {
  const plugins = () => vize({ configMode: false, scanPatterns });
  return [
    [await startupHost(root, { plugins: plugins() }), false],
    [await startupHost(root, { plugins: plugins(), build: { ssr: true } }), true],
  ];
}

void test("one app build stats and reads each SFC once across its environments", async (t) => {
  const { root, components } = project(t, 12);
  const environments = await appBuild(root, ["**/*.vue"]);
  const calls = await vueFileCalls(async () => {
    for (const [environment, ssr] of environments) {
      await environment.start();
      for (const file of components) assert.ok(await environment.load(file, ssr));
      await environment.close();
    }
  });
  // Before: one `stat` and one read per environment's pre-compile, and one read per `load`.
  assert.deepEqual(sorted(calls.stat), once(components));
  assert.deepEqual(sorted(calls.read), once(components));
});

void test("an app build with no scan reads each SFC once across its environments", async (t) => {
  const { root, components } = project(t, 12);
  const importer = path.join(root, "src/main.ts");
  const environments = await appBuild(root, undefined);
  const calls = await vueFileCalls(async () => {
    for (const [environment, ssr] of environments) {
      await environment.start();
      // A module's imports are resolved together and loaded afterwards, so the
      // first component's request compiles all twelve in one batch.
      for (const file of components) {
        const context = { resolve: async () => null };
        assert.ok(
          await hook(mainPlugin(environment).resolveId).call(context, file, importer, { ssr }),
        );
      }
      for (const file of components) assert.ok(await environment.load(file, ssr));
      await environment.close();
    }
  });
  // Nothing is scanned, so nothing is stat'ed. The first environment probes for
  // each file once and reads it once, for its batch or its `load`, whichever
  // comes first. The second environment asks the filesystem nothing.
  assert.deepEqual(sorted(calls.stat), []);
  assert.deepEqual(sorted(calls.exists), once(components));
  assert.deepEqual(sorted(calls.read), once(components));
});

for (const { name, scanPatterns } of scanModes) {
  void test(`the next build reads its sources again (${name})`, async (t) => {
    const { root, components } = project(t, 2);
    for (const [environment, ssr] of await appBuild(root, scanPatterns)) {
      await environment.start();
      assert.equal(await loadedText(environment, components[0], ssr), component("0"));
      await environment.close();
    }

    fs.writeFileSync(components[0], component("edited"));
    const next = await appBuild(root, scanPatterns);
    const calls = await vueFileCalls(async () => {
      for (const [environment, ssr] of next) {
        await environment.start();
        assert.equal(await loadedText(environment, components[0], ssr), component("edited"));
        await environment.close();
      }
    });
    // A scan reads every file it matches; without one only the loaded file is read.
    assert.deepEqual(sorted(calls.read), once(scanPatterns ? components : [components[0]]));
  });

  void test(`an environment that builds again reads its sources again (${name})`, async (t) => {
    const { root, components } = project(t, 1);
    const host = await startupHost(root, { plugins: vize({ configMode: false, scanPatterns }) });
    await host.start();
    assert.equal(await loadedText(host, components[0]), component("0"));
    await host.close();

    // A build loop that starts the same environment again reports nothing in between.
    fs.writeFileSync(components[0], component("edited"));
    await host.start();
    assert.equal(await loadedText(host, components[0]), component("edited"));
  });

  void test(`a build loads the snapshot it took; a dev server reads the file (${name})`, async (t) => {
    const { root, components } = project(t, 1);
    for (const command of ["build", "serve"] as const) {
      const host = await startupHost(
        root,
        { plugins: vize({ configMode: false, scanPatterns }) },
        command,
      );
      if (command === "serve") {
        hook(mainPlugin(host).configureServer).call(mainPlugin(host), {
          config: host.resolved,
          httpServer: new EventEmitter(),
          watcher: new EventEmitter(),
          middlewares: { use() {} },
        });
      }
      await host.start();
      // The scan took the snapshot at start. Without one, the first load takes it.
      if (!scanPatterns) assert.equal(await loadedText(host, components[0]), component("0"));
      // No event reports this write, as for a file in a directory the watcher ignores.
      fs.writeFileSync(components[0], component("edited"));
      assert.equal(
        await loadedText(host, components[0]),
        component(command === "build" ? "0" : "edited"),
        command,
      );
      fs.writeFileSync(components[0], component("0"));
    }
  });
}

void test("a build does not probe for an SFC it compiled; a dev server does", async (t) => {
  const { root, components } = project(t, 1);
  const importer = path.join(root, "src/main.ts");
  const compiled: CompiledModule = {
    code: "export default {}",
    scopeId: "probe",
    hasScoped: false,
  };
  const context = { resolve: async () => null };
  const resolve = (state: ReturnType<typeof makeState>) =>
    resolveIdHook(context, state, "./components/Item0.vue", importer);

  let uncompiled: unknown;
  const first = await vueFileCalls(async () => (uncompiled = await resolve(makeState(root))));
  assert.deepEqual(sorted(first.exists), once(components));

  for (const cache of ["cache", "ssrCache"] as const) {
    const state = makeState(root);
    state[cache].set(components[0], compiled);
    let resolved: unknown;
    const build = await vueFileCalls(async () => (resolved = await resolve(state)));
    assert.deepEqual(sorted(build.exists), [], cache);
    assert.deepEqual(resolved, uncompiled, cache);

    const dev = await vueFileCalls(() => resolve({ ...state, server: {} as never }));
    assert.deepEqual(sorted(dev.exists), once(components), cache);
  }
});

void test("a deleted cached SFC returns to the dev resolver before its watcher reports the deletion", async (t) => {
  const {
    root,
    components: [file],
  } = project(t, 1);
  const compiled: CompiledModule = {
    code: "export default {}",
    scopeId: "deleted",
    hasScoped: false,
  };
  fs.unlinkSync(file);
  const upstream = Object.freeze({
    id: file,
    external: false,
    meta: { deletionProbe: true },
  });

  for (const kind of ["direct", "alias", "package"] as const) {
    const id =
      kind === "direct"
        ? "./components/Item0.vue"
        : kind === "alias"
          ? "@components/Item0.vue"
          : "#components/Item0.vue";
    const importer =
      kind === "package"
        ? toVirtualId(path.join(root, "src/Parent.vue"))
        : path.join(root, "src/main.ts");
    const context = { resolve: async () => (kind === "direct" ? null : upstream) };

    for (const cache of ["cache", "ssrCache"] as const) {
      for (const ssr of [false, true]) {
        for (const dev of [false, true]) {
          const state = makeState(root);
          state[cache].set(file, compiled);
          if (dev) state.server = {} as never;
          let resolved: unknown;
          const calls = await vueFileCalls(async () => {
            resolved = await resolveIdHook(context, state, id, importer, { ssr });
          });
          const label = `${kind}, ${cache}, ssr=${ssr}, dev=${dev}`;

          if (dev) {
            assert.equal(calls.exists.get(file), 1, label);
            // A package resolver's answer still belongs to the host, including
            // metadata the host needs; Vize must not replace it with an SFC id.
            assert.equal(resolved, kind === "package" ? upstream : null, label);
          } else {
            // A build deliberately keeps the snapshot it already compiled.
            assert.equal(calls.exists.get(file) ?? 0, 0, label);
            assert.equal(resolved, toPluginVisibleVirtualId(file, ssr), label);
          }
        }
      }
    }
  }
});

/** A host tests an id filter against the id with forward slashes (Vite), or as given (Rolldown). */
function hostCalls(filter: readonly RegExp[], id: string): boolean {
  const vite = filter.some((pattern) => pattern.test(id.replaceAll("\\", "/")));
  assert.equal(
    filter.some((pattern) => pattern.test(id)),
    vite,
    JSON.stringify(id),
  );
  return vite;
}

const STORY_MODULE = /\.(?:stories|story)\.[jt]sx$/;

/** The handlers' accept conditions as they were written before the filters existed. */
const accepts = {
  load: (id: string) =>
    id === RESOLVED_CSS_MODULE ||
    id.startsWith("\0") ||
    id.includes(".vue") ||
    id.includes("vize-file="),
  transform: (id: string) =>
    id.startsWith("\0") ||
    id.includes(".vue") ||
    id.includes(".tsrx") ||
    (/\.[jt]sx$/.test(id) && !STORY_MODULE.test(id)),
};

function idCorpus(): string[] {
  const prefixes = ["", "\0", "\0vize:", "/@fs", "/@id/__x00__", "C:\\app", "\0plugin-vue:"];
  const paths = [
    "/src/App.vue",
    "/src/App.vue.ts",
    "/src/App.ce.vue",
    "/src/main.ts",
    "/src/App.tsx",
    "/src/App.jsx",
    "/src/App.stories.tsx",
    "/src/App.story.jsx",
    "/src/App.stories.vue",
    "/src/widget.tsrx",
    "/src/theme.css",
    "/src/Page.md",
    "/src/tsx",
    "/src/vue.ts",
    "\\src\\App.vue",
    "\\src\\App.tsx",
    "/node_modules/vue/dist/vue.runtime.esm-bundler.js",
    "/node_modules/vue/server-renderer/index.mjs",
  ];
  const queries = [
    "",
    "?vue&vize",
    "?vue&vize-ssr",
    "?vue&type=style&index=0&lang=css",
    "?vue&type=style&index=0&lang.tsx",
    "?vue&type=i18n&index=0&lang.json",
    "?macro=true",
    "?definePage",
    "?nuxt_component=async",
    "?raw",
    "?v=1a2b3c",
    "?vue&type=style&vize-file=%2Fsrc%2FApp%2Evue",
    "?commonjs-proxy",
  ];
  const bare = ["vue", "vue/server-renderer", "virtual:vize-styles", "#imports", "./App.vue"];
  return [
    RESOLVED_CSS_MODULE,
    ...bare,
    ...prefixes.flatMap((prefix) =>
      paths.flatMap((file) => queries.map((query) => `${prefix}${file}${query}`)),
    ),
  ];
}

void test("the main hooks declare the ids their handlers accept", async (t) => {
  const { root } = project(t, 1);
  const plugin = vize({ configMode: false }).find((value) => value.name === "vite-plugin-vize");
  assert.ok(plugin);
  assert.equal((plugin.load as { filter?: { id?: unknown } }).filter?.id, LOAD_ID_FILTER);
  assert.equal((plugin.transform as { filter?: { id?: unknown } }).filter?.id, TRANSFORM_ID_FILTER);
  assert.equal((plugin.resolveId as { filter?: unknown }).filter, RESOLVE_ID_FILTER);

  const state = makeState(root);
  const corpus = idCorpus();
  let sfcModules = 0;
  for (const id of corpus) {
    const label = JSON.stringify(id);
    // `load` is where an SFC main module joins its environment's compile queue.
    if (getSfcModuleRequest(state, id)) {
      sfcModules++;
      assert.equal(hostCalls(LOAD_ID_FILTER, id), true, `SFC module ${label}`);
    }
    assert.equal(hostCalls(LOAD_ID_FILTER, id), accepts.load(id), `load ${label}`);
    // The one id family the transform filter lets through and its handler turns away.
    assert.equal(
      hostCalls(TRANSFORM_ID_FILTER, id),
      accepts.transform(id) || STORY_MODULE.test(id),
      `transform ${label}`,
    );
    if (!accepts.load(id)) assert.equal(loadHook(state, id), null, `load ${label}`);
    if (!accepts.transform(id)) {
      assert.equal(await transformHook(state, "", id), null, `transform ${label}`);
    }
  }
  assert.deepEqual(
    [true, false].map((accepted) => ({
      load: corpus.filter((id) => accepts.load(id) === accepted).length,
      transform: corpus.filter((id) => accepts.transform(id) === accepted).length,
    })),
    [
      { load: 1016, transform: 1076 },
      { load: 628, transform: 568 },
    ],
  );
  assert.equal(typeof loadHook(state, RESOLVED_CSS_MODULE), "string");
  assert.equal(sfcModules > 0, true, "the corpus holds SFC main modules");
});

const APP = `<script setup lang="ts">
defineProps<{ label: string }>();
</script>

<template><p class="plain scoped theme">{{ label }}</p></template>

<style>.plain { color: red; }</style>
<style scoped>.scoped { color: blue; }</style>
<style src="./theme.css"></style>
`;

type HookObject = { filter?: unknown; handler: (...args: unknown[]) => unknown };

/** A real build, with the host applying the hook filters or calling the bare handlers. */
async function bundle(root: string, hostFilters: boolean) {
  const plugins = vize({ configMode: false });
  const main = plugins.find((value) => value.name === "vite-plugin-vize") as unknown as Record<
    "load" | "transform",
    HookObject | HookObject["handler"]
  >;
  const answered: Record<"load" | "transform", string[]> = { load: [], transform: [] };
  for (const [name, idAt] of [
    ["load", 0],
    ["transform", 1],
  ] as const) {
    const declared = main[name] as HookObject;
    const observed = async function (this: unknown, ...args: unknown[]) {
      const result = await declared.handler.apply(this, args);
      if (result !== null && result !== undefined) answered[name].push(String(args[idAt]));
      return result;
    };
    main[name] = hostFilters ? { ...declared, handler: observed } : observed;
  }
  const result = (await build({
    root,
    configFile: false,
    logLevel: "silent",
    plugins,
    build: {
      write: false,
      minify: false,
      rollupOptions: { input: path.join(root, "entry.js"), external: [/^vue(?:\/|$)/] },
    },
  })) as Rollup.RollupOutput | Rollup.RollupOutput[];
  return {
    load: answered.load.sort(order),
    transform: answered.transform.sort(order),
    output: (Array.isArray(result) ? result : [result])
      .flatMap((item) => item.output)
      .map((item) => [item.fileName, item.type === "chunk" ? item.code : String(item.source)])
      .sort(([left], [right]) => order(left, right)),
  };
}

void test("a real build answers the same ids whether or not the host applies the filters", async (t) => {
  const { root } = project(t, 0);
  fs.writeFileSync(path.join(root, "src/App.vue"), APP);
  fs.writeFileSync(path.join(root, "src/theme.css"), ".theme { color: green; }\n");
  fs.copyFileSync(
    path.resolve(import.meta.dirname, "../../../../../tests/_fixtures/tsrx/mount/counter.tsrx"),
    path.join(root, "src/Counter.tsrx"),
  );
  fs.writeFileSync(
    path.join(root, "entry.js"),
    [
      'import "virtual:vize-styles";',
      'export { default } from "./src/App.vue";',
      'export { Counter } from "./src/Counter.tsrx";',
      "",
    ].join("\n"),
  );

  const filtered = await bundle(root, true);
  assert.deepEqual(filtered, await bundle(root, false));
  // Every id family reached its handler through the host's own matcher.
  const count = (ids: string[]) => ({
    virtual: ids.filter((id) => id.startsWith("\0")).length,
    style: ids.filter((id) => id.includes("vize-file=")).length,
    tsrx: ids.filter((id) => id.endsWith(".tsrx")).length,
    sfc: ids.filter((id) => /\.vue(?:\?vue&vize)?$/.test(id)).length,
  });
  assert.deepEqual(count(filtered.load), { virtual: 1, style: 3, tsrx: 0, sfc: 1 });
  assert.deepEqual(count(filtered.transform), { virtual: 0, style: 0, tsrx: 1, sfc: 0 });
});

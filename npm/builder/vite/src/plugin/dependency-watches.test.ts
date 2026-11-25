import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  buildWatchScope,
  createDependencyWatch,
  resetBuildWatches,
  UNWATCHED_DEPENDENCY,
} from "./dependency-watches.ts";
import { CompiledModuleCache, ownersOfDependency } from "./compiled-module-cache.ts";

void test("dev compiler inputs are watched without entering the module import graph", () => {
  const watched: string[] = [];
  const imports: string[] = [];
  const inputs = ["/project/src/tsconfig.json", "/project/src/Props.ts", "/project/theme.scss"];
  const watch = createDependencyWatch(
    {
      watcher: {
        add(file) {
          watched.push(file);
        },
      },
    },
    (file) => imports.push(file),
  );
  for (const input of inputs) watch!(input);
  assert.deepEqual(watched, inputs);
  assert.deepEqual(imports, []);
});

void test("absent configuration watches retain their HMR owners when the file appears", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-watch-"));
  try {
    const config = path.join(root, "tsconfig.json");
    const owner = path.join(root, "Card.vue");
    const declaration = path.join(root, "Props.ts");
    fs.writeFileSync(declaration, "export interface Props { label: string }");
    const dependencies = [config, declaration];
    const cache = new CompiledModuleCache();
    cache.set(owner, {
      code: "export default {}",
      scopeId: "watch",
      hasScoped: false,
      styles: [],
      dependencies,
    });
    const watched: string[] = [];
    const watch = createDependencyWatch(
      {
        watcher: {
          add(file) {
            watched.push(file);
          },
        },
      },
      () => assert.fail("watch-only config must never become a module import"),
    );
    assert.equal(fs.existsSync(config), false);
    for (const dependency of dependencies) watch!(dependency);
    assert.deepEqual(ownersOfDependency(cache, config), [owner]);
    fs.writeFileSync(config, "{}");
    assert.deepEqual(ownersOfDependency(cache, config), [owner]);
    cache.evict(owner);
    assert.deepEqual(ownersOfDependency(cache, config), [owner]);
    fs.writeFileSync(declaration, "export interface Props { label: number }");
    assert.deepEqual(ownersOfDependency(cache, declaration), [owner]);
    cache.set(owner, {
      code: "export default {}",
      scopeId: "watch",
      hasScoped: false,
      styles: [],
      dependencies,
    });
    for (const dependency of dependencies) {
      assert.deepEqual(ownersOfDependency(cache, dependency), [owner]);
    }
    assert.deepEqual(watched, [config, declaration]);
    assert.deepEqual(dependencies, [config, declaration]);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("build compilation retains its host watch registration", () => {
  const watched: string[] = [];
  const addWatchFile = (file: string) => {
    watched.push(file);
  };
  const watch = createDependencyWatch(null, addWatchFile);
  watch!("/project/src/tsconfig.json");
  assert.deepEqual(watched, ["/project/src/tsconfig.json"]);
  assert.equal(createDependencyWatch(null, undefined), undefined);
});

void test("Windows verbatim dependencies are watched once in their plain spelling", () => {
  for (const server of [true, false]) {
    const watched: string[] = [];
    const record = (file: string) => {
      watched.push(file);
    };
    const watch = createDependencyWatch(server ? { watcher: { add: record } } : null, record);
    for (const dependency of [
      "C:\\app\\tsconfig.json",
      "\\\\?\\C:\\app\\src\\tsconfig.json",
      "\\\\?\\C:\\app\\src\\types.ts",
      "\\\\?\\C:\\app\\tsconfig.json",
      "\\\\?\\UNC\\host\\share\\types.ts",
      "\\\\host\\share\\types.ts",
    ])
      watch!(dependency);
    assert.deepEqual(watched, [
      "C:\\app\\tsconfig.json",
      "C:\\app\\src\\tsconfig.json",
      "C:\\app\\src\\types.ts",
      "\\\\host\\share\\types.ts",
    ]);
  }
});

void test("shared inputs are watched once per server across client and SSR transforms", () => {
  const watched: string[] = [];
  const server = { watcher: { add: (file: string) => watched.push(file) } };
  const inputs = ["/project/types.ts", "/project/models/index.ts", "/project/tsconfig.json"];
  for (let module = 0; module < 1000; module += 1) {
    for (const environment of ["client", "ssr"]) {
      const watch = createDependencyWatch(server, () => assert.fail(environment));
      for (const input of inputs) watch!(input);
    }
  }
  assert.deepEqual(watched, inputs);
  const restarted = { watcher: { add: (file: string) => watched.push(file) } };
  for (const input of inputs) createDependencyWatch(restarted, undefined)!(input);
  assert.deepEqual(watched, [...inputs, ...inputs]);
});

void test("separate build modules each retain their own watch dependencies", () => {
  const watched: string[][] = [[], []];
  for (const module of watched) {
    const watch = createDependencyWatch(null, (file) => module.push(file));
    watch!("/project/types.ts");
    watch!("/project/types.ts");
  }
  assert.deepEqual(watched, [["/project/types.ts"], ["/project/types.ts"]]);
});

const barrel = Array.from({ length: 800 }, (_, index) => `/project/src/models/Model${index}.ts`);
const rolldownMeta = { rolldownVersion: "1.3.3" };

/** Each module's transform registers every type input of a component typed through `barrel`. */
function transformModules(scope: ReturnType<typeof buildWatchScope>, modules: number): number {
  let calls = 0;
  for (let module = 0; module < modules; module += 1) {
    const watch = createDependencyWatch(null, () => (calls += 1), scope);
    if (watch === UNWATCHED_DEPENDENCY) continue;
    for (const input of barrel) watch!(input);
  }
  return calls;
}

void test("a build that does not watch registers no compiler input", () => {
  const environment = { config: { build: {} } };
  const scope = buildWatchScope({ meta: { ...rolldownMeta, watchMode: false }, environment });
  assert.equal(scope, false);
  assert.equal(createDependencyWatch(null, undefined, scope), UNWATCHED_DEPENDENCY);
  assert.equal(transformModules(scope, 1000), 0);
});

void test("a full Rolldown rebuild registers each input once per environment build", () => {
  const client = { config: { build: {} } };
  const ssr = { config: { build: {} } };
  const meta = { ...rolldownMeta, watchMode: true };
  assert.equal(buildWatchScope({ meta, environment: client }), client);
  assert.equal(transformModules(client, 1000), barrel.length);
  assert.equal(transformModules(ssr, 1000), barrel.length, "each environment has its own watcher");
  assert.equal(transformModules(client, 1000), 0, "the same build registered them already");
  resetBuildWatches(client);
  assert.equal(transformModules(client, 1000), barrel.length, "a rebuild collects them again");
});

void test("rebuilds that re-transform only changed modules keep per-module inputs", () => {
  const incremental = {
    config: { build: { rolldownOptions: { experimental: { incrementalBuild: true } } } },
  };
  const watching = { watchMode: true };
  for (const context of [
    { meta: { ...rolldownMeta, ...watching }, environment: incremental },
    // Rollup's watch cache reuses a module unless one of its own inputs changed.
    { meta: watching, environment: { config: { build: {} } } },
    // A standalone Rolldown build has no environment to scope a registration to.
    { meta: { ...rolldownMeta, ...watching } },
  ]) {
    const scope = buildWatchScope(context);
    assert.equal(scope, "module");
    assert.equal(transformModules(scope, 3), barrel.length * 3);
  }
});

void test("the dev server watches only inputs outside the root its watcher covers", () => {
  const watched: string[] = [];
  const server = {
    config: { root: "/project" },
    watcher: { add: (file: string) => watched.push(file) },
  };
  for (let module = 0; module < 3; module += 1) {
    const watch = createDependencyWatch(server, () => assert.fail("not a module import"));
    for (const input of ["/project/src/types.ts", "/project-ui/types.ts", "/shared/tsconfig.json"])
      watch!(input);
  }
  assert.deepEqual(watched, ["/project-ui/types.ts", "/shared/tsconfig.json"]);

  const windows: string[] = [];
  const windowsServer = {
    config: { root: "C:/app" },
    watcher: { add: (f: string) => windows.push(f) },
  };
  const watch = createDependencyWatch(windowsServer, undefined)!;
  for (const input of ["\\\\?\\C:\\app\\src\\types.ts", "C:\\shared\\types.ts"]) watch(input);
  assert.deepEqual(windows, ["C:\\shared\\types.ts"]);
});

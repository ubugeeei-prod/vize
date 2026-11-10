import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { createDependencyWatch } from "./dependency-watches.ts";
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

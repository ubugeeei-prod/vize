import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";

import { createNuxtComponentBridge } from "./component-hmr.ts";
import { createNuxtComponentResolver, injectNuxtComponentImports } from "./components.ts";

const TEST_TMP_ROOT = path.resolve(process.cwd(), "target", "vize-tests", "nuxt-component-hmr");
const missingProject = {
  buildDir: "/nonexistent-vize-test/.nuxt",
  rootDir: "/nonexistent-vize-test",
};
const raceA = { pascalName: "RaceA", filePath: "/app/components/RaceA.vue" };
const raceB = { pascalName: "RaceB", filePath: "/app/components/RaceB.vue" };
const page = `const _component_RaceA = _resolveComponent("RaceA");\nexport default {};\n`;

function environment(ids: string[]) {
  const modules = new Map(ids.map((id) => [id, { id }]));
  return { moduleGraph: { getModuleById: (id: string) => modules.get(id) } };
}

void test("a rescan replaces the registered components", () => {
  const resolver = createNuxtComponentResolver(missingProject);
  resolver.register([raceA]);
  assert.equal(resolver.resolve("RaceA", "client")?.filePath, raceA.filePath);
  resolver.register([raceB]);
  assert.equal(resolver.resolve("RaceA", "client"), null, "a renamed component is gone");
  assert.equal(resolver.resolve("RaceB", "client")?.filePath, raceB.filePath);
});

void test("the generated d.ts does not bring back a component a rescan removed", () => {
  fs.mkdirSync(TEST_TMP_ROOT, { recursive: true });
  const rootDir = fs.realpathSync(fs.mkdtempSync(path.join(TEST_TMP_ROOT, "fixture-")));
  const buildDir = path.join(rootDir, ".nuxt");
  fs.mkdirSync(buildDir, { recursive: true });
  const removed = path.join(rootDir, "app/components/RaceA.vue");
  const external = path.join(rootDir, "node_modules/ui/Card.vue");
  fs.writeFileSync(
    path.join(buildDir, "components.d.ts"),
    [
      `export const RaceA: typeof import(${JSON.stringify(path.relative(buildDir, removed))})['default']`,
      `export const UiCard: typeof import(${JSON.stringify(path.relative(buildDir, external))})['default']`,
    ].join("\n"),
  );
  const resolver = createNuxtComponentResolver({ buildDir, rootDir });
  resolver.register([{ pascalName: "RaceA", filePath: removed }]);
  resolver.register([]);

  assert.equal(resolver.resolve("RaceA", "client"), null);
  assert.equal(
    resolver.resolve("UiCard", "client")?.filePath,
    external,
    "a d.ts component Nuxt never scanned still resolves",
  );
});

void test("a new component reloads the compiled module that could not resolve it", () => {
  const bridge = createNuxtComponentBridge({ ...missingProject, dev: true, modules: [] });
  const client = environment(["/app/pages/index.vue?vue&vize"]);
  const ssr = environment(["/app/pages/index.vue?vue&vize"]);
  const hotUpdate = (env: object, type: "create" | "update" | "delete", modules: unknown[] = []) =>
    bridge.hotUpdate.handler.call({ environment: env as never }, { type, modules });

  bridge.register([]);
  const transform = (env: object, mode: "client" | "server") =>
    injectNuxtComponentImports(
      page,
      bridge.resolverFor(env, "/app/pages/index.vue?vue&vize", mode),
    );
  assert.equal(transform(client, "client"), page, "the page compiled before RaceA existed");
  transform(ssr, "server");

  bridge.register([raceA]);
  assert.equal(hotUpdate(client, "update"), undefined, "an edit does not rescan components");
  assert.deepEqual(hotUpdate(client, "create"), [{ id: "/app/pages/index.vue?vue&vize" }]);
  assert.deepEqual(hotUpdate(ssr, "create"), [{ id: "/app/pages/index.vue?vue&vize" }]);

  assert.match(transform(client, "client"), /import __vize_nuxt_component_0 from/);
  assert.equal(hotUpdate(client, "create"), undefined, "a resolved page is left alone");

  bridge.register([raceB]);
  const existing = { id: "/app/components/RaceA.vue" };
  assert.deepEqual(
    hotUpdate(client, "delete", [existing]),
    [existing, { id: "/app/pages/index.vue?vue&vize" }],
    "a page that imported a deleted component reloads without it",
  );
});

void test("builds record no component resolutions", () => {
  const bridge = createNuxtComponentBridge({ ...missingProject, dev: false, modules: [] });
  const client = environment(["/app/pages/index.vue?vue&vize"]);
  injectNuxtComponentImports(
    page,
    bridge.resolverFor(client, "/app/pages/index.vue?vue&vize", "client"),
  );
  bridge.register([raceA]);
  assert.equal(
    bridge.hotUpdate.handler.call({ environment: client }, { type: "create", modules: [] }),
    undefined,
  );
});

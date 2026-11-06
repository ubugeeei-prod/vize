import assert from "node:assert/strict";
import test from "node:test";

import { createNuxtComponentResolver, injectNuxtComponentImports } from "./components.ts";

void test("standalone server components retain Nuxt's island wrapper", () => {
  const resolver = createNuxtComponentResolver({
    buildDir: "/nonexistent-vize-test/.nuxt",
    rootDir: "/nonexistent-vize-test",
  });
  resolver.register([
    {
      pascalName: "IslandProbe",
      filePath: "/virtual/IslandProbe.server.vue",
      mode: "server",
    },
  ]);

  const resolved = resolver.resolve("IslandProbe", "server");
  assert.deepEqual(resolved, {
    exportName: "default",
    filePath:
      "/virtual/IslandProbe.server.vue?nuxt_component=server&nuxt_component_name=IslandProbe&nuxt_component_export=default",
    mode: "server",
  });
  assert.match(
    injectNuxtComponentImports('const island = resolveComponent("IslandProbe");', (name) =>
      resolver.resolve(name, "server"),
    ),
    /import __vize_nuxt_component_0 from "\/virtual\/IslandProbe\.server\.vue\?nuxt_component=server&nuxt_component_name=IslandProbe&nuxt_component_export=default";/,
  );
});

void test("raw server registrations stay direct imports", () => {
  const resolver = createNuxtComponentResolver({
    buildDir: "/nonexistent-vize-test/.nuxt",
    rootDir: "/nonexistent-vize-test",
  });
  resolver.register([
    {
      _raw: true,
      pascalName: "RawServer",
      filePath: "/virtual/RawServer.server.vue",
      mode: "server",
    },
  ]);

  assert.deepEqual(resolver.resolve("RawServer", "server"), {
    exportName: "default",
    filePath: "/virtual/RawServer.server.vue",
    mode: "server",
  });
});

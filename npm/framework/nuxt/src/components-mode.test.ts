import assert from "node:assert/strict";
import test from "node:test";

import { createNuxtComponentResolver } from "./components.ts";

void test("registered components resolve by build mode instead of registration order", () => {
  const resolver = createNuxtComponentResolver({
    buildDir: "/nonexistent-vize-test/.nuxt",
    rootDir: "/nonexistent-vize-test",
  });

  resolver.register([
    {
      pascalName: "NuxtClientFallback",
      filePath: "/virtual/client-fallback.client.vue",
      mode: "client",
    },
    {
      pascalName: "NuxtClientFallback",
      filePath: "/virtual/client-fallback.server.vue",
      mode: "server",
    },
  ]);

  assert.deepEqual(resolver.resolve("NuxtClientFallback", "server"), {
    exportName: "default",
    filePath: "/virtual/client-fallback.server.vue",
    mode: "server",
  });
  assert.deepEqual(resolver.resolve("nuxt-client-fallback", "client"), {
    exportName: "default",
    filePath: "/virtual/client-fallback.client.vue",
    mode: "client",
  });
  assert.deepEqual(resolver.resolve("LazyNuxtClientFallback", "server"), {
    exportName: "default",
    filePath: "/virtual/client-fallback.server.vue",
    lazy: true,
    mode: "server",
  });
});

void test("an opposite-mode registration stays with Nuxt's host resolver", () => {
  const resolver = createNuxtComponentResolver({
    buildDir: "/nonexistent-vize-test/.nuxt",
    rootDir: "/nonexistent-vize-test",
  });
  resolver.register([
    { pascalName: "ClientWidget", filePath: "/virtual/ClientWidget.client.vue", mode: "client" },
  ]);

  assert.equal(resolver.resolve("ClientWidget", "server"), null);
  assert.deepEqual(resolver.resolve("ClientWidget", "client"), {
    exportName: "default",
    filePath: "/virtual/ClientWidget.client.vue",
    mode: "client",
  });
});

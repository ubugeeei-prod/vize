import assert from "node:assert/strict";
import test from "node:test";

import {
  patchNuxtClientManifestCloseBundlePlugin,
  type NuxtClientManifestVitePlugin,
} from "./client-manifest-bridge.ts";

function invokeCloseBundle(
  plugin: NuxtClientManifestVitePlugin,
  context: object = plugin,
): Promise<unknown> {
  const hook = plugin.closeBundle;
  if (typeof hook === "function") {
    return Promise.resolve(hook.call(context));
  }
  if (hook && typeof hook.handler === "function") {
    return Promise.resolve(hook.handler.call(context));
  }
  throw new Error("plugin has no closeBundle hook");
}

const inEnvironment = (name: string) => ({ environment: { name } });

void test("Nuxt client manifest closeBundle bridge runs the SSR finalizer after the client call", async () => {
  // Mirrors Nuxt's Environment API plugin: one instance, applied to client and
  // SSR, where only the SSR closeBundle reads and removes the client manifest.
  const calls: (string | undefined)[] = [];
  const plugin: NuxtClientManifestVitePlugin = {
    name: "nuxt:client-manifest",
    async closeBundle(this: unknown) {
      const name = (this as { environment?: { name?: string } }).environment?.name;
      calls.push(name);
      return name === "ssr" ? "finalized" : undefined;
    },
  };

  patchNuxtClientManifestCloseBundlePlugin(plugin, {});

  assert.equal(await invokeCloseBundle(plugin, inEnvironment("client")), undefined);
  assert.equal(await invokeCloseBundle(plugin, inEnvironment("ssr")), "finalized");
  assert.equal(await invokeCloseBundle(plugin, inEnvironment("ssr")), "finalized");
  assert.equal(await invokeCloseBundle(plugin, inEnvironment("worker")), undefined);
  assert.deepEqual(calls, ["client", "ssr", "worker"]);
});

void test("Nuxt client manifest closeBundle bridge dedupes each environment across instances", async () => {
  const calls: string[] = [];
  const scope = {};
  const plugins = [0, 1].map((index): NuxtClientManifestVitePlugin => ({
    name: "nuxt:client-manifest",
    closeBundle: {
      handler(this: unknown) {
        calls.push(`${index}:${(this as { environment: { name: string } }).environment.name}`);
      },
    },
  }));
  for (const plugin of plugins) patchNuxtClientManifestCloseBundlePlugin(plugin, scope);

  for (const plugin of plugins) {
    await invokeCloseBundle(plugin, inEnvironment("client"));
    await invokeCloseBundle(plugin, inEnvironment("ssr"));
  }

  assert.deepEqual(calls, ["0:client", "0:ssr"]);
});

void test("Nuxt client manifest closeBundle bridge is idempotent per build scope", async () => {
  let calls = 0;
  const scope = {};
  const firstPlugin: NuxtClientManifestVitePlugin = {
    name: "nuxt:client-manifest",
    closeBundle() {
      calls++;
    },
  };
  const duplicatePlugin: NuxtClientManifestVitePlugin = {
    name: "nuxt:client-manifest",
    closeBundle() {
      calls++;
      throw new Error("duplicate manifest closeBundle should not run");
    },
  };

  patchNuxtClientManifestCloseBundlePlugin(firstPlugin, scope);
  patchNuxtClientManifestCloseBundlePlugin(duplicatePlugin, scope);

  await invokeCloseBundle(firstPlugin);
  await invokeCloseBundle(duplicatePlugin);

  assert.equal(calls, 1);
});

void test("Nuxt client manifest closeBundle bridge keeps failed teardown retryable", async () => {
  let calls = 0;
  const scope = {};
  const plugin: NuxtClientManifestVitePlugin = {
    name: "nuxt:client-manifest",
    closeBundle() {
      calls++;
      if (calls === 1) {
        return Promise.reject(new Error("teardown failed"));
      }
      return "retried";
    },
  };

  patchNuxtClientManifestCloseBundlePlugin(plugin, scope);

  await assert.rejects(invokeCloseBundle(plugin), /teardown failed/);
  assert.equal(await invokeCloseBundle(plugin), "retried");
  assert.equal(calls, 2);
  assert.equal(await invokeCloseBundle(plugin), "retried");
  assert.equal(calls, 2);
});

void test("Nuxt client manifest closeBundle bridge supports hook objects", async () => {
  let calls = 0;
  const plugin: NuxtClientManifestVitePlugin = {
    name: "nuxt:client-manifest",
    closeBundle: {
      handler() {
        calls++;
        return "manifest";
      },
    },
  };

  patchNuxtClientManifestCloseBundlePlugin(plugin, {});

  assert.equal(await invokeCloseBundle(plugin), "manifest");
  assert.equal(await invokeCloseBundle(plugin), "manifest");
  assert.equal(calls, 1);
});

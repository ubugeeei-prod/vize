import assert from "node:assert/strict";
import test from "node:test";

import {
  patchNuxtClientManifestCloseBundlePlugin,
  type NuxtClientManifestVitePlugin,
} from "./client-manifest-bridge.ts";

function invokeCloseBundle(plugin: NuxtClientManifestVitePlugin): Promise<unknown> {
  const hook = plugin.closeBundle;
  if (typeof hook === "function") {
    return Promise.resolve(hook.call(plugin));
  }
  if (hook && typeof hook.handler === "function") {
    return Promise.resolve(hook.handler.call(plugin));
  }
  throw new Error("plugin has no closeBundle hook");
}

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

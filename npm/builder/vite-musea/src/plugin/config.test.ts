import assert from "node:assert/strict";
import test from "node:test";

import type { ResolvedConfig } from "vite";
import {
  registerResolvedVizeConfig,
  unregisterResolvedVizeConfig,
} from "@vizejs/vite-plugin/internal/config-bridge";

import { resolveMuseaSharedConfig } from "./config.js";

function resolvedConfig(root: string): ResolvedConfig {
  return {
    root,
    mode: "production",
    command: "build",
    base: "/",
    isProduction: true,
    build: { assetsDir: "assets", ssr: false },
    define: {},
    plugins: [],
    resolve: { alias: [] },
  } as unknown as ResolvedConfig;
}

void test("Musea resolves only the exact Vite config registration", async () => {
  const previousConfigFile = process.env.VIZE_CONFIG_FILE;
  const root = "/workspace/shared-root";
  const client = resolvedConfig(root);
  const server = resolvedConfig(root);
  const restarted = resolvedConfig(root);
  const pending = resolvedConfig(root);
  const reverseOrdered = resolvedConfig(root);
  const clientConfig = { musea: { basePath: "/client" } };
  const fallbackConfig = { musea: { basePath: "/fallback" } };
  let fallbackCalls = 0;
  const fallbackLoader = async () => {
    fallbackCalls += 1;
    return fallbackConfig;
  };

  process.env.VIZE_CONFIG_FILE = "/workspace/explicit-vize.config.ts";
  registerResolvedVizeConfig(client, clientConfig);
  registerResolvedVizeConfig(server, null);
  try {
    assert.equal(await resolveMuseaSharedConfig(client, fallbackLoader), clientConfig);
    assert.equal(
      await resolveMuseaSharedConfig(server, fallbackLoader),
      null,
      "registered null must win over the explicit config-file fallback",
    );
    assert.equal(fallbackCalls, 0);

    assert.equal(
      await resolveMuseaSharedConfig(restarted, fallbackLoader),
      fallbackConfig,
      "an unregistered restart may use the explicit config-file fallback",
    );
    assert.equal(fallbackCalls, 1);

    registerResolvedVizeConfig(restarted, null);
    assert.equal(
      await resolveMuseaSharedConfig(restarted, fallbackLoader),
      null,
      "config removal on restart must overwrite fallback behavior",
    );
    assert.equal(fallbackCalls, 1);

    const pendingConfig = Promise.withResolvers<typeof clientConfig | null>();
    registerResolvedVizeConfig(pending, pendingConfig.promise);
    const pendingResolution = resolveMuseaSharedConfig(pending, fallbackLoader);
    assert.equal(fallbackCalls, 1, "a pending exact registration must not use the fallback");
    pendingConfig.resolve(clientConfig);
    assert.equal(await pendingResolution, clientConfig);
    assert.equal(fallbackCalls, 1);

    const reverseOrderedResolution = resolveMuseaSharedConfig(reverseOrdered, fallbackLoader);
    registerResolvedVizeConfig(reverseOrdered, clientConfig);
    assert.equal(
      await reverseOrderedResolution,
      clientConfig,
      "Musea should observe Vize when its configResolved hook starts first",
    );
    assert.equal(fallbackCalls, 1);
  } finally {
    unregisterResolvedVizeConfig(client);
    unregisterResolvedVizeConfig(server);
    unregisterResolvedVizeConfig(restarted);
    unregisterResolvedVizeConfig(pending);
    unregisterResolvedVizeConfig(reverseOrdered);
    if (previousConfigFile === undefined) {
      Reflect.deleteProperty(process.env, "VIZE_CONFIG_FILE");
    } else {
      process.env.VIZE_CONFIG_FILE = previousConfigFile;
    }
  }
});

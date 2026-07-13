import assert from "node:assert/strict";
import { EventEmitter } from "node:events";

import type { Plugin, ResolvedConfig } from "vite";

import { vizeConfigStore } from "../config.ts";
import { getResolvedVizeConfigRegistration } from "../internal/config-bridge.ts";
import type { ResolvedVizeConfig } from "../types.ts";
import * as configBridge from "./config-lifecycle.ts";
import { vize } from "./index.ts";

function createResolvedConfig(ssr: boolean): ResolvedConfig {
  return {
    root: "/workspace/shared-root",
    base: "/",
    mode: "production",
    command: "build",
    isProduction: true,
    build: { assetsDir: "assets", ssr },
    define: {},
    plugins: [],
    resolve: { alias: [] },
  } as unknown as ResolvedConfig;
}

function functionHook<T extends Function>(hook: T | { handler: T } | undefined, name: string): T {
  const handler = typeof hook === "function" ? hook : hook?.handler;
  assert.equal(typeof handler, "function", `${name} hook should exist`);
  return handler as T;
}

async function assertRegisteredConfig(
  resolvedConfig: ResolvedConfig,
  expectedConfig: ResolvedVizeConfig | null,
  message?: string,
): Promise<void> {
  const registration = getResolvedVizeConfigRegistration(resolvedConfig);
  assert.equal(registration.registered, true, message);
  if (registration.registered) {
    assert.equal(await registration.config, expectedConfig, message);
  }
}

const plugins = vize({ configMode: false });
const plugin = plugins.find((candidate) => candidate.name === "vite-plugin-vize") as Plugin;
const client = createResolvedConfig(false);
const server = createResolvedConfig(true);
vizeConfigStore.set(client.root, { compiler: { sourceMap: true } });
const configResolved = functionHook(plugin.configResolved, "configResolved") as (
  config: ResolvedConfig,
) => Promise<void>;

await Promise.all([configResolved(client), configResolved(server)]);
assert.equal(
  vizeConfigStore.has(client.root),
  false,
  "explicit null should clear stale legacy state",
);
await assertRegisteredConfig(client, null);
await assertRegisteredConfig(server, null);

const closeBundle = functionHook(plugin.closeBundle, "closeBundle") as (this: unknown) => void;
closeBundle.call({ environment: { getTopLevelConfig: () => client } });
assert.deepEqual(getResolvedVizeConfigRegistration(client), { registered: false });
await assertRegisteredConfig(
  server,
  null,
  "build cleanup should delete only the current ResolvedConfig",
);

const httpServer = new EventEmitter();
const watcher = new EventEmitter();
const configureServer = functionHook(plugin.configureServer, "configureServer") as (
  server: unknown,
) => void;
configureServer({
  config: server,
  httpServer,
  watcher,
  middlewares: { use() {} },
});
httpServer.emit("close");
assert.deepEqual(
  getResolvedVizeConfigRegistration(server),
  { registered: false },
  "dev-server close should release its exact registration",
);
watcher.emit("close");

const compatibilityClient = createResolvedConfig(false);
const compatibilityServer = createResolvedConfig(true);
const staleNullConfig = createResolvedConfig(false);
const currentConfig = createResolvedConfig(true);
const clientConfig = { compiler: { sourceMap: true } } as ResolvedVizeConfig;
const serverConfig = { compiler: { sourceMap: false } } as ResolvedVizeConfig;
const pendingConfig = Promise.withResolvers<ResolvedVizeConfig | null>();
const pendingRegistration = configBridge.register(
  compatibilityClient,
  compatibilityClient.root,
  pendingConfig.promise,
);
const registration = getResolvedVizeConfigRegistration(compatibilityClient);
assert.equal(
  registration.registered,
  true,
  "the pending config must be visible before configResolved yields to parallel hooks",
);
pendingConfig.resolve(clientConfig);
await pendingRegistration;
await configBridge.register(
  compatibilityServer,
  compatibilityServer.root,
  Promise.resolve(serverConfig),
);
configBridge.unregisterBuild({
  environment: { getTopLevelConfig: () => compatibilityClient },
});
assert.equal(
  vizeConfigStore.get(compatibilityClient.root),
  serverConfig,
  "cleaning one legacy projection must preserve a newer same-root owner",
);
configBridge.unregisterBuild({
  environment: { getTopLevelConfig: () => compatibilityServer },
});
assert.equal(
  vizeConfigStore.has(compatibilityServer.root),
  false,
  "cleaning the current owner should release legacy compatibility state",
);

const pendingNull = Promise.withResolvers<ResolvedVizeConfig | null>();
const staleNullRegistration = configBridge.register(
  staleNullConfig,
  staleNullConfig.root,
  pendingNull.promise,
);
await configBridge.register(currentConfig, currentConfig.root, Promise.resolve(serverConfig));
pendingNull.resolve(null);
await staleNullRegistration;
assert.equal(
  vizeConfigStore.get(currentConfig.root),
  serverConfig,
  "an older null resolution must not clear the current same-root owner",
);
configBridge.unregisterBuild({ environment: { getTopLevelConfig: () => staleNullConfig } });
configBridge.unregisterBuild({ environment: { getTopLevelConfig: () => currentConfig } });

console.log("vite-plugin-vize bridge lifecycle tests passed!");

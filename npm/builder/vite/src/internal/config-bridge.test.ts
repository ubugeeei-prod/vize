import assert from "node:assert/strict";

import type { ResolvedConfig } from "vite";

import type { ResolvedVizeConfig } from "../types.ts";
import {
  getResolvedVizeConfigRegistration,
  registerResolvedVizeConfig,
  unregisterResolvedVizeConfig,
} from "./config-bridge.ts";

function resolvedConfig(root: string): ResolvedConfig {
  return { root } as ResolvedConfig;
}

const root = "/workspace/shared-root";
const client = resolvedConfig(root);
const server = resolvedConfig(root);
const exactClone = resolvedConfig(root);
const clientConfig = { compiler: { sourceMap: true } } as ResolvedVizeConfig;

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

registerResolvedVizeConfig(client, clientConfig);
registerResolvedVizeConfig(server, null);

await assertRegisteredConfig(
  client,
  clientConfig,
  "parallel client config should keep its exact registration",
);
await assertRegisteredConfig(
  server,
  null,
  "a parallel SSR config should preserve an explicit null registration",
);
assert.deepEqual(
  getResolvedVizeConfigRegistration(exactClone),
  { registered: false },
  "same-root objects must not inherit another Vite instance's registration",
);

assert.equal(unregisterResolvedVizeConfig(client), true);
assert.deepEqual(getResolvedVizeConfigRegistration(client), { registered: false });
await assertRegisteredConfig(
  server,
  null,
  "cleaning one build must not clear a concurrent same-root build",
);
assert.equal(unregisterResolvedVizeConfig(client), false, "cleanup should be idempotent");
assert.equal(unregisterResolvedVizeConfig(server), true);

console.log("vite-plugin-vize config bridge tests passed!");

import assert from "node:assert/strict";
import { once } from "node:events";
import { createServer } from "node:net";
import { test } from "node:test";
import { buildMisskeyDevConfig, resolveAvailablePort, runMisskeyBeforeStart } from "./dev-app.ts";

test("buildMisskeyDevConfig uses a valid id generation method", () => {
  const config = buildMisskeyDevConfig(
    3001,
    "/Users/ubugeeei/projects/personal/oss/ubugeeei/vize/__agent_only/vize-dev.pid",
  );

  assert.match(config, /^id: aidx$/m);
  assert.match(config, /^port: 3001$/m);
  assert.match(
    config,
    /^pidFile: \/Users\/ubugeeei\/projects\/personal\/oss\/ubugeeei\/vize\/__agent_only\/vize-dev\.pid$/m,
  );
});

test("runMisskeyBeforeStart builds backend before readiness check", () => {
  const calls: string[] = [];

  runMisskeyBeforeStart("/misskey", "vize-dev.yml", {
    startLocalServices(misskeyRoot) {
      calls.push(`start:${misskeyRoot}`);
    },
    ensureBackendBuilt(misskeyRoot, configName) {
      calls.push(`build:${misskeyRoot}:${configName}`);
    },
    ensureNativeDependencies(misskeyRoot) {
      calls.push(`native:${misskeyRoot}`);
    },
    waitForLocalServices(misskeyRoot, configName) {
      calls.push(`ready:${misskeyRoot}:${configName}`);
    },
    ensureMigrated(misskeyRoot, configName) {
      calls.push(`migrate:${misskeyRoot}:${configName}`);
    },
  });

  assert.deepEqual(calls, [
    "start:/misskey",
    "build:/misskey:vize-dev.yml",
    "native:/misskey",
    "ready:/misskey:vize-dev.yml",
    "migrate:/misskey:vize-dev.yml",
  ]);
});

test("resolveAvailablePort skips ports occupied by IPv6 wildcard listeners", async () => {
  const blocker = createServer();
  blocker.listen(0, "::");
  await once(blocker, "listening");

  const address = blocker.address();
  assert.ok(address != null && typeof address === "object");

  try {
    const nextPort = await resolveAvailablePort(address.port);
    assert.notEqual(nextPort, address.port);
  } finally {
    await new Promise<void>((resolve, reject) => {
      blocker.close((error) => {
        if (error) {
          reject(error);
          return;
        }
        resolve();
      });
    });
  }
});

test("resolveAvailablePort skips ports occupied by IPv4 listeners", async () => {
  const blocker = createServer();
  blocker.listen(0, "127.0.0.1");
  await once(blocker, "listening");

  const address = blocker.address();
  assert.ok(address != null && typeof address === "object");

  try {
    const nextPort = await resolveAvailablePort(address.port);
    assert.notEqual(nextPort, address.port);
  } finally {
    await new Promise<void>((resolve, reject) => {
      blocker.close((error) => {
        if (error) {
          reject(error);
          return;
        }
        resolve();
      });
    });
  }
});

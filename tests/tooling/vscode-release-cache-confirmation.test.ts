import assert from "node:assert/strict";
import fs from "node:fs";
import { stripTypeScriptTypes } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import vm from "node:vm";
import { createSync, deferred } from "./support/vscode-config-sync.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const discoverySource = stripTypeScriptTypes(
  fs.readFileSync(path.join(root, "editors/vscode/src/server-discovery.ts"), "utf8"),
  { mode: "transform" },
)
  .replace(/^import[\s\S]*?;\n/gm, "")
  .replace(/^export /gm, "");

interface CachedInspection {
  inspect(file: string, probe: boolean): Promise<Record<string, unknown> | undefined>;
}

await test("a cached release starts unprobed; only a refresh spawns its --version probe", async () => {
  const operations: string[][] = [];
  const context = vm.createContext({
    path,
    process: { env: {} },
    Date,
    promisify: () => () => {},
    execFile: () => {},
    fs: { existsSync: (file: string) => file === "/storage/vize" },
    operations,
  });
  const hooks = `
    inspectServerCandidate = async (candidate) => {
      operations.push(["probe", candidate.path]);
      return { ...candidate, version: "1.2.3" };
    };
    return { inspect: (file, probe) => inspectCachedReleaseServer(file, "1.2.3", probe) };
  `;
  const control: CachedInspection = new vm.Script(
    `${discoverySource.replace(/return\s*\{\s*findServerPath\s*\};/u, hooks)}\n
    createServerDiscovery(
      { configurationSyncGeneration: 1, outputChannel: { appendLine: () => {} } },
      () => {}, () => {},
    );
  `,
  ).runInContext(context);
  assert.deepEqual(
    { ...(await control.inspect("/storage/vize", false)) },
    { path: "/storage/vize", source: "release", version: "1.2.3", versionUnconfirmed: true },
  );
  assert.deepEqual(operations, []);
  assert.deepEqual(
    { ...(await control.inspect("/storage/vize", true)) },
    { path: "/storage/vize", source: "release", version: "1.2.3" },
  );
  assert.deepEqual(
    operations.map((row) => [...row]),
    [["probe", "/storage/vize"]],
  );
  assert.equal(await control.inspect("/storage/missing", false), undefined);
});

await test("a cached release is confirmed by its initialize result without a probe", async () => {
  const ready = deferred();
  ready.resolve();
  const current = createSync(async () => {}, undefined, false, ready, { reported: ["1.2.3"] });
  await current.control.syncClientToConfiguration({}, "initial activation");
  assert.deepEqual(
    current.operations.filter(([kind]) => kind === "find").map((row) => [...row]),
    [["find", false]],
  );
  assert.deepEqual(
    { ...current.control.getCandidate() },
    { path: "/test/vize", source: "release", version: "1.2.3" },
  );
  assert.equal(current.control.state().hasClient, true);
});

await test("a cached release reporting another version restarts through the probed refresh", async () => {
  const ready = deferred();
  ready.resolve();
  const current = createSync(async () => {}, undefined, false, ready, {
    reported: ["0.9.0", "1.2.3"],
  });
  await current.control.syncClientToConfiguration({}, "initial activation");
  assert.deepEqual(
    current.operations
      .filter(([kind]) => kind === "find" || kind.startsWith("real-"))
      .map((row) => [...row]),
    [["find", false], ["real-start"], ["real-stop"], ["find", true], ["real-start"]],
  );
  assert.ok(current.logs.some((line) => line.includes("reported 0.9.0, not 1.2.3")));
  assert.equal(current.control.getCandidate()?.versionUnconfirmed, undefined);
  assert.equal(current.control.state().hasClient, true);
});

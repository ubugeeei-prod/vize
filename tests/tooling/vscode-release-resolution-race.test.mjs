import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { stripTypeScriptTypes } from "node:module";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import vm from "node:vm";
import { runVmProgram, vmProgramSource } from "./support/vm-programs.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const source = fs.readFileSync(path.join(root, "editors/vscode/src/server-discovery.ts"), "utf8");
const body = stripTypeScriptTypes(source, { mode: "transform" })
  .replace(/^import[\s\S]*?;\n/gm, "")
  .replace(/^export /gm, "");

function deferred() {
  let resolve;
  const promise = new Promise((complete) => {
    resolve = complete;
  });
  return { promise, resolve };
}

function createResolution(pauseAt, cachedCandidate) {
  const operations = [];
  const entered = deferred();
  const resumed = deferred();
  const operation = async (stage, result) => {
    operations.push(stage);
    if (stage === pauseAt) {
      entered.resolve();
      await resumed.promise;
    }
    return result;
  };
  const candidate = {
    path: "/storage/servers/1.2.3/native/vize",
    version: "1.2.3",
    source: "release",
  };
  const context = vm.createContext({
    path,
    process: { env: {} },
    Date,
    promisify: () => () => {},
    execFile: () => {},
    WorkspaceLspConfigDiscovery: class {},
    WORKSPACE_LSP_CONFIG_FILES: [],
    fs: {
      rmSync: () => operations.push("remove"),
      promises: { mkdir: () => operation("mkdir") },
    },
    operations,
    operation,
    candidate,
    cachedCandidate,
    downloadFile: (url) => operation(url.endsWith(".sha256") ? "checksum" : "archive"),
    extractReleaseArchive: () => operation("extract"),
    pruneDownloadedServerVersions: () => operation("prune", { removed: [], failed: [] }),
  });
  const hooks = vmProgramSource("release-resolution-hooks.cjs");
  const marker = /return\s*\{\s*findServerPath\s*\};/gu;
  assert.equal([...body.matchAll(marker)].length, 1, "one actual factory dependency seam");
  new vm.Script(body.replace(marker, () => hooks)).runInContext(context);
  const control = runVmProgram("create-server-discovery.js", context);
  return { control, operations, entered, resumed, candidate };
}

await test("current release resolution retains the full download, checksum and extraction sequence", async () => {
  const current = createResolution();
  assert.deepEqual(await current.control.resolve(), current.candidate);
  assert.deepEqual(current.operations, [
    "cached",
    "starting",
    "remove",
    "mkdir",
    "archive",
    "checksum",
    "verify",
    "extract",
    "inspect",
    "prune",
  ]);
  const cached = createResolution(undefined, { path: "/cached/vize", version: "1.2.3" });
  assert.deepEqual(await cached.control.resolve(), { path: "/cached/vize", version: "1.2.3" });
  assert.deepEqual(cached.operations, ["cached", "prune"]);
});

for (const stage of ["cached", "mkdir", "archive", "checksum", "verify", "extract"]) {
  await test(`disable during ${stage} prevents subsequent release effects and stale status`, async () => {
    const current = createResolution(stage);
    const pending = current.control.resolve();
    await current.entered.promise;
    current.control.disable();
    const beforeResume = [...current.operations];
    current.resumed.resolve();
    assert.equal(await pending, undefined);
    assert.deepEqual(current.operations, beforeResume);
    assert.equal(current.operations.at(-1), "disabled");
  });
}

import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { pruneDownloadedServerVersions } from "../../editors/vscode/src/server-cache.ts";

test("downloaded server cache retains current and newest previous versions", async (t) => {
  const serversDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-server-cache-"));
  t.after(() => fs.rmSync(serversDir, { force: true, recursive: true }));

  for (const [version, modified] of [
    ["0.515.0", 1],
    ["0.516.0", 2],
    ["0.517.0", 3],
  ] as const) {
    const versionDir = path.join(serversDir, version);
    fs.mkdirSync(versionDir);
    fs.writeFileSync(path.join(versionDir, "vize"), version);
    fs.utimesSync(versionDir, modified, modified);
  }
  fs.writeFileSync(path.join(serversDir, "README"), "leave me alone");

  const result = await pruneDownloadedServerVersions(serversDir, "0.517.0");

  assert.deepEqual(result, { removed: ["0.515.0"], failed: [] });
  assert.deepEqual(fs.readdirSync(serversDir).sort(), ["0.516.0", "0.517.0", "README"]);
});

test("downloaded server cache tolerates a missing root", async () => {
  const missing = path.join(os.tmpdir(), `vize-missing-server-cache-${process.pid}-${Date.now()}`);
  assert.deepEqual(await pruneDownloadedServerVersions(missing, "0.517.0"), {
    removed: [],
    failed: [],
  });
});

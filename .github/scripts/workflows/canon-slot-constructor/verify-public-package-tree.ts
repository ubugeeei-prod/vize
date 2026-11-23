import assert from "node:assert/strict";
import { join } from "node:path";
import { readJson, requiredEnv, writeJson, fileSha256, fileIdentity } from "../shared/files.ts";

import { createHash } from "node:crypto";
import { copyFileSync, readdirSync, statSync } from "node:fs";
const runner = requiredEnv("RUNNER_TEMP"),
  provider = join(runner, "slot-provider"),
  lockPath = join(provider, "package-lock.json");
const integrity =
  "sha512-uuQ+uXHCbsUTcSsA37edH3hxg3OpnM95odFmEkXsntEi3Wo6Wk8d/mCanBcxKgRr989bKdxrmZi7BQZ+SPbFlA==";
const lockSha256 = fileSha256(lockPath);
assert.equal(lockSha256, "9ca03c3cb103e1ff8d0a8c633145a743a056f21c5a0be56fa633a19541ab5256");
const pkg = readJson<{ packages: Record<string, { version: string; integrity: string }> }>(lockPath)
  .packages["node_modules/vize"];
assert.equal(pkg?.version, "0.603.0");
assert.equal(pkg.integrity, integrity);
const root = join(provider, "node_modules/vize"),
  manifest = readJson<{ version: string; bin: { vize: string } }>(join(root, "package.json"));
assert.equal(manifest.version, "0.603.0");
assert.equal(manifest.bin.vize, "bin/vize");
const names = readdirSync(root, { recursive: true, encoding: "utf8" }).filter(
  (file) => !file.split("/").includes("node_modules") && statSync(join(root, file)).isFile(),
);
// pathlib compares path components, so siblings sort before entries in a later sibling directory.
names.sort((a, b) => {
  const x = a.split("/"),
    y = b.split("/");
  for (let i = 0; i < Math.min(x.length, y.length); i++) {
    if (x[i] !== y[i]) return x[i]! < y[i]! ? -1 : 1;
  }
  return x.length - y.length;
});
const entries = names.map((path) => {
  const { bytes, sha256 } = fileIdentity(join(root, path));
  return { path, bytes, sha256 };
});
const tree = {
  files: entries.length,
  bytes: entries.reduce((total, entry) => total + entry.bytes, 0),
  sha256: createHash("sha256").update(JSON.stringify(entries)).digest("hex"),
};
assert.deepEqual(tree, {
  files: 78,
  bytes: 737358,
  sha256: "d7cdc31d3cd55b8495a3649598971950819b95becbcbd85c5bf6a088a6189d1e",
});
const entrypoint = fileIdentity(join(root, "bin/vize"));
assert.equal(entrypoint.sha256, "c468a9fe05d3f46520392ed4bdbccf1ff9b323308f9aa23cca0b507b829a6f54");
const evidence = join(runner, "slot-evidence");
writeJson(join(evidence, "public-baseline.json"), {
  version: manifest.version,
  lockSha256,
  registryIntegrity: pkg.integrity,
  packageTree: {
    ...tree,
    scope: "published vize package-owned files; nested dependencies are pinned by package-lock",
  },
  entrypoint,
  timingUsable: false,
});
for (const file of ["package.json", "package-lock.json"])
  copyFileSync(join(provider, file), join(evidence, file));

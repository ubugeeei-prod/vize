import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";

import { createLintSnapshotAlignment } from "../../../tests/_helpers/lint-snapshot-alignment.ts";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const predecessorCommit = process.argv[2] ?? "";
assert.match(predecessorCommit, /^[a-f\d]{40}$/);
const snapshotCommit = execFileSync("git", ["rev-parse", "HEAD"], {
  cwd: root,
  encoding: "utf8",
}).trim();
const projects = [
  "ant-design-vue",
  "elk",
  "misskey",
  "npmx.dev",
  "nuxt-ui",
  "reka-ui",
  "vuefes-2025",
];
const entries = projects.map((project) => {
  const snapshotPath = `tests/snapshots/lint/__snapshots__/${project}-lint.snap`;
  const previousRaw = execFileSync("git", ["show", `${predecessorCommit}:${snapshotPath}`], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
  const currentRaw = fs.readFileSync(path.join(root, snapshotPath), "utf8");
  const fixturePin = execFileSync("git", ["ls-tree", "HEAD", `tests/_fixtures/_git/${project}`], {
    cwd: root,
    encoding: "utf8",
  })
    .trim()
    .split(/\s+/)[2];
  assert.match(fixturePin, /^[a-f\d]{40}$/);
  return {
    path: snapshotPath,
    project,
    fixturePin,
    previousRaw,
    currentRaw,
    alignment: createLintSnapshotAlignment(snapshotPath, previousRaw, currentRaw),
  };
});
const directory = path.join(
  root,
  "tests/_fixtures/_projects/lint-review/2026-11-06-app-e2e-refresh",
);
fs.mkdirSync(directory, { recursive: true });
const snapshots = gzipSync(
  `${JSON.stringify(
    {
      schema: "vize-lint-snapshot-successor-v2",
      predecessorCommit,
      snapshotCommit,
      entries,
    },
    null,
    2,
  )}\n`,
  { level: 9 },
);
fs.writeFileSync(path.join(directory, "snapshots.json.gz"), snapshots);
fs.writeFileSync(
  path.join(directory, "inventory.json"),
  `${JSON.stringify(
    [
      {
        path: "snapshots.json.gz",
        bytes: snapshots.length,
        sha256: createHash("sha256").update(snapshots).digest("hex"),
      },
    ],
    null,
    2,
  )}\n`,
);

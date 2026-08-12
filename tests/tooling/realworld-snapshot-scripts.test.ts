import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { CHECK_FIXTURE_NODE_ARGS } from "./support/check-fixtures/manifest.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const realworldSnapshotApps = [
  "ant-design-vue",
  "element-plus",
  "elk",
  "hoppscotch",
  "misskey",
  "npmx",
  "nuxt-ui",
  "reka-ui",
  "voicevox",
  "vue-vben-admin",
  "vuefes",
] as const;

const serialTestConcurrency = /(?:^|\s)--test-concurrency=1(?:\s|$)/;

function readJsonFile<T>(...segments: string[]): T {
  return JSON.parse(fs.readFileSync(path.join(root, ...segments), "utf8")) as T;
}

test("real-world check and lint snapshots are wired into e2e scripts", () => {
  const pkg = readJsonFile<{ scripts: Record<string, string> }>("tests", "package.json");

  assert.match(pkg.scripts["test:build"], serialTestConcurrency);
  assert.match(pkg.scripts["test:check"], serialTestConcurrency);
  // The fixture lane runs through the supervisor now (#4126), so the flag it
  // passes every phase lives in the manifest rather than in the script string.
  // `CHECK_FIXTURE_NODE_ARGS` is an argv array, so the flag is one whole
  // element: match it exactly rather than reusing the pattern that exists to
  // find the flag inside a shell string.
  assert.ok(
    CHECK_FIXTURE_NODE_ARGS.includes("--test-concurrency=1"),
    "test:check:fixtures phases should stay serial",
  );

  for (const app of realworldSnapshotApps) {
    assert.match(
      pkg.scripts["test:check"],
      new RegExp(`snapshots/check/${app}\\.ts`),
      `${app} check snapshot should run in test:check`,
    );
    assert.match(
      pkg.scripts["test:lint"],
      new RegExp(`snapshots/lint/${app}\\.ts`),
      `${app} lint snapshot should run in test:lint`,
    );
  }
});

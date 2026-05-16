import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const realworldSnapshotApps = [
  "ant-design-vue",
  "elk",
  "misskey",
  "npmx",
  "nuxt-ui",
  "reka-ui",
  "vuefes",
] as const;

function readJsonFile<T>(...segments: string[]): T {
  return JSON.parse(fs.readFileSync(path.join(root, ...segments), "utf8")) as T;
}

test("real-world check and lint snapshots are wired into e2e scripts", () => {
  const pkg = readJsonFile<{ scripts: Record<string, string> }>("tests", "package.json");

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

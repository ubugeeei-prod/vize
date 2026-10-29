import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("shared mapper types are generated from the canonical standalone helper bodies", () => {
  const checked = spawnSync(
    process.execPath,
    ["npm/cli/scripts/generate-content-mapper-types.ts", "--check"],
    {
      cwd: root,
      encoding: "utf8",
      timeout: 30_000,
    },
  );
  assert.equal(checked.status, 0, `${checked.stdout}${checked.stderr}`);
  const generated = readFileSync(
    path.join(root, "npm/cli/generated/content-mapper-types.d.ts"),
    "utf8",
  );
  assert.doesNotMatch(generated, /declare\s+(global|namespace|function|var|const)/u);
  const manifest = JSON.parse(readFileSync(path.join(root, "npm/cli/package.json"), "utf8")) as {
    files: string[];
  };
  assert.ok(manifest.files.includes("generated/content-mapper-types.d.ts"));
});

test("ESM and CommonJS share exact helper types and observe the consumer Vue augmentation", () => {
  const checked = spawnSync(
    process.execPath,
    [
      path.join(root, "npm/cli/node_modules/typescript/bin/tsc"),
      "--ignoreConfig",
      "--noEmit",
      "--strict",
      "--target",
      "ES2022",
      "--module",
      "Node16",
      "--moduleResolution",
      "Node16",
      "npm/cli/tests/content-mapper-input.test-d.ts",
      "npm/cli/tests/content-mapper-input.test-d.cts",
    ],
    { cwd: root, encoding: "utf8", timeout: 60_000 },
  );
  assert.equal(checked.status, 0, `${checked.stdout}${checked.stderr}`);
});

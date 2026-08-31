import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const commandPath = "tools/commands/release/npm/inject-native-optional-deps.rs";

function runInjectNativeOptionalDeps(args: readonly string[]) {
  return spawnSync("rust-script", [commandPath, ...args], {
    cwd: repoRoot,
    encoding: "utf8",
  });
}

test("inject native optional deps updates only native optional dependency pins", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "vize-inject-native-"));
  const targetPath = path.join(tempDir, "package.json");
  const versionPath = path.join(tempDir, "version-package.json");

  try {
    writeFileSync(
      targetPath,
      `${JSON.stringify(
        {
          name: "@vizejs/example",
          version: "0.0.1",
          optionalDependencies: {
            "@vizejs/native-linux-x64-gnu": "0.0.1",
            "@vizejs/native-darwin-arm64": "0.0.1",
            fsevents: "^2.3.3",
          },
        },
        null,
        2,
      )}\n`,
    );
    writeFileSync(versionPath, `${JSON.stringify({ version: "1.2.3-beta.1" }, null, 2)}\n`);

    const result = runInjectNativeOptionalDeps([targetPath, versionPath]);
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /Updated optionalDependencies to native version 1\.2\.3-beta\.1/);

    const updated = JSON.parse(fs.readFileSync(targetPath, "utf8")) as {
      optionalDependencies: Record<string, string>;
    };
    assert.deepEqual(updated.optionalDependencies, {
      "@vizejs/native-darwin-arm64": "1.2.3-beta.1",
      "@vizejs/native-linux-x64-gnu": "1.2.3-beta.1",
      fsevents: "^2.3.3",
    });
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("inject native optional deps can print the updated target manifest", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "vize-inject-native-print-"));
  const targetPath = path.join(tempDir, "package.json");

  try {
    writeFileSync(
      targetPath,
      `${JSON.stringify(
        {
          name: "vize",
          version: "2.0.0",
          optionalDependencies: {
            "@vizejs/native-linux-arm64-gnu": "0.0.1",
          },
        },
        null,
        2,
      )}\n`,
    );

    const result = runInjectNativeOptionalDeps([targetPath, "--print"]);
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /"@vizejs\/native-linux-arm64-gnu": "2\.0\.0"/);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

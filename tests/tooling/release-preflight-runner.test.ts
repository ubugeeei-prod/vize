import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";

import {
  parseReleasePreflightMode,
  readPackageManifests,
} from "../../tools/github/release-preflight.mjs";
import { workspaceVersionFromCargoToml } from "../../tools/github/release-preflight-core.mjs";
import { repoRoot } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

const sha = "a".repeat(40);

test("release preflight CLI fails closed on unknown or ambiguous modes", () => {
  assert.equal(parseReleasePreflightMode([]), "bootstrap");
  assert.equal(parseReleasePreflightMode(["--verify-only"]), "verify-only");
  assert.equal(parseReleasePreflightMode(["--target-only"]), "target-only");
  assert.throws(() => parseReleasePreflightMode(["--verify-onyl"]), /Usage:/);
  assert.throws(() => parseReleasePreflightMode(["--verify-only", "--target-only"]), /Usage:/);
});

test("target-only mode verifies HEAD, current main, and the peeled remote tag", () => {
  const tempDir = fs.mkdtempSync(path.join(tmpdir(), "vize-release-target-"));
  const binDir = path.join(tempDir, "bin");
  const version = workspaceVersionFromCargoToml(
    fs.readFileSync(path.join(repoRoot, "Cargo.toml"), "utf8"),
  );
  fs.mkdirSync(binDir, { recursive: true });
  writeFakeCommand(
    binDir,
    "git",
    [
      "const args = process.argv.slice(2);",
      "const command = args.join(' ');",
      "if (command === 'rev-parse HEAD') console.log(process.env.TEST_RELEASE_SHA);",
      "else if (command === 'rev-parse refs/remotes/origin/main') console.log(process.env.TEST_MAIN_SHA);",
      "else if (args[0] === 'fetch') process.exit(0);",
      "else if (args[0] === 'ls-remote') {",
      "  console.log(`${process.env.TEST_TAG_OBJECT}\\trefs/tags/${process.env.TEST_TAG}`);",
      "  console.log(`${process.env.TEST_TAG_SHA}\\trefs/tags/${process.env.TEST_TAG}^{}`);",
      "} else if (args[0] === 'rev-list') console.log(`${process.env.TEST_RELEASE_SHA} ${process.env.TEST_BASE_SHA}`);",
      "else if (args[0] === 'merge-base') process.exit(0);",
      "else process.exit(2);",
    ].join("\n"),
  );
  const run = (overrides: Record<string, string> = {}) =>
    spawnSync(process.execPath, ["tools/github/release-preflight.mjs", "--target-only"], {
      cwd: repoRoot,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        GITHUB_REF_TYPE: "tag",
        GITHUB_REF_NAME: `v${version}`,
        GITHUB_SHA: sha,
        TEST_RELEASE_SHA: sha,
        TEST_MAIN_SHA: sha,
        TEST_TAG: `v${version}`,
        TEST_TAG_OBJECT: "c".repeat(40),
        TEST_TAG_SHA: sha,
        TEST_BASE_SHA: "b".repeat(40),
        ...overrides,
      },
    });

  try {
    const success = run();
    assert.equal(success.status, 0, `${success.stderr}\n${success.stdout}`.trim());
    assert.match(run({ TEST_MAIN_SHA: "d".repeat(40) }).stderr, /not the current origin\/main/);
    assert.match(run({ TEST_TAG_SHA: "e".repeat(40) }).stderr, /Remote tag .* points to/);
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test("release metadata inventory discovers every non-private npm and editor package", () => {
  assert.deepEqual(
    readPackageManifests().map((manifest) => manifest.path),
    [
      "editors/vscode-art/package.json",
      "editors/vscode/package.json",
      "npm/builder/rspack/package.json",
      "npm/builder/unplugin/package.json",
      "npm/builder/vite-musea/package.json",
      "npm/builder/vite/package.json",
      "npm/cli/package.json",
      "npm/framework/musea-nuxt/package.json",
      "npm/framework/nuxt/package.json",
      "npm/fresco-native/package.json",
      "npm/fresco/package.json",
      "npm/mcp-musea/package.json",
      "npm/native/package.json",
      "npm/oxint/package.json",
      "npm/wasm/package.json",
    ],
  );
});

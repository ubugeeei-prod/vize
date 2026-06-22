import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { repoRoot, runMoonScript } from "./_helpers/moonbit.ts";

function writeTempFile(contents: string): string {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-test-"));
  const file = path.join(dir, "input.yaml");
  fs.writeFileSync(file, contents);
  return file;
}

const cargoToml = fs.readFileSync(path.join(repoRoot, "Cargo.toml"), "utf8");
const currentVersion = cargoToml.match(/^version = "(.+)"$/m)?.[1];

assert.ok(currentVersion, "Failed to read current version from Cargo.toml");

test("release script fails clearly when stdin is not interactive", () => {
  const result = runMoonScript("release", ["minor"]);

  assert.equal(result.status, 1);
  assert.match(
    result.stderr,
    /Error: Confirmation requires an interactive terminal\. Re-run with -y to skip the prompt\.\n$/,
  );
  assert.match(
    result.stdout,
    new RegExp(
      `^Current version: ${currentVersion.replaceAll(".", "\\.")}\\nNew version: .+ \\(tag: v.+\\)\\n\\n$`,
    ),
  );
});

test("release script clears prerelease suffixes for stable bumps", () => {
  const cases = [
    ["1.2.3-alpha.1", "patch", "1.2.4"],
    ["1.2.3-beta", "minor", "1.3.0"],
    ["1.2.3-rc.1", "major", "2.0.0"],
    ["1.2.3-alpha.1", "release", "1.2.3"],
    ["1.2.3-alpha.1", "alpha", "1.2.3-alpha.2"],
  ] as const;

  for (const [current, bump, expected] of cases) {
    const result = runMoonScript("release", ["--print-bump", current, bump]);

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.equal(result.stdout.trim(), expected);
  }
});

test("release script rewrites only the native-binaries catalog block in pnpm-workspace.yaml", () => {
  const workspaceYaml = [
    "catalogs:",
    "  repo-tooling:",
    '    "@iarna/toml": "2.2.5"',
    "  some-other:",
    '    "@vizejs/native-darwin-arm64": "0.106.0"',
    "  # Published native binary packages.",
    "  native-binaries:",
    '    "@vizejs/native-darwin-arm64": "0.106.0"',
    '    "@vizejs/native-darwin-x64": "0.106.0"',
    '    "@vizejs/native-linux-arm64-gnu": "0.106.0"',
    "",
    "peerDependencyRules:",
    "  allowAny:",
    '    - "*"',
    "",
  ].join("\n");
  const inputPath = writeTempFile(workspaceYaml);

  const result = runMoonScript("release", [
    "--print-workspace-catalog-update",
    inputPath,
    "0.106.0",
    "0.107.0",
  ]);

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
  const lines = result.stdout.split("\n");

  const otherCatalogLine = lines.find((line) =>
    line.startsWith('    "@vizejs/native-darwin-arm64": '),
  );
  assert.ok(otherCatalogLine, "first native-darwin-arm64 line preserved");
  assert.equal(
    otherCatalogLine,
    '    "@vizejs/native-darwin-arm64": "0.106.0"',
    "non-native-binaries catalog must not be rewritten",
  );

  const nativeBlockStart = lines.indexOf("  native-binaries:");
  assert.notEqual(nativeBlockStart, -1, "native-binaries header preserved");
  assert.equal(lines[nativeBlockStart + 1], '    "@vizejs/native-darwin-arm64": "0.107.0"');
  assert.equal(lines[nativeBlockStart + 2], '    "@vizejs/native-darwin-x64": "0.107.0"');
  assert.equal(lines[nativeBlockStart + 3], '    "@vizejs/native-linux-arm64-gnu": "0.107.0"');

  assert.ok(result.stdout.includes("peerDependencyRules:"), "later sections preserved");
});

test("release script rewrites only the native-binaries catalog block in pnpm-lock.yaml", () => {
  const lockfile = [
    "catalogs:",
    "  linting:",
    "    oxlint:",
    "      specifier: 1.64.0",
    "      version: 1.64.0",
    "  native-binaries:",
    "    '@vizejs/native-darwin-arm64':",
    "      specifier: 0.106.0",
    "      version: 0.106.0",
    "    '@vizejs/native-darwin-x64':",
    "      specifier: 0.106.0",
    "      version: 0.106.0",
    "  oxc:",
    "    oxc-transform:",
    "      specifier: 0.130.0",
    "      version: 0.130.0",
    "importers:",
    "  npm/native:",
    "    optionalDependencies:",
    "      '@vizejs/native-darwin-arm64':",
    "        specifier: catalog:native-binaries",
    "        version: 0.106.0",
    "packages:",
    "  '@vizejs/native-darwin-arm64@0.106.0':",
    "    resolution: {integrity: sha512-AAA==}",
    "",
  ].join("\n");
  const inputPath = writeTempFile(lockfile);

  const result = runMoonScript("release", [
    "--print-lockfile-catalog-update",
    inputPath,
    "0.106.0",
    "0.107.0",
  ]);

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
  const out = result.stdout;

  assert.match(
    out,
    /native-binaries:\n {4}'@vizejs\/native-darwin-arm64':\n {6}specifier: 0\.107\.0\n {6}version: 0\.107\.0\n/,
  );
  assert.match(
    out,
    / {4}'@vizejs\/native-darwin-x64':\n {6}specifier: 0\.107\.0\n {6}version: 0\.107\.0\n/,
  );

  assert.match(out, /linting:\n {4}oxlint:\n {6}specifier: 1\.64\.0\n {6}version: 1\.64\.0\n/);

  assert.ok(
    out.includes("        version: 0.106.0"),
    "project importer version (six-space indent) preserved",
  );
  assert.ok(
    out.includes("'@vizejs/native-darwin-arm64@0.106.0':"),
    "packages section key preserved",
  );
  assert.ok(out.includes("resolution: {integrity: sha512-AAA==}"), "integrity hash preserved");
});

test("release script includes the VS Code extension package in extra synced manifests", () => {
  const result = runMoonScript("release", ["--print-extra-package-json-paths"]);

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
  assert.ok(
    result.stdout.split("\n").includes("editors/vscode/package.json"),
    "VS Code extension version must be bumped with release commits",
  );
});

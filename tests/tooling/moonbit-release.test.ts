import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { repoRoot, runMoonScript } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

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

test("release script includes nested release packages in extra synced manifests", () => {
  const result = runMoonScript("release", ["--print-extra-package-json-paths"]);

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
  const paths = result.stdout.split("\n");

  for (const manifestPath of [
    "editors/vscode/package.json",
    "editors/vscode-art/package.json",
    "npm/builder/rspack/package.json",
    "npm/builder/unplugin/package.json",
    "npm/builder/vite/package.json",
    "npm/builder/vite-musea/package.json",
    "npm/framework/musea-nuxt/package.json",
    "npm/framework/nuxt/package.json",
  ]) {
    assert.ok(
      paths.includes(manifestPath),
      `${manifestPath} version must be bumped with release commits`,
    );
  }
});

test("release script creates immutable tags and pushes main and tag atomically", () => {
  const fixture = runRepositoryGuardFixture({ branch: "main" });

  try {
    assert.equal(fixture.result.status, 0, fixture.result.stderr);
    assert.match(fixture.gitLog, /^commit --no-verify -m chore: release v0\.290\.1$/m);
    assert.match(fixture.gitLog, /^tag -a v0\.290\.1 -m Release 0\.290\.1$/m);
    assert.match(fixture.gitLog, /^push --atomic origin main refs\/tags\/v0\.290\.1$/m);
    assert.doesNotMatch(fixture.gitLog, /--force-tag|(?:^|\s)--force(?:\s|$)|--allow-empty/);
  } finally {
    fs.rmSync(fixture.tempDir, { recursive: true, force: true });
  }
});

test("release script explains local cleanup after an atomic push failure", () => {
  const fixture = runRepositoryGuardFixture({ branch: "main", pushFails: true });

  try {
    assert.equal(fixture.result.status, 1);
    assert.match(fixture.result.stderr, /Failed to atomically push main and the release tag/);
    assert.match(fixture.result.stderr, /git tag -d v0\.290\.1/);
    assert.match(fixture.result.stderr, /git reset --hard origin\/main/);
  } finally {
    fs.rmSync(fixture.tempDir, { recursive: true, force: true });
  }
});

test("release script rejects the removed force-tag escape hatch", () => {
  const result = runMoonScript("release", ["patch", "-y", "--force-tag"]);

  assert.equal(result.status, 1);
  assert.match(result.stderr, /--force-tag is not supported; published release tags are immutable/);
});

interface RepositoryGuardOptions {
  branch: string;
  dirty?: boolean;
  ancestor?: boolean;
  headSha?: string;
  remoteSha?: string;
  localTagExists?: boolean;
  remoteTagExists?: boolean;
  pushFails?: boolean;
  stagedFiles?: boolean;
  manifestTestFails?: boolean;
}

function runRepositoryGuardFixture(options: RepositoryGuardOptions) {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-guard-"));
  const binDir = path.join(tempDir, "bin");
  const gitLogPath = path.join(tempDir, "git.log");
  const cargoTomlPath = path.join(tempDir, "Cargo.toml");
  const cargoToml = '[workspace.package]\nversion = "0.290.0"\n';
  fs.mkdirSync(binDir, { recursive: true });
  fs.mkdirSync(path.join(tempDir, "npm"));
  fs.mkdirSync(path.join(tempDir, "tests/tooling"), { recursive: true });
  fs.writeFileSync(gitLogPath, "");
  fs.writeFileSync(cargoTomlPath, cargoToml);
  fs.writeFileSync(path.join(tempDir, "pnpm-workspace.yaml"), "");
  fs.writeFileSync(path.join(tempDir, "pnpm-lock.yaml"), "");
  fs.writeFileSync(
    path.join(tempDir, "tests/tooling/package-manifests.test.ts"),
    options.manifestTestFails ? 'throw new Error("manifest drift");\n' : "",
  );
  writeFakeCommand(binDir, "cargo", "process.exit(0);");
  writeFakeCommand(
    binDir,
    "git",
    [
      "const fs = require('node:fs');",
      "const args = process.argv.slice(2);",
      "fs.appendFileSync(process.env.GIT_LOG, args.join(' ') + '\\n');",
      "if (args[0] === 'branch') { console.log(process.env.TEST_BRANCH); process.exit(0); }",
      "if (args[0] === 'status') { if (process.env.TEST_DIRTY === 'true') console.log(' M Cargo.toml'); process.exit(0); }",
      "if (args[0] === 'fetch') process.exit(0);",
      "if (args[0] === 'merge-base') process.exit(process.env.TEST_ANCESTOR === 'false' ? 1 : 0);",
      "if (args[0] === 'rev-parse' && args.includes('--verify')) process.exit(process.env.LOCAL_TAG_EXISTS === 'true' ? 0 : 1);",
      "if (args[0] === 'rev-parse') { console.log(args.at(-1) === 'HEAD' ? process.env.TEST_HEAD_SHA : process.env.TEST_REMOTE_SHA); process.exit(0); }",
      "if (args[0] === 'ls-remote' && process.env.REMOTE_TAG_EXISTS === 'true') { console.log('bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\\t' + args.at(-1)); process.exit(0); }",
      "if (args[0] === 'ls-remote') process.exit(2);",
      "if (args[0] === 'diff' && args.includes('--cached')) { if (process.env.TEST_STAGED_FILES !== 'false') console.log('Cargo.toml'); process.exit(0); }",
      "if (args[0] === 'push') process.exit(process.env.TEST_PUSH_FAIL === 'true' ? 1 : 0);",
      "if (['add', 'commit', 'tag'].includes(args[0])) process.exit(0);",
      "process.exit(1);",
    ].join("\n"),
  );

  const result = runMoonScript("release", ["patch", "-y"], {
    cwd: tempDir,
    env: {
      PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
      GIT_LOG: gitLogPath,
      TEST_BRANCH: options.branch,
      TEST_DIRTY: String(options.dirty ?? false),
      TEST_ANCESTOR: String(options.ancestor ?? true),
      TEST_HEAD_SHA: options.headSha ?? "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      TEST_REMOTE_SHA: options.remoteSha ?? "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      LOCAL_TAG_EXISTS: String(options.localTagExists ?? false),
      REMOTE_TAG_EXISTS: String(options.remoteTagExists),
      TEST_PUSH_FAIL: String(options.pushFails ?? false),
      TEST_STAGED_FILES: String(options.stagedFiles ?? true),
      VIZE_RELEASE_GUARD_SCRIPT: path.join(repoRoot, "tools/github/release-local-guard.mjs"),
    },
  });
  const gitLog = fs.readFileSync(gitLogPath, "utf8");
  return { cargoToml, cargoTomlPath, gitLog, result, tempDir };
}

test("release script refuses to create an empty release commit", () => {
  const fixture = runRepositoryGuardFixture({ branch: "main", stagedFiles: false });

  try {
    assert.equal(fixture.result.status, 1);
    assert.match(fixture.result.stderr, /No release changes were staged/);
    assert.doesNotMatch(fixture.gitLog, /^(?:commit|tag|push)\b/m);
  } finally {
    fs.rmSync(fixture.tempDir, { recursive: true, force: true });
  }
});

test("release script explains cleanup after manifest verification fails", () => {
  const fixture = runRepositoryGuardFixture({ branch: "main", manifestTestFails: true });

  try {
    assert.equal(fixture.result.status, 1);
    assert.match(fixture.result.stderr, /package manifest alignment tests failed/);
    assert.match(fixture.result.stderr, /git reset --hard origin\/main/);
    assert.doesNotMatch(fixture.gitLog, /^(?:tag|push)\b/m);
  } finally {
    fs.rmSync(fixture.tempDir, { recursive: true, force: true });
  }
});

test("release repository guard rejects unsafe refs before mutation", () => {
  const cases: Array<[RepositoryGuardOptions, RegExp]> = [
    [{ branch: "feature/unsafe-release" }, /must be prepared from the local main branch/],
    [{ branch: "" }, /must be prepared from the local main branch/],
    [{ branch: "main", dirty: true }, /uncommitted changes/],
    [{ branch: "main", ancestor: false }, /HEAD is not reachable from the current origin\/main/],
    [
      { branch: "main", remoteSha: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" },
      /HEAD must exactly match the current origin\/main/,
    ],
    [{ branch: "main", localTagExists: true }, /Tag v0\.290\.1 already exists locally/],
    [{ branch: "main", remoteTagExists: true }, /Remote tag v0\.290\.1 already exists/],
  ];
  for (const [options, message] of cases) {
    const fixture = runRepositoryGuardFixture(options);
    try {
      assert.equal(fixture.result.status, 1);
      assert.match(fixture.result.stderr, message);
      assert.doesNotMatch(fixture.gitLog, /^(?:add|commit|tag|push)\b/m);
      assert.equal(fs.readFileSync(fixture.cargoTomlPath, "utf8"), fixture.cargoToml);
    } finally {
      fs.rmSync(fixture.tempDir, { recursive: true, force: true });
    }
  }
});

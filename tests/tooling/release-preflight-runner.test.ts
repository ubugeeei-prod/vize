import assert from "node:assert/strict";
import { Buffer } from "node:buffer";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";
import { crc32 } from "node:zlib";

import {
  parseReleasePreflightMode,
  readPackageManifests,
} from "../../legacy-tools/github/release-preflight.mjs";
import { workspaceVersionFromCargoToml } from "../../legacy-tools/github/release-preflight-core.mjs";
import { repoRoot } from "./_helpers/moonbit.ts";
import {
  realProjectArtifacts,
  shardEntries,
} from "./_helpers/release-preflight-matrix-evidence-fixture.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

const sha = "a".repeat(40);

test("release preflight CLI fails closed on unknown or ambiguous modes", () => {
  assert.equal(parseReleasePreflightMode([]), "bootstrap");
  assert.equal(parseReleasePreflightMode(["--verify-only"]), "verify-only");
  assert.equal(parseReleasePreflightMode(["--target-only"]), "target-only");
  assert.throws(() => parseReleasePreflightMode(["--verify-onyl"]), /Usage:/);
  assert.throws(() => parseReleasePreflightMode(["--verify-only", "--target-only"]), /Usage:/);
});

test("target-only mode verifies the hydrated main ref, HEAD, and the peeled remote tag", () => {
  const tempDir = fs.mkdtempSync(path.join(tmpdir(), "vize-release-target-"));
  const binDir = path.join(tempDir, "bin");
  const version = workspaceVersionFromCargoToml(
    fs.readFileSync(path.join(repoRoot, "Cargo.toml"), "utf8"),
  );
  const trackedManifests = spawnSync("git", ["ls-files", "-z", "--", "editors", "npm"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(trackedManifests.status, 0, trackedManifests.stderr);
  fs.mkdirSync(binDir, { recursive: true });
  writeFakeCommand(
    binDir,
    "git",
    [
      "const args = process.argv.slice(2);",
      "const command = args.join(' ');",
      "if (command === 'rev-parse HEAD') console.log(process.env.TEST_HEAD_SHA);",
      "else if (command === 'rev-parse refs/remotes/origin/main') console.log(process.env.TEST_MAIN_SHA);",
      "else if (args[0] === 'ls-files') process.stdout.write(JSON.parse(process.env.TEST_PACKAGE_MANIFESTS).join('\\0') + '\\0');",
      "else if (args[0] === 'ls-remote') {",
      "  console.log(`${process.env.TEST_TAG_OBJECT}\\trefs/tags/${process.env.TEST_TAG}`);",
      "  console.log(`${process.env.TEST_TAG_SHA}\\trefs/tags/${process.env.TEST_TAG}^{}`);",
      "} else if (command === 'rev-list --first-parent refs/remotes/origin/main') console.log(process.env.TEST_MAIN_FIRST_PARENT_HISTORY);",
      "else if (command === 'show refs/remotes/origin/main:Cargo.toml') process.stdout.write(process.env.TEST_MAIN_CARGO_TOML);",
      "else if (args[0] === 'rev-list') console.log(`${process.env.TEST_RELEASE_SHA} ${process.env.TEST_BASE_SHA}`);",
      "else if (args[0] === 'merge-base') process.exit(0);",
      "else process.exit(2);",
    ].join("\n"),
  );
  const run = (overrides: Record<string, string> = {}) =>
    spawnSync("rust-script", ["tools/commands/ci/github/release-preflight.rs", "--target-only"], {
      cwd: repoRoot,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        GITHUB_REF_TYPE: "tag",
        GITHUB_REF_NAME: `v${version}`,
        GITHUB_SHA: sha,
        TEST_HEAD_SHA: sha,
        TEST_RELEASE_SHA: sha,
        TEST_MAIN_SHA: sha,
        TEST_MAIN_FIRST_PARENT_HISTORY: [sha, "b".repeat(40)].join("\n"),
        TEST_MAIN_CARGO_TOML: `[workspace.package]\nversion = "${version}"\n`,
        TEST_TAG: `v${version}`,
        TEST_TAG_OBJECT: "c".repeat(40),
        TEST_TAG_SHA: sha,
        TEST_BASE_SHA: "b".repeat(40),
        TEST_PACKAGE_MANIFESTS: JSON.stringify(trackedManifests.stdout.split("\0").filter(Boolean)),
        ...overrides,
      },
    });

  const outcome = (result: ReturnType<typeof run>) => [result.status, result.stderr];

  try {
    const success = run();
    assert.deepEqual(outcome(success), [0, ""], `${success.stderr}\n${success.stdout}`.trim());

    // The repository's merge automation lands PRs throughout the 30-40 minute
    // gate wait. Ordinary drift keeps the workspace version, so the release
    // still owns its version line and the gates measured at the tag still say
    // what they said.
    assert.deepEqual(outcome(run({ TEST_MAIN_SHA: "d".repeat(40) })), [0, ""]);

    // A second release commit is the one kind of drift that does invalidate
    // this one: finishing now publishes a lower version after a higher one.
    assert.deepEqual(
      outcome(
        run({
          TEST_MAIN_SHA: "d".repeat(40),
          TEST_MAIN_CARGO_TOML: '[workspace.package]\nversion = "99.99.99"\n',
        }),
      ),
      [
        1,
        `Release v${version} (${sha}) is superseded: origin/main ${"d".repeat(40)} is at workspace version 99.99.99, not ${version}. Publishing it now would ship an older version after a newer one; cut the next release instead.\n`,
      ],
    );

    assert.deepEqual(outcome(run({ TEST_HEAD_SHA: "f".repeat(40) })), [
      1,
      `Checked out HEAD ${"f".repeat(40)} does not match release event SHA ${sha}\n`,
    ]);

    assert.deepEqual(
      outcome(
        run({
          TEST_MAIN_SHA: "d".repeat(40),
          TEST_MAIN_FIRST_PARENT_HISTORY: ["d".repeat(40), "b".repeat(40)].join("\n"),
        }),
      ),
      [
        1,
        `Release commit ${sha} is not on the first-parent history of current origin/main ${"d".repeat(40)}\n`,
      ],
    );

    assert.deepEqual(outcome(run({ TEST_TAG_SHA: "e".repeat(40) })), [
      1,
      `Remote tag v${version} points to ${"e".repeat(40)}, expected ${sha}\n`,
    ]);
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
      "npm/compose/core/package.json",
      "npm/framework/musea-nuxt/package.json",
      "npm/framework/nuxt-lint-config/package.json",
      "npm/framework/nuxt/package.json",
      "npm/fresco-native/package.json",
      "npm/fresco/package.json",
      "npm/marquette/package.json",
      "npm/mcp-musea/package.json",
      "npm/native/package.json",
      "npm/oxlint/package.json",
      "npm/ui/package.json",
      "npm/wasm/package.json",
    ],
  );
});

test("verify-only mode accepts flat job evidence returned by pagination", () => {
  const tempDir = fs.mkdtempSync(path.join(tmpdir(), "vize-release-jobs-"));
  const binDir = path.join(tempDir, "bin");
  const version = workspaceVersionFromCargoToml(
    fs.readFileSync(path.join(repoRoot, "Cargo.toml"), "utf8"),
  );
  const trackedManifests = spawnSync("git", ["ls-files", "-z", "--", "editors", "npm"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(trackedManifests.status, 0, trackedManifests.stderr);
  const baseSha = "b".repeat(40);
  const tag = `v${version}`;
  const run = (
    id: number,
    name: string,
    path: string,
    event: string,
    displayTitle = `${name} release evidence`,
  ) => ({
    id,
    name,
    display_title: displayTitle,
    path,
    event,
    head_branch: event === "push" ? "main" : tag,
    head_sha: sha,
    status: "completed",
    conclusion: "success",
    html_url: `https://example.test/runs/${id}`,
    created_at: `2026-07-12T00:${id}:00Z`,
    run_started_at: `2026-07-12T00:${id}:00Z`,
    updated_at: `2026-07-12T00:${id}:00Z`,
  });
  const jobs = Object.fromEntries(
    [
      [101, ["test-scripts"]],
      [102, ["pr-benchmark-budget"]],
      [
        103,
        [
          "Fuzz sfc_parse",
          "Fuzz template_lexer",
          "Fuzz js_ts_expression",
          "Fuzz css_parse",
          "Fuzz template_compile",
        ],
      ],
      [106, Array.from({ length: 22 }, (_, shard) => `real projects (${shard}/22)`)],
    ].map(([id, names]) => [
      id,
      (names as string[]).map((name) => ({ name, status: "completed", conclusion: "success" })),
    ]),
  );
  const runs = [
    run(101, "Check", ".github/workflows/check.yml", "push"),
    run(
      102,
      "Benchmark",
      ".github/workflows/benchmark.yml",
      "workflow_dispatch",
      `Benchmark ${baseSha}...${sha}`,
    ),
    run(103, "Fuzz", ".github/workflows/fuzz.yml", "workflow_dispatch", `Fuzz replay @ ${sha}`),
    run(104, "Miri", ".github/workflows/miri.yml", "push"),
    run(105, "Docs build", ".github/workflows/build-docs.yml", "push"),
    run(
      106,
      "Real Project Matrix",
      ".github/workflows/real-project-matrix.yml",
      "workflow_dispatch",
      `Real Project Matrix @ ${sha}`,
    ),
  ];
  const matrixRun = runs.find((candidate) => candidate.name === "Real Project Matrix");
  assert.ok(matrixRun);
  const typecheckIds = typecheckProjectIds();
  const artifacts = realProjectArtifacts(matrixRun);
  const artifactZips = Object.fromEntries(
    artifacts.map((artifact, shard) => [
      String(shard),
      storedZip(
        Object.entries(shardEntries(shard, { typecheckProject: typecheckIds[shard] ?? null })),
      ).toString("base64"),
    ]),
  );
  fs.mkdirSync(binDir, { recursive: true });
  writeFakeCommand(
    binDir,
    "git",
    [
      "const args = process.argv.slice(2);",
      "const command = args.join(' ');",
      "if (command === 'rev-parse HEAD') console.log(process.env.TEST_RELEASE_SHA);",
      "else if (command === 'rev-parse refs/remotes/origin/main') console.log(process.env.TEST_RELEASE_SHA);",
      "else if (command === 'rev-list --first-parent refs/remotes/origin/main') console.log(process.env.TEST_RELEASE_SHA);",
      "else if (command === 'show refs/remotes/origin/main:Cargo.toml') process.stdout.write(process.env.TEST_MAIN_CARGO_TOML);",
      "else if (args[0] === 'ls-files') process.stdout.write(JSON.parse(process.env.TEST_PACKAGE_MANIFESTS).join('\\0') + '\\0');",
      "else if (args[0] === 'ls-remote') console.log(`${process.env.TEST_TAG_SHA}\\trefs/tags/${process.env.TEST_TAG}`);",
      "else if (args[0] === 'rev-list') console.log(`${process.env.TEST_RELEASE_SHA} ${process.env.TEST_BASE_SHA}`);",
      "else if (args[0] === 'merge-base') process.exit(0);",
      "else if (args[0] === 'diff') console.log('crates/vize/src/lib.rs');",
      "else process.exit(2);",
    ].join("\n"),
  );
  writeFakeCommand(
    binDir,
    "curl",
    [
      "const fs = require('node:fs');",
      "const url = process.argv.at(-1);",
      "const args = process.argv.slice(2);",
      "const send = (value) => process.stdout.write(JSON.stringify(value));",
      "if (url.includes('/actions/runs?')) send({ workflow_runs: JSON.parse(process.env.TEST_RUNS) });",
      "else if (url.includes('/actions/runs/') && url.includes('/artifacts')) send({ artifacts: JSON.parse(process.env.TEST_ARTIFACTS) });",
      "else if (url.startsWith('https://example.test/artifacts/')) {",
      "  const output = args[args.indexOf('--output') + 1];",
      "  const shard = url.match(/\\/artifacts\\/(\\d+)\\.zip$/)?.[1];",
      "  const zip = JSON.parse(process.env.TEST_ARTIFACT_ZIPS)[shard];",
      "  if (output == null || shard == null || zip == null) process.exit(22);",
      "  fs.writeFileSync(output, Buffer.from(zip, 'base64'));",
      "} else if (url.includes('/actions/runs/')) {",
      "  const id = url.match(/\\/actions\\/runs\\/(\\d+)\\/jobs/)?.[1];",
      "  send({ jobs: JSON.parse(process.env.TEST_JOBS)[id] ?? [] });",
      "} else if (url.includes('/issues?')) send([]);",
      "else { console.error(`unexpected curl url: ${url}`); process.exit(22); }",
    ].join("\n"),
  );

  try {
    const result = spawnSync(
      "rust-script",
      ["tools/commands/ci/github/release-preflight.rs", "--verify-only"],
      {
        cwd: repoRoot,
        encoding: "utf8",
        env: {
          ...process.env,
          PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
          GITHUB_API_URL: "https://api.github.test",
          GITHUB_REPOSITORY: "owner/repository",
          GITHUB_REF_NAME: tag,
          GITHUB_REF_TYPE: "tag",
          GITHUB_SHA: sha,
          GITHUB_TOKEN: "secret",
          TEST_ARTIFACT_ZIPS: JSON.stringify(artifactZips),
          TEST_ARTIFACTS: JSON.stringify(artifacts),
          TEST_BASE_SHA: baseSha,
          TEST_JOBS: JSON.stringify(jobs),
          TEST_MAIN_CARGO_TOML: `[workspace.package]\nversion = "${version}"\n`,
          TEST_PACKAGE_MANIFESTS: JSON.stringify(
            trackedManifests.stdout.split("\0").filter(Boolean),
          ),
          TEST_RELEASE_SHA: sha,
          TEST_RUNS: JSON.stringify(runs),
          TEST_TAG: tag,
          TEST_TAG_SHA: sha,
        },
      },
    );
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, new RegExp(`Release preflight passed for ${tag}`));
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

test("release metadata inventory ignores untracked package manifests", () => {
  const untrackedDirectory = path.join(repoRoot, "npm", `.preflight-untracked-${process.pid}`);
  fs.mkdirSync(untrackedDirectory, { recursive: true });
  fs.writeFileSync(
    path.join(untrackedDirectory, "package.json"),
    '{"name":"untracked-release-output","version":"9.9.9"}',
  );
  try {
    assert.equal(
      readPackageManifests().some((manifest) => manifest.path.includes(".preflight-untracked-")),
      false,
    );
  } finally {
    fs.rmSync(untrackedDirectory, { recursive: true, force: true });
  }
});

function typecheckProjectIds() {
  const registry = JSON.parse(
    fs.readFileSync(path.join(repoRoot, "tests/_fixtures/vue-ecosystem-fixtures.json"), "utf8"),
  );
  return registry.projects
    .filter(
      (project: { typecheckPerformance?: { enabled?: boolean } }) =>
        project.typecheckPerformance?.enabled === true,
    )
    .map((project: { id: string }) => project.id);
}

// Minimal stored ZIP writer so the Rust release-preflight path validates a real
// archive without depending on the system `zip` command.
function storedZip(files: [string, string][]) {
  const locals: Buffer[] = [];
  const central: Buffer[] = [];
  let offset = 0;
  for (const [name, text] of files) {
    const nameBytes = Buffer.from(name, "utf8");
    const data = Buffer.from(text, "utf8");
    const checksum = crc32(data);
    const local = Buffer.alloc(30 + nameBytes.byteLength);
    local.writeUInt32LE(0x04034b50, 0);
    local.writeUInt16LE(20, 4);
    local.writeUInt32LE(checksum, 14);
    local.writeUInt32LE(data.byteLength, 18);
    local.writeUInt32LE(data.byteLength, 22);
    local.writeUInt16LE(nameBytes.byteLength, 26);
    nameBytes.copy(local, 30);
    locals.push(local, data);

    const header = Buffer.alloc(46 + nameBytes.byteLength);
    header.writeUInt32LE(0x02014b50, 0);
    header.writeUInt16LE(20, 4);
    header.writeUInt16LE(20, 6);
    header.writeUInt32LE(checksum, 16);
    header.writeUInt32LE(data.byteLength, 20);
    header.writeUInt32LE(data.byteLength, 24);
    header.writeUInt16LE(nameBytes.byteLength, 28);
    header.writeUInt32LE(offset, 42);
    nameBytes.copy(header, 46);
    central.push(header);
    offset += local.byteLength + data.byteLength;
  }
  const directory = Buffer.concat(central);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0);
  end.writeUInt16LE(files.length, 8);
  end.writeUInt16LE(files.length, 10);
  end.writeUInt32LE(directory.byteLength, 12);
  end.writeUInt32LE(offset, 16);
  return Buffer.concat([...locals, directory, end]);
}

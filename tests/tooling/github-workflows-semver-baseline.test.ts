import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

test("immutable renamed-package baseline rejects corruption and preserves old manifest API", () => {
  const result = spawnSync(
    "rust-script",
    ["--test", path.join(repoRoot, "tools/commands/ci/github/semver-baseline.rs")],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
});

test("registry failures cannot turn the first-publish baseline into a skipped check", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-semver-registry-"));
  const bin = path.join(root, "bin");
  fs.mkdirSync(bin);
  fs.mkdirSync(path.join(root, "crates", "vize_l1_to_l2"), { recursive: true });
  fs.writeFileSync(path.join(root, "Cargo.toml"), '[workspace.package]\nversion = "0.429.0"\n');
  fs.writeFileSync(
    path.join(root, "crates", "vize_l1_to_l2", "Cargo.toml"),
    '[package]\nname = "vize_l1_to_l2"\nversion.workspace = true\n',
  );
  writeFakeCommand(
    bin,
    "curl",
    [
      "const fs = require('node:fs');",
      "const args = process.argv.slice(2);",
      "const output = args[args.indexOf('--output') + 1];",
      "if (args.at(-1).includes('static.crates.io')) { fs.writeFileSync(output, 'changed Rust API'); process.stdout.write('200'); process.exit(0); }",
      "const mode = process.env.SEMVER_REGISTRY_MODE;",
      "if (mode === '404' || mode === '403') { fs.writeFileSync(output, 'missing'); process.stdout.write(mode); process.exit(22); }",
      "fs.writeFileSync(output, mode === 'malformed' ? 'bad json' : JSON.stringify({ name: 'vize_l1_to_l2', vers: '0.428.1', yanked: false }));",
      "process.stdout.write('200');",
    ].join("\n"),
  );
  try {
    for (const [mode, diagnostic] of [
      ["403", /registry request failed/u],
      ["malformed", /invalid registry index/u],
      ["404", /archive checksum mismatch/u],
      ["registered", undefined],
    ] as const) {
      const output = fs.mkdtempSync(path.join(root, "baseline-"));
      const result = spawnSync(
        "rust-script",
        [
          path.join(repoRoot, "tools/commands/ci/github/semver-baseline.rs"),
          "vize_l1_to_l2",
          output,
        ],
        {
          cwd: root,
          encoding: "utf8",
          env: {
            ...process.env,
            PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
            SEMVER_REGISTRY_MODE: mode,
          },
        },
      );
      if (diagnostic) {
        assert.notEqual(result.status, 0, mode);
        assert.match(result.stderr, diagnostic);
      } else {
        assert.equal(result.status, 0, result.stderr);
        assert.equal(result.stdout, "");
      }
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("a renamed package uses the exact Git parent source and version before registry lookup", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-semver-git-"));
  const git = (args: string[]) => {
    const result = spawnSync("git", args, { cwd: root, encoding: "utf8" });
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`);
    return result.stdout.trim();
  };
  const oldCrate = path.join(root, "crates", "vize_s1_to_s2");
  const oldSource = 'pub fn initial_fourteen_api() -> &\'static str { "exact parent" }\n';
  fs.mkdirSync(path.join(oldCrate, "src"), { recursive: true });
  fs.writeFileSync(
    path.join(root, "Cargo.toml"),
    '[workspace.package]\nversion = "0.427.99"\n[workspace.dependencies]\nvize_s1_to_s2 = { path = "crates/vize_s1_to_s2", version = "=0.427.99" }\n',
  );
  fs.writeFileSync(
    path.join(oldCrate, "Cargo.toml"),
    '[package]\nname = "vize_s1_to_s2"\nversion.workspace = true\n[dependencies]\nvize_s1.workspace = true\n',
  );
  fs.writeFileSync(path.join(oldCrate, "src/lib.rs"), oldSource);
  git(["init", "-q"]);
  git(["add", "."]);
  git([
    "-c",
    "user.name=Vize",
    "-c",
    "user.email=vize@example.com",
    "-c",
    "commit.gpgsign=false",
    "commit",
    "-qm",
    "exact API parent",
  ]);
  const base = git(["rev-parse", "HEAD"]);
  const newCrate = path.join(root, "crates", "vize_l1_to_l2");
  fs.renameSync(oldCrate, newCrate);
  fs.writeFileSync(
    path.join(newCrate, "Cargo.toml"),
    '[package]\nname = "vize_l1_to_l2"\nversion.workspace = true\n',
  );
  fs.writeFileSync(path.join(newCrate, "src/lib.rs"), "pub fn changed_current_api() {}\n");
  const run = (revision: string) =>
    spawnSync(
      "rust-script",
      [
        path.join(repoRoot, "tools/commands/ci/github/semver-baseline.rs"),
        "vize_l1_to_l2",
        fs.mkdtempSync(path.join(root, "baseline-")),
        revision,
      ],
      { cwd: root, encoding: "utf8" },
    );
  try {
    const exact = run(base);
    assert.equal(exact.status, 0, exact.stderr);
    const baseline = exact.stdout.trim();
    assert.equal(fs.readFileSync(path.join(baseline, "src/lib.rs"), "utf8"), oldSource);
    const manifest = fs.readFileSync(path.join(baseline, "Cargo.toml"), "utf8");
    assert.match(manifest, /name = "vize_l1_to_l2"/u);
    assert.match(manifest, /vize_s1\.workspace = true/u);
    const workspace = fs.readFileSync(path.resolve(baseline, "../../Cargo.toml"), "utf8");
    assert.match(workspace, /version = "0\.427\.99"/u);
    assert.match(
      workspace,
      /vize_s1_to_s2 = \{ path = "crates\/vize_s1_to_s2", version = "=0\.427\.99"\s*, package = "vize_l1_to_l2" \}/u,
    );
    assert.match(exact.stderr, new RegExp(base));
    assert.notEqual(run("missing-base").status, 0);
    // Once the package already exists in the base, preserve --baseline-rev.
    git(["add", "Cargo.toml", "crates"]);
    git([
      "-c",
      "user.name=Vize",
      "-c",
      "user.email=vize@example.com",
      "-c",
      "commit.gpgsign=false",
      "commit",
      "-qm",
      "renamed package",
    ]);
    const registered = run(git(["rev-parse", "HEAD"]));
    assert.equal(registered.status, 0, registered.stderr);
    assert.equal(registered.stdout, "");
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("release SemVer invokes the required check with the immutable baseline and fails closed", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-semver-baseline-"));
  const bin = path.join(root, "bin");
  const runnerTemp = path.join(root, "runner");
  const log = path.join(root, "cargo.jsonl");
  const helperLog = path.join(root, "helper.jsonl");
  fs.mkdirSync(bin);
  fs.mkdirSync(runnerTemp);
  fs.writeFileSync(path.join(runnerTemp, "semver-change-marker.txt"), "");
  writeFakeCommand(
    bin,
    "cargo",
    [
      "const fs = require('node:fs');",
      "fs.appendFileSync(process.env.SEMVER_TEST_LOG, JSON.stringify(process.argv.slice(2)) + '\\n');",
    ].join("\n"),
  );
  writeFakeCommand(
    bin,
    "rust-script",
    [
      "const fs = require('node:fs');",
      "fs.appendFileSync(process.env.SEMVER_HELPER_LOG, JSON.stringify(process.argv.slice(2)) + '\\n');",
      "if (process.env.SEMVER_TEST_FAIL) process.exit(1);",
      "if (process.env.SEMVER_TEST_ROOT) process.stdout.write(process.env.SEMVER_TEST_ROOT + '\\n');",
    ].join("\n"),
  );
  // This dispatch fixture has no merge parent; the exact-base test uses real Git.
  writeFakeCommand(bin, "git", "process.exit(1);");
  const script = path.join(repoRoot, "tools/commands/ci/github/check-semver.sh");
  const run = (extra: NodeJS.ProcessEnv = {}) => {
    fs.writeFileSync(log, "");
    fs.writeFileSync(helperLog, "");
    return spawnSync("bash", [script, "vize_l1_to_l2"], {
      cwd: root,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
        RUNNER_TEMP: runnerTemp,
        BASELINE_REV: "",
        SEMVER_TEST_LOG: log,
        SEMVER_HELPER_LOG: helperLog,
        SEMVER_TEST_ROOT: "",
        SEMVER_TEST_FAIL: "",
        ...extra,
      },
    });
  };
  const logged = () =>
    fs
      .readFileSync(log, "utf8")
      .trim()
      .split("\n")
      .filter(Boolean)
      .map((line) => JSON.parse(line));
  try {
    const baselineRoot = path.join(root, "old published API");
    const first = run({ SEMVER_TEST_ROOT: baselineRoot });
    assert.equal(first.status, 0, `${first.stderr}\n${first.stdout}`);
    assert.deepEqual(logged(), [
      [
        "semver-checks",
        "check-release",
        "--package",
        "vize_l1_to_l2",
        "--baseline-root",
        baselineRoot,
      ],
    ]);
    assert.equal(fs.readFileSync(helperLog, "utf8").trim().split("\n").length, 1);
    assert.deepEqual(fs.readdirSync(runnerTemp), ["semver-change-marker.txt"]);

    const registered = run();
    assert.equal(registered.status, 0, registered.stderr);
    assert.deepEqual(logged(), [["semver-checks", "check-release", "--package", "vize_l1_to_l2"]]);

    const failed = run({ SEMVER_TEST_FAIL: "1" });
    assert.notEqual(failed.status, 0);
    assert.deepEqual(logged(), []);
    assert.deepEqual(fs.readdirSync(runnerTemp), ["semver-change-marker.txt"]);

    const explicit = run({ BASELINE_REV: "verified-base" });
    assert.equal(explicit.status, 0, explicit.stderr);
    assert.deepEqual(logged(), [
      [
        "semver-checks",
        "check-release",
        "--package",
        "vize_l1_to_l2",
        "--baseline-rev",
        "verified-base",
      ],
    ]);
    assert.equal(JSON.parse(fs.readFileSync(helperLog, "utf8").trim()).at(-1), "verified-base");
    const renamedBase = run({ BASELINE_REV: "verified-old-base", SEMVER_TEST_ROOT: baselineRoot });
    assert.equal(renamedBase.status, 0, renamedBase.stderr);
    assert.deepEqual(logged(), [
      [
        "semver-checks",
        "check-release",
        "--package",
        "vize_l1_to_l2",
        "--baseline-root",
        baselineRoot,
      ],
    ]);
    assert.equal(JSON.parse(fs.readFileSync(helperLog, "utf8").trim()).at(-1), "verified-old-base");
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

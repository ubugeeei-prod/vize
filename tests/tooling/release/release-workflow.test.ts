import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { cadenceDecision } from "../../../tools/support/release/release/cadence.ts";
import { extraReleasePackageJsonPaths } from "../../../tools/support/release/release/preparation.ts";
import { stampCargoLock } from "../../../tools/support/release/release/stamp.ts";
import {
  isUnclaimedVersion,
  nextReleaseVersion,
  updateWorkspaceManifestVersions,
} from "../../../tools/support/release/release/versions.ts";
import { readRepoFile, root } from "../support/github-workflows.ts";

type Job = {
  needs?: string | string[];
  environment?: string;
  permissions?: Record<string, string>;
  uses?: string;
  steps?: Array<{ uses?: string; run?: string; with?: Record<string, unknown> }>;
};
const workflow = parse(readRepoFile(".github", "workflows", "release.yml")) as {
  on: { workflow_dispatch: { inputs: Record<string, unknown> } };
  env: Record<string, string>;
  jobs: Record<string, Job>;
};
const needs = (job: string) => [workflow.jobs[job]?.needs ?? []].flat();

test("the next release bumps the minor above every claimed version", () => {
  assert.equal(nextReleaseVersion(["0.474.0", "v0.471.0", "v0.474.0", "0.1.0"]), "0.475.0");
  assert.equal(nextReleaseVersion(["0.474.0", "v0.480.0", "0.0.1-alpha.106"]), "0.481.0");
  assert.equal(nextReleaseVersion(["0.474.0", "0.474.3"]), "0.475.0");
  assert.equal(nextReleaseVersion(["not-a-version"]), null);
  assert.ok(isUnclaimedVersion("0.476.0", ["0.474.0", "v0.475.0"]));
  assert.ok(!isUnclaimedVersion("0.475.0", ["0.474.0", "v0.475.0"]));
  assert.ok(!isUnclaimedVersion("v0.476.0", ["0.474.0"]));
  assert.ok(!isUnclaimedVersion("0.476.0-rc", ["0.474.0"]));
});

test("the stamp rewrites only owned Cargo.lock packages", () => {
  const lock = [
    "[[package]]",
    'name = "vize_carton"',
    'version = "0.1.0"',
    "",
    "[[package]]",
    'name = "serde"',
    'version = "0.1.0"',
  ].join("\n");
  assert.equal(
    stampCargoLock(lock, "0.1.0", "0.2.0"),
    lock.replace('version = "0.1.0"', 'version = "0.2.0"'),
  );
});

test("stamping moves every versioned manifest and leaves the native pins alone", () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-stamp-"));
  const write = (file: string, content: string) => {
    fs.mkdirSync(path.dirname(path.join(dir, file)), { recursive: true });
    fs.writeFileSync(path.join(dir, file), content);
  };
  const manifest = (name: string) => `{\n  "name": "${name}",\n  "version": "0.1.0"\n}\n`;
  try {
    write(
      "Cargo.toml",
      '[workspace.package]\nversion = "0.1.0"\n\n[workspace.dependencies]\nvize_carton = { path = "c", version = "=0.1.0" }\n',
    );
    write("Cargo.lock", '[[package]]\nname = "vize"\nversion = "0.1.0"\n');
    write("npm/cli/package.json", manifest("vize"));
    write("npm/builder/vite/package.json", manifest("@vizejs/vite-plugin"));
    write("editors/vscode/package.json", manifest("vize"));
    write("npm/cli/README.md", "npm i vize@0.1.0\n");
    write(
      "pnpm-workspace.yaml",
      'catalogs:\n  native-binaries:\n    "@vizejs/native-darwin-arm64": "0.1.0"\n',
    );
    execFileSync("git", ["init", "-q"], { cwd: dir });
    execFileSync(
      process.execPath,
      [path.join(root, "tools/support/release/release/main.ts"), "stamp", "0.2.0"],
      { cwd: dir, stdio: "pipe" },
    );
    for (const file of [
      "Cargo.toml",
      "Cargo.lock",
      "npm/cli/package.json",
      "npm/builder/vite/package.json",
      "editors/vscode/package.json",
      "npm/cli/README.md",
    ]) {
      const content = fs.readFileSync(path.join(dir, file), "utf8");
      assert.ok(content.includes("0.2.0") && !content.includes("0.1.0"), file);
    }
    assert.match(fs.readFileSync(path.join(dir, "pnpm-workspace.yaml"), "utf8"), /"0\.1\.0"/);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test("release is dispatched on main with an optional exact version", () => {
  assert.deepEqual(Object.keys(workflow.on.workflow_dispatch.inputs), ["version"]);
  const version = workflow.jobs.version;
  assert.deepEqual(version.permissions, { contents: "write", actions: "read" });
  assert.match(JSON.stringify(version.steps), /release\/release\/main\.ts cut/);
});

test("every build checks out the release tag and starts right after tagging", () => {
  for (const [name, job] of Object.entries(workflow.jobs)) {
    if (name === "version" || job.uses) continue;
    assert.ok(needs(name).includes("version"), `${name} needs version`);
    const checkout = job.steps?.find((step) => step.uses?.startsWith("actions/checkout@"));
    assert.equal(checkout?.with?.ref, "${{ needs.version.outputs.tag }}", name);
  }
  for (const build of ["build-cli", "build-native", "build-packages", "build-wasm"]) {
    assert.deepEqual(needs(build), ["version"], build);
  }
});

test("registries publish in parallel behind one credential-free smoke", () => {
  assert.equal(workflow.jobs.smoke.environment, undefined);
  assert.equal(workflow.jobs.smoke.permissions, undefined);
  const publishers = Object.keys(workflow.jobs).filter((name) => name.startsWith("publish-"));
  assert.deepEqual(publishers.sort(), [
    "publish-crates",
    "publish-jsr",
    "publish-npm",
    "publish-open-vsx",
    "publish-vscode",
  ]);
  for (const name of ["publish-crates", "publish-npm", "publish-open-vsx", "publish-vscode"]) {
    assert.ok(needs(name).includes("smoke"), name);
    assert.ok(!needs(name).some((other) => other.startsWith("publish-")), name);
  }
  const npm = JSON.stringify(workflow.jobs["publish-npm"].steps);
  assert.match(
    npm,
    /publish_npm_package_dirs\/main\.ts \\"\$\{dirs\[@\]\}\\" --concurrency 16 --provenance --no-wait/,
  );
  for (const dir of workflow.env.RELEASE_JS_PACKAGES.split(" ")) {
    assert.match(
      readRepoFile(".github", "actions", "release-build-packages", "action.yml"),
      new RegExp(`^          ${dir}$`, "m"),
    );
  }
});

test("registry visibility is checked once, after publishing, and gates only JSR", () => {
  assert.deepEqual(needs("verify-npm"), ["version", "publish-npm"]);
  assert.deepEqual(needs("publish-jsr"), ["version", "verify-npm"]);
  assert.ok(!needs("github-release").includes("verify-npm"));
  assert.equal(workflow.jobs["publish-jsr"].uses, "./.github/workflows/release-jsr.yml");
});

test("the stamp reaches every tracked non-private npm and editor manifest", async () => {
  const extra = new Set(
    (await extraReleasePackageJsonPaths()).map((file) => file.replaceAll("\\", "/")),
  );
  const tracked = execFileSync("git", ["ls-files", "-z", "--", "editors", "npm"], { cwd: root })
    .toString()
    .split("\0")
    .filter((file) => file.endsWith("/package.json"));
  for (const file of tracked) {
    const manifest = JSON.parse(fs.readFileSync(path.join(root, file), "utf8"));
    const flat = /^npm\/[^/]+\/package\.json$/.test(file);
    if (manifest.private === true) assert.ok(!extra.has(file), `${file} is private`);
    else if (!flat) assert.ok(extra.has(file), `${file} must be stamped`);
  }
});

test("the stamp rewrites aliased fork pins and keeps upstream versions", () => {
  const input =
    '[workspace.package]\nversion = "0.430.0"\n[workspace.dependencies]\noxc_parser = { package = "vize_oxc_parser", version = "=0.430.0", path = "vendor/oxc_parser" }\noxc_ast = { version = "=0.142.0" }\n';
  assert.equal(
    updateWorkspaceManifestVersions(input, "0.430.0", "0.430.1"),
    input.replaceAll("0.430.0", "0.430.1"),
  );
});

test("Open VSX publication tolerates ovsx dependency build scripts", () => {
  const job = (workflow.jobs["publish-open-vsx"] as Job & { env?: Record<string, string> }).env;
  assert.equal(job?.PNPM_CONFIG_STRICT_DEP_BUILDS, "false");
});

test("the hourly cadence releases a green main commit only after user-facing changes", () => {
  const released = "a".repeat(40);
  const green = "b".repeat(40);
  const decide = (subjects: string[], greenSha: string | null = green) =>
    cadenceDecision({ greenSha, releasedSha: released, subjects }).release;
  assert.ok(decide(["fix(patina): x", "chore: y"]));
  assert.ok(decide(["feat!: x"]) && decide(["perf(lsp): x"]));
  assert.ok(!decide(["docs: x", "ci(release): y", "test(jsx): z", "refactor: w"]));
  assert.ok(!decide(["fix: x"], null), "no green Check");
  assert.ok(!decide(["fix: x"], released), "already released");
  const on = workflow.on as unknown as { schedule?: Array<{ cron: string }> };
  assert.equal(on.schedule?.length, 1);
  for (const build of ["build-cli", "build-native", "build-packages", "build-wasm"]) {
    assert.equal(
      (workflow.jobs[build] as Job & { if?: string }).if,
      "needs.version.outputs.tag != ''",
    );
  }
});

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { cadenceDecision } from "../../../tools/support/release/release/cadence.ts";
import { incompletePriorReleaseTargets } from "../../../tools/support/release/release/main.ts";
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
  "timeout-minutes"?: number;
  environment?: string;
  env?: Record<string, string>;
  permissions?: Record<string, string>;
  uses?: string;
  steps?: Array<{ id?: string; uses?: string; run?: string; with?: Record<string, unknown> }>;
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

test("a new release requires the prior tag on npm and every crates.io target", async () => {
  const complete = await incompletePriorReleaseTargets("0.510.0", ["0.510.0"], async () => ({
    kind: "published",
  }));
  assert.deepEqual(complete, []);

  let calls = 0;
  const incomplete = await incompletePriorReleaseTargets("0.510.0", [], async (crate) => {
    calls += 1;
    if (crate === "vize_oxc_parser") return { kind: "missing" };
    if (crate === "vize_patina") return { kind: "query-failed", message: "HTTP 429" };
    return { kind: "published" };
  });
  assert.equal(calls, 28);
  assert.deepEqual(incomplete, [
    "npm:vize@0.510.0",
    "crates:vize_oxc_parser@0.510.0",
    "crates:vize_patina@0.510.0",
  ]);
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

test("release is dispatched on main with an optional exact version or npm resume run", () => {
  // Dispatch only: crates.io trusted publishing refuses workflow_run and schedule.
  assert.deepEqual(Object.keys(workflow.on), ["workflow_dispatch"]);
  assert.deepEqual(Object.keys(workflow.on.workflow_dispatch.inputs), [
    "version",
    "source",
    "npm_resume_run",
  ]);
  const version = workflow.jobs.version;
  assert.deepEqual(version.permissions, { actions: "read", contents: "write" });
  assert.match(JSON.stringify(version.steps), /release\/release\/main\.ts cut/);
  assert.match(JSON.stringify(version.steps), /release\/npm_resume\/main\.ts/);
});

test("npm resume preserves the trusted workflow and restores only the selected run artifacts", () => {
  const npm = workflow.jobs["publish-npm"];
  const release = workflow.jobs["github-release"];
  assert.equal(npm.environment, "npm");
  assert.deepEqual(npm.permissions, { actions: "read", contents: "read", "id-token": "write" });
  assert.match(String((npm as Job & { if?: string }).if), /inputs\.npm_resume_run/);
  assert.match(String((release as Job & { if?: string }).if), /inputs\.npm_resume_run/);
  assert.match(JSON.stringify(npm.steps), /"run-id":"\$\{\{ inputs\.npm_resume_run \}\}"/);
  assert.match(JSON.stringify(release.steps), /"run-id":"\$\{\{ inputs\.npm_resume_run \}\}"/);
  assert.equal(npm.env?.NPM_PUBLISH_TAG, "${{ needs.version.outputs.publish_tag }}");
  assert.equal(npm.env?.NPM_ALLOW_NEWER_TAG, "${{ needs.version.outputs.allow_newer_tag }}");

  const restore = readRepoFile(".github", "actions", "release-npm-artifacts", "action.yml");
  assert.match(
    JSON.stringify(npm.steps),
    /"release-tag":"\$\{\{ inputs\.npm_resume_run != '' && needs\.version\.outputs\.tag \|\| '' \}\}"/,
  );
  assert.match(restore, /git fetch --no-tags --depth=1 origin "refs\/tags\/\$RELEASE_TAG"/);
  assert.match(
    restore,
    /for file in pnpm-workspace\.yaml npm\/native\/package\.json npm\/fresco-native\/package\.json/,
  );
  assert.match(restore, /git show "FETCH_HEAD:\$file" > "\$file"/);
});

test("builds use the untagged candidate; only a fully built release is tagged", () => {
  const prePublish = (name: string) =>
    name.startsWith("build-") || name === "smoke" || name === "tag";
  for (const [name, job] of Object.entries(workflow.jobs)) {
    // publish-jsr only dispatches release-jsr.yml and checks nothing out.
    if (name === "version" || name === "publish-jsr") continue;
    assert.ok(needs(name).includes("version"), `${name} needs version`);
    const checkout = job.steps?.find((step) => step.uses?.startsWith("actions/checkout@"));
    const ref = prePublish(name) ? "ref" : "tag";
    const expected =
      name === "publish-npm"
        ? "${{ inputs.npm_resume_run != '' && github.sha || needs.version.outputs.tag }}"
        : `${"${{"} needs.version.outputs.${ref} }}`;
    assert.equal(checkout?.with?.ref, expected, name);
  }
  for (const gate of ["smoke", "build-cli", "build-editor-extensions"]) {
    assert.ok(needs("tag").includes(gate), `tag waits for ${gate}`);
  }
  assert.match(JSON.stringify(workflow.jobs.tag.steps), /refs\/tags\/\$RELEASE_TAG/);
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
  for (const build of ["build-packages", "build-native", "build-wasm"]) {
    assert.ok(needs("smoke").includes(build), `smoke waits for ${build}`);
  }
  for (const name of ["publish-crates", "publish-npm", "publish-open-vsx", "publish-vscode"]) {
    assert.ok(needs(name).includes("tag"), name);
    assert.ok(!needs(name).some((other) => other.startsWith("publish-")), name);
  }
  for (const dir of workflow.env.RELEASE_JS_PACKAGES.split(" ")) {
    assert.match(
      readRepoFile(".github", "actions", "release-build-packages", "action.yml"),
      new RegExp(`^          ${dir}$`, "m"),
    );
  }
});

test("npm exposes every native platform before packages that depend on it", () => {
  const job = workflow.jobs["publish-npm"];
  const run = job.steps?.find((step) => step.id === "publish")?.run ?? "";
  const platformPublish =
    'node tools/support/release/publish_npm_package_dirs/main.ts "${platforms[@]}" --concurrency 8 --provenance';
  const nativePublish =
    'node tools/support/release/publish_npm_package_dirs/main.ts "${native[@]}" --concurrency 1 --provenance';
  const dependentPublish =
    'node tools/support/release/publish_npm_package_dirs/main.ts "${dependents[@]}" --concurrency 16 --provenance --no-wait';
  const platformLine = run
    .split("\n")
    .find((line) => line.includes(platformPublish))
    ?.trim();
  const nativeLine = run
    .split("\n")
    .find((line) => line.includes(nativePublish))
    ?.trim();

  assert.match(run, /^platforms=\(npm\/native\/npm\)$/m);
  assert.match(run, /^native=\(npm\/native\)$/m);
  assert.match(run, /^dependents=\(npm\/fresco-native npm\/wasm "\$\{js\[@\]\}"\)$/m);
  assert.equal(platformLine, platformPublish, "waits for every platform package to become visible");
  assert.equal(nativeLine, nativePublish, "waits for the native wrapper to become visible");
  assert.ok(
    run.indexOf(nativePublish) > run.indexOf(platformPublish),
    "publishes the native wrapper only after the platform visibility barrier",
  );
  assert.ok(
    run.indexOf(dependentPublish) > run.indexOf(nativePublish),
    "publishes vize and other dependents only after the native wrapper visibility barrier",
  );
  assert.match(
    run,
    /--names "\$\{platforms\[@\]\}" "\$\{native\[@\]\}" "\$\{dependents\[@\]\}"/,
    "final verification retains every package name",
  );
  assert.ok(
    Number(job.env?.PUBLISH_RESOLUTION_RETRY_LIMIT) *
      Number(job.env?.PUBLISH_RESOLUTION_RETRY_DELAY) >=
      30 * 60,
    "the platform barrier tolerates npm propagation for 30 minutes",
  );
  assert.ok((job["timeout-minutes"] ?? 0) >= 55, "the job can outlive both visibility barriers");
});

test("final registry verification checks every package and gates only JSR", () => {
  assert.deepEqual(needs("verify-npm"), ["version", "publish-npm"]);
  assert.deepEqual(needs("publish-jsr"), ["version", "verify-npm"]);
  assert.match(String((workflow.jobs["verify-npm"] as Job & { if?: string }).if), /always\(\)/);
  assert.match(
    String((workflow.jobs["verify-npm"] as Job & { if?: string }).if),
    /publish-npm\.result == 'success'/,
  );
  assert.match(
    String((workflow.jobs["publish-jsr"] as Job & { if?: string }).if),
    /verify-npm\.result == 'success'/,
  );
  assert.ok(!needs("github-release").includes("verify-npm"));
  // JSR runs as its own workflow so its slow server-side processing never
  // holds this run (and with it the cadence) open.
  const jsr = workflow.jobs["publish-jsr"];
  assert.equal(jsr.uses, undefined);
  assert.equal(jsr.environment, "release-operator");
  assert.match(JSON.stringify(jsr.steps), /gh workflow run release-jsr\.yml/);
  assert.match(JSON.stringify(jsr.steps), /secrets\.RELEASE_OPERATOR_TOKEN/);
});

test("post-publish product checks follow npm verification and gate nothing", () => {
  for (const check of ["verify-config", "verify-musea-vrt"]) {
    assert.deepEqual(needs(check), ["version", "verify-npm"]);
    assert.match(
      String((workflow.jobs[check] as Job & { if?: string }).if),
      /verify-npm\.result == 'success'/,
    );
    for (const name of Object.keys(workflow.jobs)) assert.ok(!needs(name).includes(check), name);
  }
  assert.match(
    JSON.stringify(workflow.jobs["verify-musea-vrt"].steps),
    /verify_published\/musea_vrt\.ts/,
  );
  assert.match(
    JSON.stringify(workflow.jobs["verify-config"].steps),
    /verify_published\/config_free\.ts/,
  );
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
  assert.ok(Number(job?.PUBLISH_RESOLUTION_RETRY_LIMIT) * 10 >= 600, "waits 10 min for the scan");
});

test("the automatic cadence releases a green main commit only after user-facing changes", () => {
  const released = "a".repeat(40);
  const green = "b".repeat(40);
  const decide = (subjects: string[], greenSha: string | null = green) =>
    cadenceDecision({ greenSha, releasedSha: released, subjects }).release;
  assert.ok(decide(["fix(patina): x", "chore: y"]));
  assert.ok(decide(["feat!: x"]) && decide(["perf(lsp): x"]));
  assert.ok(!decide(["docs: x", "ci(release): y", "test(jsx): z", "refactor: w"]));
  assert.ok(!decide(["fix: x"], null), "no green Check");
  assert.ok(!decide(["fix: x"], released), "already released");
  assert.ok(
    !cadenceDecision({
      greenSha: green,
      releasedSha: released,
      subjects: ["fix: x"],
      attempted: true,
    }).release,
    "a failed candidate of the same commit is not retried hourly",
  );
  assert.ok(
    !cadenceDecision({
      greenSha: green,
      releasedSha: released,
      subjects: ["fix: x"],
      running: true,
    }).release,
    "no second release while one is still building",
  );
  const minute = 60 * 1000;
  const spaced = (releasedAt: number) =>
    cadenceDecision({
      greenSha: green,
      releasedSha: released,
      subjects: ["fix: x"],
      releasedAt,
      now: 600 * minute,
    }).release;
  assert.ok(spaced(589 * minute) && !spaced(595 * minute), "at most one release per 10 minutes");
  const cadence = parse(readRepoFile(".github", "workflows", "release-cadence.yml")) as {
    on: {
      schedule?: Array<{ cron: string }>;
      workflow_run?: { workflows: string[]; types: string[]; branches: string[] };
    };
    jobs: Record<string, Job>;
  };
  const on = cadence.on;
  assert.match(JSON.stringify(cadence.jobs.dispatch.steps), /gh workflow run release\.yml/);
  // JSR rejects OIDC publishes by github-actions[bot]; dispatch as the maintainer.
  assert.match(JSON.stringify(cadence.jobs.dispatch.steps), /secrets\.RELEASE_OPERATOR_TOKEN/);
  assert.equal(on.schedule?.length, 1);
  // A Release completion re-decides, so fixes that land mid-release ship next.
  assert.deepEqual(on.workflow_run, {
    workflows: ["Check", "Release"],
    types: ["completed"],
    branches: ["main"],
  });
  for (const build of ["build-cli", "build-native", "build-packages", "build-wasm"]) {
    assert.equal(
      (workflow.jobs[build] as Job & { if?: string }).if,
      "needs.version.outputs.ref != ''",
    );
  }
});

test("release builds embed the exact planned version across trusted target caches", () => {
  assert.equal(workflow.env.VIZE_RELEASE_BUILD, "1");
  for (const build of ["build-cli", "build-native"]) {
    const recipe = workflow.jobs[build].steps?.find((step) =>
      step.uses?.startsWith("./.github/actions/build-release-"),
    );
    assert.equal(recipe?.with?.["release-version"], "${{ needs.version.outputs.version }}");
  }
  assert.match(
    readRepoFile("crates", "vize_carton", "src", "lib.rs"),
    /option_env!\("VIZE_RELEASE_VERSION"\)[\s\S]*option_env!\("VIZE_RELEASE_BUILD"\)[\s\S]*concat!\(env!\("CARGO_PKG_VERSION"\), "-dev"\)/,
  );
});

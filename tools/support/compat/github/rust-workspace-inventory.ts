import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";

export const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const sourceFiles = [
  "Cargo.toml",
  "Cargo.lock",
  "rust-toolchain.toml",
  ".cargo/config.toml",
  ".config/nextest.toml",
];
const runtimeKeys = [
  "VIZE_TEST_REQUIRE_TSGO",
  "VIZE_TEST_DISABLE_TSGO",
  "VIZE_NUXT_CONFIG_ITERATIONS",
  "RUST_TEST_THREADS",
  "CARGO_BUILD_JOBS",
  "RUSTFLAGS",
  "RUSTDOCFLAGS",
  "CARGO_ENCODED_RUSTFLAGS",
  "CARGO_TARGET_DIR",
];
const git = (...args) => execFileSync("git", args, { encoding: "utf8" }).trim();

export function inferWorkspaceProfileDefaults(manifest, cargoConfig, env = {}) {
  // Only the execution runner needs TOML; the aggregate intentionally runs
  // without a dependency install and only validates the retained receipts.
  const TOML = createRequire(import.meta.url)("@iarna/toml");
  const profiles = TOML.parse(manifest).profile ?? {};
  const config = TOML.parse(cargoConfig);
  assert.equal(profiles.ci?.inherits, "dev", "ci must inherit dev assertions");
  for (const name of ["ci", "dev", "test"]) {
    const profile = profiles[name] ?? {};
    for (const flag of ["opt-level", "debug-assertions", "overflow-checks"])
      assert.ok(!Object.hasOwn(profile, flag), `${name} overrides ${flag}`);
    assert.ok(!Object.hasOwn(profile, "build-override"), `${name} has build overrides`);
    if (Object.hasOwn(profile, "package")) {
      assert.equal(name, "ci", `${name} has package overrides`);
      assert.deepEqual(
        profile.package,
        {
          "sha2:0.11.1": {
            "opt-level": 3,
            "debug-assertions": true,
            "overflow-checks": true,
          },
        },
        "ci package overrides differ from the reviewed native image hash optimization",
      );
    }
    assert.ok(!config.profile?.[name], `Cargo config overrides ${name}`);
  }
  assert.ok(
    !Object.keys(env).some((key) => /^CARGO_PROFILE_(?:CI|DEV|TEST)_/.test(key)),
    "profile environment overrides require separate review",
  );
  const configuredFlags = [config.build, ...Object.values(config.target ?? {})]
    .flatMap((section) => [section?.rustflags, section?.rustdocflags])
    .flat();
  const flags = [env.RUSTFLAGS, env.RUSTDOCFLAGS, env.CARGO_ENCODED_RUSTFLAGS, ...configuredFlags]
    .filter(Boolean)
    .join(" ");
  assert.ok(
    !/(?:debug-assertions|overflow-checks)=(?:off|false|no|0)\b/.test(flags),
    "runtime flags disable test checks",
  );
  return {
    basis: "committed-manifest-config-and-runtime-environment",
    debugAssertions: true,
    overflowChecks: true,
  };
}

export function workspaceFingerprint(env = process.env) {
  assert.equal(
    git("status", "--porcelain", "--untracked-files=no"),
    "",
    "workspace inventory needs clean committed source",
  );
  const sha = git("rev-parse", "HEAD");
  if (env.GITHUB_SHA) assert.equal(sha, env.GITHUB_SHA, "workspace checkout/source SHA mismatch");
  const files = Object.fromEntries(sourceFiles.map((file) => [file, digest(readFileSync(file))]));
  const manifest = readFileSync("Cargo.toml", "utf8");
  const inferredProfileDefaults = inferWorkspaceProfileDefaults(
    manifest,
    readFileSync(".cargo/config.toml", "utf8"),
    env,
  );
  const channel = readFileSync("rust-toolchain.toml", "utf8").match(
    /^channel\s*=\s*"([^"]+)"/m,
  )?.[1];
  const rustc = execFileSync("rustc", ["-Vv"], { encoding: "utf8" }).trim();
  assert.ok(
    channel && rustc.startsWith(`rustc ${channel} `),
    "workspace compiler differs from normal committed toolchain",
  );
  assert.equal(env.VIZE_TEST_REQUIRE_TSGO, "1");
  assert.equal(env.VIZE_TEST_DISABLE_TSGO, undefined);
  assert.equal(env.VIZE_NUXT_CONFIG_ITERATIONS, "100");
  return {
    sha,
    tree: git("rev-parse", "HEAD^{tree}"),
    files,
    rustc,
    cargo: execFileSync("cargo", ["-Vv"], { encoding: "utf8" }).trim(),
    platform: process.platform,
    arch: process.arch,
    workspaceRoot: process.cwd(),
    runtime: Object.fromEntries(runtimeKeys.map((key) => [key, env[key] ?? null])),
    inferredProfileDefaults,
  };
}

export function normalizeWorkspaceInventory(list) {
  const suites = list["rust-suites"];
  assert.ok(suites && typeof suites === "object" && !Array.isArray(suites));
  const binaries = Object.entries(suites)
    .map(([id, suite]) => {
      assert.equal(suite["binary-id"], id);
      assert.equal(suite.status, "listed", `full inventory omits binary ${id}`);
      assert.equal(typeof suite["package-name"], "string");
      assert.equal(typeof suite["binary-name"], "string");
      assert.ok(suite.testcases && typeof suite.testcases === "object");
      const cases = Object.entries(suite.testcases)
        .map(([name, value]) => {
          assert.equal(
            value["filter-match"]?.status,
            "matches",
            `full inventory excludes ${id}/${name}`,
          );
          assert.equal(typeof value.ignored, "boolean");
          return { name, ignored: value.ignored };
        })
        .sort((a, b) => a.name.localeCompare(b.name, "en"));
      return { id, package: suite["package-name"], binary: suite["binary-name"], cases };
    })
    .sort((a, b) => a.id.localeCompare(b.id, "en"));
  assert.ok(
    binaries.some((binary) => binary.cases.length > 0),
    "empty workspace inventory",
  );
  return binaries;
}

export function readWorkspaceInventory(file) {
  const receipt = JSON.parse(readFileSync(file, "utf8"));
  assert.equal(receipt.schema, "vize.rust.workspace-inventory");
  assert.equal(receipt.schemaVersion, 1);
  assert.ok(["test", "ci"].includes(receipt.cargoProfile));
  assert.equal(path.basename(receipt.rawFile), receipt.rawFile);
  const raw = readFileSync(path.join(path.dirname(file), receipt.rawFile));
  assert.equal(digest(raw), receipt.rawSha256, "raw workspace inventory digest mismatch");
  assert.deepEqual(
    normalizeWorkspaceInventory(JSON.parse(raw)),
    receipt.binaries,
    "raw workspace inventory differs from receipt",
  );
  return receipt;
}

export function compareWorkspaceInventories(before, after) {
  assert.equal(before.cargoProfile, "test");
  assert.equal(after.cargoProfile, "ci");
  assert.deepEqual(after.source, before.source, "inventory source/compiler/runtime mismatch");
  assert.deepEqual(
    after.binaries,
    before.binaries,
    "Cargo profile changes workspace binaries/case identities",
  );
}

export function requireFullWorkspaceJobs(needs) {
  assert.ok(needs && typeof needs === "object" && !Array.isArray(needs));
  assert.deepEqual(
    Object.keys(needs).sort(),
    ["clippy-and-test", "full-rust-workspace"],
    "missing full Rust qualification job",
  );
  for (const [name, job] of Object.entries(needs))
    assert.equal(job?.result, "success", `${name} did not succeed`);
}

function attributes(text) {
  const decode = (value) =>
    value.replace(/&(?:quot|apos|lt|gt|amp|#\d+|#x[\da-f]+);/gi, (entity) => {
      const named = { "&quot;": '"', "&apos;": "'", "&lt;": "<", "&gt;": ">", "&amp;": "&" };
      return (
        named[entity] ??
        String.fromCodePoint(
          entity.startsWith("&#x")
            ? parseInt(entity.slice(3, -1), 16)
            : Number(entity.slice(2, -1)),
        )
      );
    });
  return Object.fromEntries(
    [...text.matchAll(/([\w-]+)="([^"]*)"/g)].map((match) => [match[1], decode(match[2])]),
  );
}

export function verifyWorkspaceExecution(receipt, timing, junit, expectedSha, cargoProfile = "ci") {
  assert.equal(receipt.source.sha, expectedSha, "workspace result is not this source head");
  assert.deepEqual(timing.source, receipt.source, "execution/inventory source or runtime mismatch");
  assert.deepEqual(receipt.source.inferredProfileDefaults, {
    basis: "committed-manifest-config-and-runtime-environment",
    debugAssertions: true,
    overflowChecks: true,
  });
  assert.equal(receipt.source.runtime.VIZE_TEST_REQUIRE_TSGO, "1");
  assert.equal(receipt.source.runtime.VIZE_TEST_DISABLE_TSGO, null);
  assert.equal(receipt.source.runtime.VIZE_NUXT_CONFIG_ITERATIONS, "100");
  assert.equal(receipt.cargoProfile, cargoProfile);
  assert.equal(timing.sourceSha, expectedSha);
  assert.equal(timing.cargoProfile, cargoProfile);
  assert.equal(timing.runner, "nextest");
  assert.equal(timing.inventoryOnly, false);
  assert.equal(timing.status, "passed");
  assert.deepEqual(
    timing.phases.map((phase) => [phase.name, phase.exitCode]),
    [
      ["inventory", 0],
      ["workspace", 0],
      ["doctests", 0],
    ],
  );
  assert.deepEqual(
    timing.phases.map((phase) => phase.args),
    [
      [
        "nextest",
        "list",
        "--locked",
        "--workspace",
        "--cargo-profile",
        cargoProfile,
        "--profile",
        "full",
        "--run-ignored",
        "all",
        "--message-format",
        "json",
      ],
      [
        "nextest",
        "run",
        "--locked",
        "--workspace",
        "--cargo-profile",
        cargoProfile,
        "--profile",
        "full",
      ],
      ["test", "--locked", "--workspace", "--profile", cargoProfile, "--doc"],
    ],
  );
  const expected = receipt.binaries
    .flatMap((binary) =>
      binary.cases
        .filter((test) => !test.ignored)
        .map((test) => JSON.stringify([binary.id, test.name])),
    )
    .sort();
  // Native observations can contain XML-looking original source in CDATA.
  const structure = junit.replace(/<!\[CDATA\[[\s\S]*?\]\]>/g, "").replace(/<!--[\s\S]*?-->/g, "");
  const actual = [...structure.matchAll(/<testcase\b([^>]*?)(?:\/?>)/g)]
    .map((match) => {
      const test = attributes(match[1]);
      assert.ok(test.classname && test.name, "JUnit case identity is incomplete");
      return JSON.stringify([test.classname, test.name]);
    })
    .sort();
  assert.ok(!/<(?:failure|error)\b/.test(structure), "workspace JUnit contains failed cases");
  assert.ok(!/<skipped\b/.test(structure), "workspace JUnit skipped a required case");
  assert.deepEqual(
    actual,
    expected,
    "executed workspace case identities differ from complete inventory",
  );
}

function executionReceipt(file) {
  const timing = JSON.parse(readFileSync(file, "utf8"));
  assert.match(timing.execution.runId, /^\d+$/);
  assert.match(timing.execution.attempt, /^\d+$/);
  if (process.env.GITHUB_RUN_ID)
    assert.equal(
      timing.execution.runId,
      process.env.GITHUB_RUN_ID,
      "execution is from another workflow run",
    );
  if (process.env.GITHUB_RUN_ATTEMPT)
    assert.equal(
      timing.execution.attempt,
      process.env.GITHUB_RUN_ATTEMPT,
      "execution is from another workflow attempt",
    );
  return timing;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [mode, first, second] = process.argv.slice(2);
  if (mode === "compare" && first && second) {
    const before = readWorkspaceInventory(first);
    const after = readWorkspaceInventory(second);
    compareWorkspaceInventories(before, after);
    let originalExecution;
    for (const [file, receipt, timingFile] of [
      [first, before, "workspace-reference-timing.json"],
      [second, after, "workspace-timing.json"],
    ]) {
      const timing = executionReceipt(path.join(path.dirname(file), timingFile));
      if (originalExecution) assert.deepEqual(timing.execution, originalExecution);
      else originalExecution = timing.execution;
      verifyWorkspaceExecution(
        receipt,
        timing,
        readFileSync(
          path.join(path.dirname(file), `workspace-junit-${receipt.cargoProfile}.xml`),
          "utf8",
        ),
        after.source.sha,
        receipt.cargoProfile,
      );
    }
    console.log("Complete same-source test/ci workspace inventories and executions match.");
  } else if (mode === "verify" && first && second) {
    requireFullWorkspaceJobs(JSON.parse(process.env.NEEDS_JSON ?? "null"));
    const receipt = readWorkspaceInventory(path.join(second, "workspace-inventory-ci.json"));
    assert.equal(git("rev-parse", "HEAD"), first);
    assert.equal(receipt.source.tree, git("rev-parse", "HEAD^{tree}"));
    assert.deepEqual(Object.keys(receipt.source.files).sort(), [...sourceFiles].sort());
    for (const [file, sha] of Object.entries(receipt.source.files))
      assert.equal(digest(readFileSync(file)), sha);
    const channel = readFileSync("rust-toolchain.toml", "utf8").match(
      /^channel\s*=\s*"([^"]+)"/m,
    )?.[1];
    assert.ok(receipt.source.rustc.startsWith(`rustc ${channel} `));
    verifyWorkspaceExecution(
      receipt,
      executionReceipt(path.join(second, "workspace-timing.json")),
      readFileSync(path.join(second, "workspace-junit-ci.xml"), "utf8"),
      first,
    );
    console.log("Complete same-source ci workspace inventory and execution verified.");
  } else {
    throw new Error(
      "usage: rust-workspace-inventory.ts compare TEST_RECEIPT CI_RECEIPT | verify SHA RESULTS_DIRECTORY",
    );
  }
}

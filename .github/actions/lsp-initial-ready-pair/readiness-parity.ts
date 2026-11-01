import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { SHIPPING_RECIPE } from "./shipping-adapter.ts";
type Obj = Record<string, unknown>;
const object = (value: unknown): Obj => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Obj;
};
const text = (value: unknown): string => {
  assert.equal(typeof value, "string");
  return value as string;
};
const hash = (bytes: Buffer): string => createHash("sha256").update(bytes).digest("hex");
export const BOUND_PATHS = [
  "crates/vize_maestro/src/server/diagnostic_publishing.rs",
  "crates/vize_maestro/src/server/diagnostic_publishing/complete.rs",
  "crates/vize_maestro/src/server/diagnostic_publishing/sync_feedback.rs",

  "crates/vize_maestro/src/ide/definition/service.rs",
  "crates/vize_maestro/src/ide/definition/service/private_setup_tests.rs",
  "crates/vize_maestro/src/ide/definition/service/private_setup_tests/original.rs",
  "crates/vize_maestro/src/ide/references.rs",
  "crates/vize_maestro/src/ide/references/canonical.rs",
  "crates/vize_maestro/src/ide/references/canonical/local_binding.rs",
  "crates/vize_maestro/src/ide/references/structural.rs",
  "crates/vize_maestro/src/ide/references/structural/snapshot.rs",
  "crates/vize_maestro/src/ide/references/structural/guard.rs",
  "crates/vize_maestro/src/ide/references/structural/template.rs",
  "crates/vize_maestro/src/ide/references/structural/edits.rs",
  "crates/vize_maestro/src/ide/references/structural/style.rs",
  "crates/vize_maestro/src/ide/references/structural/snapshot/registry.rs",
  "crates/vize_maestro/src/document/store/structural_navigation.rs",
  "crates/vize_maestro/src/ide/context.rs",
  "crates/vize_maestro/src/ide/definition/service/locations.rs",
  "crates/vize_maestro/src/ide/definition/service/import_target.rs",
  "crates/vize_maestro/src/ide/definition/service/import_target/alias.rs",
  "crates/vize_maestro/src/ide/definition/service/template_props.rs",
  "crates/vize_maestro/src/ide/corsa_support/rename_scope.rs",
  "tests/_fixtures/performance/lsp-dual-server/sources/App.v2.vue.txt",
  "npm/cli/content-mapper-helpers.d.cts",
  "npm/cli/content-mapper-helpers.d.mts",
  "npm/cli/content-mapper-helpers.d.ts",
  "npm/cli/generated/content-mapper-types.d.ts",
  "npm/cli/virtual-ts-globals.d.ts",
  "npm/cli/virtual-ts-patterns.d.ts",
  "npm/cli/virtual-ts-slots.d.ts",
  "npm/cli/virtual-ts.d.cts",
  "npm/cli/virtual-ts.d.mts",
  "npm/cli/virtual-ts.d.ts",
  "crates/vize_maestro/Cargo.toml",
  "crates/vize_canon/Cargo.toml",
  "Cargo.toml",
  "Cargo.lock",
  "rust-toolchain.toml",
  "package.json",
  "pnpm-lock.yaml",
  "pnpm-workspace.yaml",
  "npm/cli/package.json",
  "npm/cli/scripts/generate-content-mapper-types.ts",
  "crates/vize_maestro/src/test_support.rs",
  "tools/support/compat/github/full-rust-runtime.ts",
  "crates/vize_maestro/src/server/state/project_contexts.rs",
  "crates/vize_maestro/src/server/state/project_contexts/registry.rs",
  "crates/vize_maestro/src/server/state/corsa.rs",
  "crates/vize_maestro/src/ide/diagnostics.rs",
] as const;
export const SOURCE_PINS: Record<string, Record<string, string | null>> = {
  "36d998e36e379cc8c12842bfbc42f56e9ae0473a": {
    ".github/actions/lsp-overlay-custody/source-cli.ts":
      "d180eb1c8ec1c0aeddacdd03437e1795faa5f1aee3d465a7632157c43a7c6562",
    "crates/vize_canon/src/corsa_bridge/bridge/initial_diagnostics.rs":
      "1d094ea7f48f0aa106b0788353d09a8f3591b7dd20b699aaf4b4552de348086b",
    "crates/vize_canon/src/corsa_bridge/bridge/initial_diagnostics/tests.rs":
      "b1ef64de3bc4bcc0050f0b2668cd4fa45a9f0455b5d6b926c84f5ee8bb81d417",
    "crates/vize_canon/src/corsa_bridge/bridge/initial_diagnostics/tests/owned_read.rs": null,
    "crates/vize_maestro/src/document/store.rs":
      "5165b535053ef3034a3ae8a8e622072a82cc29c38f2ffb67473bddebc1d65098",
    "crates/vize_maestro/src/document/store/closed.rs": null,
    "crates/vize_maestro/src/document/store/closed/tests.rs": null,
    "crates/vize_maestro/src/document/store/uri_snapshot.rs":
      "8756e7267b10fec264aa18dc4dfde71b0a65e87d8dc007961b66fd5d069918b3",
    "crates/vize_maestro/src/ide/diagnostics/corsa/initial.rs":
      "72cc4f58342c5d0a7e3a465463ff880ac2a03aab15b311ecb2aa1a2b4dee7bea",
    "crates/vize_maestro/src/server/open_document.rs":
      "140ed636f8b706fe7b290f6e99dff002a4b4d85518aea396bf18abe615798928",
    "crates/vize_maestro/src/server/state/config.rs":
      "783e66a96ea4055bbc7a1244ddf40daf37a61bbd3d7159159d687bbbbdc89432",
    "crates/vize_maestro/src/server/state/features.rs":
      "2a0ce079fe4ce628d29735744045f7ff2346545a3bf75d2108527e6f25f422f9",
    "crates/vize_maestro/src/server/state/project_contexts.rs":
      "1d3c51385aebffc88de76e7b3b53169b6904adf2f2b83635084a8238359cef2c",
    "crates/vize_maestro/src/server/state/project_contexts/closed.rs": null,
    "crates/vize_maestro/src/server/state/project_contexts/closed/tests.rs": null,
    "crates/vize_maestro/src/server/state/project_contexts/registry.rs":
      "2f7fcd5ae2f7f27129e4feeeca26ba19061c0ed0b2d8324df86c3c26b5f3484d",
    "crates/vize_maestro/src/server/state/project_contexts/registry/closed_tests.rs": null,
    "crates/vize_maestro/src/server/state/project_contexts/shutdown.rs":
      "c77971db5fa5c84313529212cbffd1fce0399b09c9fe26624081c686e0a70e7e",
    "tests/tooling/lsp-overlay-owned-read-roster.test.ts": null,
  },
  "57149458f8da9d6636df1ad431ca44616e287e99": {
    ".github/actions/lsp-overlay-custody/source-cli.ts":
      "6f1a3f407d68a077724e26cfcac9b005b0c4e28abc3e9e8d2f08606aba2155b8",
    "crates/vize_canon/src/corsa_bridge/bridge/initial_diagnostics.rs":
      "d167be3b74a397a399bbfcd9594c7bd1484a0dc2b57d04db286f80081507b024",
    "crates/vize_canon/src/corsa_bridge/bridge/initial_diagnostics/tests.rs":
      "785654ce9e0276c72d9d3e3b19cd9be4f8ef313ead15a3be4174a0f9f878afab",
    "crates/vize_canon/src/corsa_bridge/bridge/initial_diagnostics/tests/owned_read.rs":
      "e4b38aacadb0f7dfca182a2290bcaab7be78313afdbe0d6f227a9060b13e039f",
    "crates/vize_maestro/src/document/store.rs":
      "6d23fcf33595a3b0d5e028ca47e1604c31a5ca23bb2e6f6458eef255dbf3e678",
    "crates/vize_maestro/src/document/store/closed.rs":
      "8684054a821369594257b96c65b3b6f8ed1671b23b32df2730365e4a748481f8",
    "crates/vize_maestro/src/document/store/closed/tests.rs":
      "9aa3c728f31e7d1d1ec5c5fde3fb9bdb34f7ba9e163817373d93e026ced13248",
    "crates/vize_maestro/src/document/store/uri_snapshot.rs":
      "675e72cca349cce9db064964e8d1ba45281d4752878197f7c9f791873c2e0450",
    "crates/vize_maestro/src/ide/diagnostics/corsa/initial.rs":
      "abaafa15a15ea477d6f1453f87312dd43f5e2d687564ca37a9c6329bc541df53",
    "crates/vize_maestro/src/server/open_document.rs":
      "18a2b7dc7f4e208d20c90f20df5bc8703cbb007b40a731fc1346cfcc95c33553",
    "crates/vize_maestro/src/server/state/config.rs":
      "d57b27304ad75bb2989282abc8841476fe9261639862d8bc6e2f47f2e78c8a22",
    "crates/vize_maestro/src/server/state/features.rs":
      "9dc7e2c8fcd4bc7a3446b336691188d7343031529a3fce080d286ccf6e643578",
    "crates/vize_maestro/src/server/state/project_contexts.rs":
      "6445287fb82fbe41eb5e7aced2ff4afda8bc22b939a2ac3f51b6301ec4d6d7ff",
    "crates/vize_maestro/src/server/state/project_contexts/closed.rs":
      "6401414858d4d7e6e51070b61c51b9809174ecf300483fef125118e997446d6b",
    "crates/vize_maestro/src/server/state/project_contexts/closed/tests.rs":
      "6a0fb9066cc8b76dc56f5eccc239afbb06d3bbee15da6a0f3c4a2122daf44a40",
    "crates/vize_maestro/src/server/state/project_contexts/registry.rs":
      "32360517ea24651f4f6dcbbec9a1f1ffb01b9a1f23e07847b4a5da85b6390b55",
    "crates/vize_maestro/src/server/state/project_contexts/registry/closed_tests.rs":
      "0a3c9f3d49d8171d873d312652d5523fde0eb8f186ca49110341b640335678e4",
    "crates/vize_maestro/src/server/state/project_contexts/shutdown.rs":
      "bd747c703b70bc23a77eb43c92c15005612856f848ecbc6875a52e8683811d84",
    "tests/tooling/lsp-overlay-owned-read-roster.test.ts":
      "ab25b1875a2679af41be15a5df247cdddf59cc9dbca6d16a165f3d62999e6d32",
  },
};
export const CLOCK_PREFIX = "server::initial_diagnostics::tests::prepared_readiness::";
export const CLOCK_TESTS = [
  "current_completed_preparation_advances_only_the_same_open_slot",
  "stale_source_reopen_environment_and_retirement_keep_the_original_grace",
  "older_completion_never_advances_a_replaced_or_reopened_job",
  "sync_feedback_and_actual_preparation_cannot_be_overtaken",
  "retained_retries_keep_their_original_deadline_after_preparation",
  "edit_deadlines_missing_slots_and_elapsed_grace_are_unchanged",
].map((name) => CLOCK_PREFIX + name);
export const NATIVE_TEST = "tests/tooling/lsp-startup-local-interactive.test.ts";
export function assertSources(repo: string): void {
  for (const [revision, pins] of Object.entries(SOURCE_PINS))
    for (const [file, digest] of Object.entries(pins)) {
      if (digest === null)
        assert.equal(
          execFileSync("git", ["ls-tree", revision, "--", file], { cwd: repo, encoding: "utf8" }),
          "",
        );
      else
        assert.equal(
          hash(execFileSync("git", ["show", `${revision}:${file}`], { cwd: repo })),
          digest,
        );
    }
}

export function checkClockExecution(stdout: string): void {
  const clean = stdout.replace(/\u001b\[[0-9;]*m/g, "");
  assert.deepEqual(
    [...clean.matchAll(/^test (\S+) \.\.\./gm)].map((row) => row[1]).sort(),
    [...CLOCK_TESTS].sort(),
  );
  const rows = [
    ...clean.matchAll(
      /test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured;/g,
    ),
  ];
  assert.equal(rows.length, 1);
  assert.deepEqual(rows[0]!.slice(1), ["6", "0", "0", "0"]);
}
export function checkNativeExecution(stdout: string): void {
  const expected = [
    "cold CSS hover and exact local completion stay prompt while native preparation owns its scope",
    "JSX opt-out keeps SFC-looking strings inert and opt-in keeps native type hover",
  ];
  assert.deepEqual(
    [...stdout.matchAll(/^ok \d+ - (.+)$/gm)].map((row) => row[1]),
    expected,
  );
  assert.doesNotMatch(stdout, /^not ok /m);
  for (const [label, count] of Object.entries({
    tests: 2,
    suites: 0,
    pass: 2,
    fail: 0,
    cancelled: 0,
    skipped: 0,
    todo: 0,
  })) {
    const rows = [...stdout.matchAll(new RegExp(`^# ${label} (\\d+)$`, "gm"))];
    assert.equal(rows.length, 1);
    assert.equal(rows[0]![1], String(count));
  }
}
export const SDK_CRATES = [
  "corsa",
  "corsa_client",
  "corsa_core",
  "corsa_jsonrpc",
  "corsa_lsp",
  "corsa_orchestrator",
  "corsa_runtime",
];
export function libraryArtifact(
  values: Record<string, unknown>[],
  target: string,
  repo: string,
): Record<string, unknown> {
  assert.equal(
    values.filter((row) => row["reason"] === "build-finished" && row["success"] === true).length,
    1,
  );
  const artifacts = values.filter((row) => row["reason"] === "compiler-artifact");
  const libraries = artifacts.filter(
    (row) =>
      object(row["target"])["name"] === "vize_maestro" &&
      object(row["profile"])["test"] === true &&
      typeof row["executable"] === "string",
  );
  assert.equal(libraries.length, 1);
  const library = libraries[0];
  assert(library !== undefined);
  const features = library["features"];
  assert(Array.isArray(features) && features.includes("native"));
  assert.ok(
    artifacts.every((row) => {
      const f = row["features"];
      return Array.isArray(f) && !f.includes("profiling") && !f.includes("micro-profiling");
    }),
  );
  const meta = object(library["target"]);
  assert.deepEqual(meta["kind"], ["lib"]);
  assert.equal(library["manifest_path"], path.join(repo, "crates/vize_maestro/Cargo.toml"));
  assert.equal(meta["src_path"], path.join(repo, "crates/vize_maestro/src/lib.rs"));
  assert.equal(
    text(library["package_id"]).split("#")[0],
    "path+" + pathToFileURL(path.join(repo, "crates/vize_maestro")).href,
  );
  const executable = text(library["executable"]);
  assert.ok(path.isAbsolute(executable));
  const relative = path.relative(target, executable);
  assert.ok(relative.startsWith("ci/deps/") && !relative.split(path.sep).includes(".."));
  const sdk = artifacts.filter((row) => SDK_CRATES.includes(text(object(row["target"])["name"])));
  assert.deepEqual(
    [...new Set(sdk.map((row) => text(row["package_id"]).split("#")[1]))].sort(),
    SDK_CRATES.map((name) => name + "@2.0.0-beta.4").sort(),
  );
  assert.ok(
    sdk.every(
      (row) =>
        text(row["package_id"]).startsWith("registry+") &&
        JSON.stringify(object(row["target"])["kind"]) === '["lib"]',
    ),
  );
  return library;
}

export function sdkArtifacts(rows: Record<string, unknown>[]): Record<string, unknown>[] {
  const artifacts = rows.filter((row) => row["reason"] === "compiler-artifact");
  assert.ok(
    artifacts.every((row) => {
      const f = row["features"];
      return Array.isArray(f) && !f.includes("profiling") && !f.includes("micro-profiling");
    }),
  );
  const sdk = artifacts.filter((row) => SDK_CRATES.includes(text(object(row["target"])["name"])));
  assert.deepEqual(
    [...new Set(sdk.map((row) => text(row["package_id"]).split("#")[1]))].sort(),
    SDK_CRATES.map((name) => name + "@2.0.0-beta.4").sort(),
  );
  assert.ok(
    sdk.every(
      (row) =>
        text(row["package_id"]).startsWith("registry+") &&
        JSON.stringify(object(row["target"])["kind"]) === '["lib"]',
    ),
  );
  return sdk;
}
export function checkCapacity(
  value: { freeDiskBytes: number; physicalMemoryBytes: number; cpuCount: number },
  minimumGiB: number,
): void {
  assert.ok(
    Number.isFinite(value.freeDiskBytes) && value.freeDiskBytes >= minimumGiB * 2 ** 30,
    "distinct build/measurement disk safety floor",
  );
  assert.ok(value.physicalMemoryBytes >= 32 * 2 ** 30);
  assert.ok(value.cpuCount >= 32);
}
export function capacity(
  repo: string,
  phase: string,
  minimumGiB: number,
  destination: string,
): Obj {
  const stat = fs.statfsSync(repo);
  const actual = {
    freeDiskBytes: stat.bavail * stat.bsize,
    physicalMemoryBytes: os.totalmem(),
    cpuCount: os.cpus().length,
    availableParallelism: os.availableParallelism(),
  };
  const read = (name: string): string | null => {
    try {
      return fs.readFileSync(name, "utf8");
    } catch {
      return null;
    }
  };
  const value = {
    phase,
    ...actual,
    minimumFreeGiB: minimumGiB,
    scope: "capacity safety only; no RSS budget relaxation or PSS sampling",
    procMeminfo: read("/proc/meminfo"),
    procSelfCgroup: read("/proc/self/cgroup"),
    cgroupMemoryMax: read("/sys/fs/cgroup/memory.max"),
    cgroupCpuMax: read("/sys/fs/cgroup/cpu.max"),
    cgroupCpuSetEffective: read("/sys/fs/cgroup/cpuset.cpus.effective"),
    cgroupMemoryCurrent: read("/sys/fs/cgroup/memory.current"),
    cgroupMemoryPeak: read("/sys/fs/cgroup/memory.peak"),
    cgroupScope: "mount-root counters; isolation UNKNOWN, charged cache/kernel are not RSS",
  };
  fs.writeFileSync(destination, JSON.stringify(value, null, 2) + "\n", { flag: "wx", mode: 0o600 });
  checkCapacity(actual, minimumGiB);
  return value;
}
export function verifyObservation(value: unknown, source: string, image: string): Obj {
  const row = object(value);
  assert.equal(row.state, "process-closed");
  assert.equal(row.sourceRevision, source);
  assert.equal(object(row.binary).binarySha256, image);
  assert.equal(object(row.binary).sourceRevision, source);
  assert.equal(object(row.buildReceipt).recipe, SHIPPING_RECIPE);
  assert.equal(object(row.buildReceipt).sourceRevision, source);
  assert.ok(Array.isArray(row.sourceWitnesses));
  assert.ok(
    row.sourceWitnesses.some((value) => {
      const witness = object(value);
      return witness.path === NATIVE_TEST && witness.sha256 === SOURCE_PINS[source]?.[NATIVE_TEST];
    }),
  );
  assert.deepEqual(row.process, { exitStatus: 0, signal: null, error: null });
  const streams = object(row.streams);
  assert.deepEqual(Object.keys(streams).sort(), ["client", "server", "stderr"]);
  for (const name of ["client", "server", "stderr"]) {
    const stream = object(streams[name]);
    assert.equal(stream.path, `${name}.bin`);
    assert.equal(stream.truncated, false);
    assert.equal(stream.capturedBytes, stream.observedBytes);
    assert.ok(Number(stream.capturedBytes) >= 0);
    if (name !== "stderr") assert.equal(object(stream.framing).state, "complete");
  }
  return row;
}
export function preserveNativeCaptures(
  repo: string,
  dir: string,
  source: string,
  image: string,
): Obj {
  const root = path.join(repo, "target/differential/lsp-sessions");
  const destination = path.join(dir, "startup-native-sessions");
  if (!fs.existsSync(root)) {
    fs.writeFileSync(
      path.join(dir, "startup-capture-validation.json"),
      JSON.stringify(
        { complete: false, rows: [], errors: ["no capture directory"], expected: 3 },
        null,
        2,
      ) + "\n",
      { flag: "wx", mode: 0o600 },
    );
    assert.fail("source native raw capture directory is required");
  }
  fs.renameSync(root, destination); // Preserve every actual byte before any observation verdict.
  const names = fs.readdirSync(destination);
  const rows: Obj[] = [];
  const errors: string[] = [];
  for (const name of names)
    try {
      assert.ok(fs.lstatSync(path.join(destination, name)).isDirectory());
      const file = path.join(destination, name, "observation.json");
      const row = verifyObservation(JSON.parse(fs.readFileSync(file, "utf8")), source, image);
      for (const [stream, value] of Object.entries(object(row.streams))) {
        const bytes = fs.readFileSync(path.join(destination, name, `${stream}.bin`));
        const metadata = object(value);
        assert.equal(bytes.length, metadata.capturedBytes);
        assert.equal(hash(bytes), metadata.sha256);
      }
      rows.push({ directory: name, observationSha256: hash(fs.readFileSync(file)) });
    } catch (error) {
      errors.push(error instanceof Error ? error.message : String(error));
    }
  fs.writeFileSync(
    path.join(dir, "startup-capture-validation.json"),
    JSON.stringify({ rows, errors, expected: 3 }, null, 2) + "\n",
    { flag: "wx", mode: 0o600 },
  );
  assert.deepEqual(errors, []);
  assert.equal(rows.length, 3, "actual first session plus both JSX modes");
  return { complete: true, rows };
}

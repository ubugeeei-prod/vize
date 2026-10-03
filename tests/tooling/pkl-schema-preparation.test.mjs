import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  downloadPklPackages,
  needsPklSchemaCache,
  pinnedPklPackages,
  verifyPklRuntime,
} from "../../tools/support/compat/github/prepare-pkl-schema.mjs";
import {
  planToolingTests,
  toolingTestFiles,
} from "../../tools/support/compat/github/plan-tooling-tests.mjs";
import { toolingShardMatrix } from "../../tools/support/compat/github/tooling-test-shards.ts";

const lock = JSON.parse(
  readFileSync(new URL("../../npm/cli/pkl/PklProject.deps.json", import.meta.url), "utf8"),
);
const success = { status: 0, stdout: "", stderr: "" };
const timeout = {
  status: 1,
  stderr: "Exception when making request GET https://pkg.pkl-lang.org: request timed out",
};

void test("the committed Pkl packages retain full versions and metadata checksums", () => {
  const packages = pinnedPklPackages(lock);
  assert.equal(packages.length, 2);
  for (const [key, dependency] of Object.entries(lock.resolvedDependencies)) {
    assert.ok(
      packages.includes(
        `${dependency.uri.replace(/^projectpackage:/, "package:")}::sha256:${dependency.checksums.sha256}`,
      ),
    );
    for (const change of [
      { uri: dependency.uri.replace(/\.\d+\.\d+$/, "") },
      { uri: dependency.uri.replace("pkg.pkl-lang.org", "untrusted.example") },
      { type: "local" },
      { checksums: { sha256: "missing" } },
      { checksums: {} },
    ]) {
      const invalid = structuredClone(lock);
      Object.assign(invalid.resolvedDependencies[key], change);
      assert.throws(() => pinnedPklPackages(invalid), /exact remote version and checksum/);
    }
  }
  assert.throws(() => pinnedPklPackages({ schemaVersion: 2 }), /invalid committed/);
  assert.throws(() => pinnedPklPackages({ schemaVersion: 1, resolvedDependencies: {} }), /empty/);
});

void test("only the actual config-generation worker prepares Pkl, after validating the whole plan", () => {
  const available = toolingTestFiles();
  const plan = planToolingTests(["README.md"], { tier: "merge" });
  const selected = toolingShardMatrix(plan).include.filter(({ index, total }) =>
    needsPklSchemaCache(plan, available, `${index}/${total}`, "merge"),
  );
  assert.equal(selected.length, 1);
  assert.throws(
    () => needsPklSchemaCache({ ...plan, tests: plan.tests.slice(1) }, available, "1/4", "merge"),
    /retain every test/,
  );
  assert.throws(() => needsPklSchemaCache(plan, available, "1/4", "pr"), /required execution tier/);
  assert.throws(
    () => needsPklSchemaCache(plan, available, "1/3", "merge"),
    /complete tooling shard/,
  );
});

void test("Pkl preparation checks both the installed package and actual native CLI version", () => {
  const manifest = { name: "@pkl-community/pkl", version: "0.30.2" };
  const run = (args, limit) => {
    assert.deepEqual(args, ["--version"]);
    assert.equal(limit, 10000);
    return { ...success, stdout: "Pkl 0.30.2 (Linux 6.14.0, native)" };
  };
  verifyPklRuntime(manifest, "0.30.2", run);
  assert.throws(
    () => verifyPklRuntime({ ...manifest, version: "0.27.2" }, "0.30.2", run),
    /installed Pkl/,
  );
  assert.throws(() => verifyPklRuntime(manifest, "latest", run), /installed Pkl/);
  for (const stdout of ["Pkl 0.27.2 (Linux, native)", "Pkl 0.30.20 (Linux, native)", ""]) {
    assert.throws(
      () => verifyPklRuntime(manifest, "0.30.2", () => ({ ...success, stdout })),
      /actual Pkl CLI/,
    );
  }
});

void test("locked downloads retry timeouts within a fixed bound while semantic and checksum errors remain fatal", () => {
  const packages = pinnedPklPackages(lock);
  const calls = [];
  downloadPklPackages(packages, "/fresh/cache", (args, limit) => {
    calls.push(args);
    assert.equal(limit, 30000);
    return calls.length === 1 ? timeout : success;
  });
  assert.equal(calls.length, 3);
  assert.deepEqual(calls[0], [
    "download-package",
    "--no-transitive",
    "--cache-dir",
    "/fresh/cache",
    packages[0],
  ]);
  assert.deepEqual(calls[1], calls[0]);
  assert.equal(calls[2].at(-1), packages[1]);
  for (const failure of [timeout, { status: null, error: { code: "ETIMEDOUT" } }]) {
    let attempts = 0;
    assert.throws(
      () =>
        downloadPklPackages(packages, "/fresh/cache", () => {
          attempts += 1;
          return failure;
        }),
      /after 3 attempt/,
    );
    assert.equal(attempts, 3);
  }
  for (const stderr of [
    "Package checksum does not match",
    `${timeout.stderr}\nPackage checksum does not match`,
    "Invalid package URI",
    "Unknown option",
    "HTTP 404",
  ]) {
    let attempts = 0;
    assert.throws(
      () =>
        downloadPklPackages(packages, "/fresh/cache", () => {
          attempts += 1;
          return { status: 1, stderr };
        }),
      /after 1 attempt/,
    );
    assert.equal(attempts, 1);
  }
});

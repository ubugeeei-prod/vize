/** Compare raw Rust source evidence with exact integers and independent source custody. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import {
  digest,
  integer,
  isMain,
  list,
  positive,
  parseExact,
  readExact,
  record,
  render,
  string,
  writeExact,
  type RecordValue,
} from "./profile-evidence-json.ts";
import { profilePair, selfTest, signature } from "./recovery_clone_contract.ts";

function sample(file: string): RecordValue {
  const row = record(readExact(file));
  positive(row.iterations);
  assert(positive(row.sourceBytes) >= positive(row.minimumSourceBytes));
  integer(row.sourceHash);
  return row;
}
function median(values: bigint[]): bigint {
  assert(values.length > 0 && values.length % 2 === 1);
  return [...values].sort((a, b) => (a < b ? -1 : a > b ? 1 : 0))[Math.floor(values.length / 2)]!;
}
function profileIdentity(root: string, requireNormal = true): void {
  const receipt = record(readExact(path.join(root, "identity.json")));
  assert.deepEqual(receipt.base, receipt.firstParent);
  const roles = record(receipt.roles),
    records: Record<string, RecordValue> = {};
  for (const role of ["base", "head"]) {
    const normal = record(roles[role]),
      source = string(normal.sourceRoot);
    if (requireNormal) {
      assert(
        list(normal.artifacts)
          .map(record)
          .every((artifact) => artifact.fresh === false),
      );
      assert(!list(record(list(normal.artifacts)[1]).features).includes("micro-profiling"));
    }
    for (const [name, relative] of [
      ["recoverySha256", "crates/vize_croquis/src/script_parser/recovery.rs"],
      ["lockSha256", "Cargo.lock"],
      ["carrierSha256", "davinci/vize_l0/examples/source_recovery_clone_benchmark.rs"],
    ])
      assert.equal(normal[name!], digest(path.join(source, relative!)));
    const artifacts = fs
      .readFileSync(path.join(root, "profile-build-" + role + ".jsonl"), "utf8")
      .trim()
      .split("\n")
      .map((line) => record(parseExact(line)))
      .filter((row) => row.reason === "compiler-artifact");
    const croquis = artifacts.filter(
      (row) =>
        record(row.target).name === "vize_croquis" && render(record(row.target).kind) === '["lib"]',
    );
    const carriers = artifacts.filter(
      (row) =>
        record(row.target).name === "source_recovery_clone_benchmark" &&
        render(record(row.target).kind) === '["example"]',
    );
    assert.equal(croquis.length, 1);
    assert.equal(carriers.length, 1);
    const compiler = croquis[0]!,
      carrier = carriers[0]!;
    assert.equal(compiler.manifest_path, path.join(source, "crates/vize_croquis/Cargo.toml"));
    assert.equal(
      record(carrier.target).src_path,
      path.join(source, "davinci/vize_l0/examples/source_recovery_clone_benchmark.rs"),
    );
    assert(list(carrier.features).includes("micro-profiling"));
    assert.equal(compiler.fresh, false);
    assert.equal(carrier.fresh, false);
    const binary = digest(string(carrier.executable));
    assert.equal(binary, digest(path.join(root, "instrumented-" + role)));
    records[role] = {
      sourceRoot: source,
      binarySha256: binary,
      artifacts: [compiler, carrier].map((row) =>
        Object.fromEntries(
          [
            "package_id",
            "manifest_path",
            "target",
            "profile",
            "features",
            "fresh",
            "executable",
          ].map((key) => [key, row[key]]),
        ),
      ),
    };
  }
  assert.notEqual(records.base!.binarySha256, records.head!.binarySha256);
  for (const index of [0, 1])
    for (const key of ["features", "profile"]) {
      assert.deepEqual(
        record(list(records.base!.artifacts)[index])[key],
        record(list(records.head!.artifacts)[index])[key],
      );
      if (requireNormal)
        assert.deepEqual(
          record(list(record(roles.base).artifacts)[index])[key],
          record(list(record(roles.head).artifacts)[index])[key],
        );
    }
  receipt.instrumentedRoles = records;
  writeExact(path.join(root, "identity.json"), receipt);
}
function profileFiles(root: string, scenario: string, size: bigint) {
  const load = (role: string) =>
    sample(path.join(root, "profile-" + scenario + "-" + role + "-" + size + ".json"));
  const profiles = { base: load("base"), head: load("head") };
  for (const row of Object.values(profiles)) assert.equal(row.minimumSourceBytes, size);
  return profiles;
}
function allocationOnly(mode: string, root: string): void {
  assert(["tiny", "matrix"].includes(mode));
  if (mode === "tiny") profileIdentity(root, false);
  const sizes = mode === "tiny" ? [4096n] : [131072n, 1048576n, 10485760n];
  const cases: unknown[] = [];
  for (const scenario of ["irreparable", "valid", "repairable"])
    for (const size of sizes) {
      const profiles = profileFiles(root, scenario, size),
        row = profiles.base;
      cases.push({
        scenario,
        minimumSourceBytes: size,
        sourceBytes: row.sourceBytes,
        sourceHash: row.sourceHash,
        iterations: row.iterations,
        allocation: profilePair(profiles.base, profiles.head),
      });
    }
  const summary = {
    scope: "instrumented Rust allocation-request and source-count laws",
    normalTimingQualified: false,
    profileElapsedNsQualified: false,
    cases,
  };
  writeExact(path.join(root, "clone-allocation-" + mode + "-summary.json"), summary);
  console.log("allocationComparison=" + render(summary));
}
function normalSummary(mode: string, root: string): void {
  assert(["tiny", "matrix"].includes(mode));
  if (mode === "tiny") profileIdentity(root);
  const sizes = mode === "tiny" ? [4096n] : [131072n, 1048576n, 10485760n],
    rounds = mode === "tiny" ? [1] : [1, 2, 3];
  const cases: unknown[] = [];
  for (const scenario of ["irreparable", "valid", "repairable"])
    for (const size of sizes) {
      const load = (role: string) =>
        rounds.map((round) =>
          sample(
            path.join(root, "normal-" + scenario + "-" + role + "-" + size + "-" + round + ".json"),
          ),
        );
      const normal = { base: load("base"), head: load("head") },
        expected = signature(normal.base[0]!);
      for (const rows of Object.values(normal))
        for (const row of rows) {
          assert.deepEqual(signature(row), expected);
          assert.equal(row.minimumSourceBytes, size);
          assert.equal(row.instrumented, false);
        }
      const profiles = profileFiles(root, scenario, size);
      for (const row of Object.values(profiles))
        for (const key of ["minimumSourceBytes", "sourceBytes", "sourceHash", "rows", "scenario"])
          assert.deepEqual(row[key], normal.base[0]![key]);
      const timing = Object.fromEntries(
        Object.entries(normal).map(([role, rows]) => {
          const samplesNs = rows.map((row) => integer(row.elapsedNs));
          return [
            role,
            { samplesNs, medianNs: median(samplesNs), iterations: rows[0]!.iterations },
          ];
        }),
      );
      cases.push({
        scenario,
        minimumSourceBytes: size,
        sourceBytes: normal.base[0]!.sourceBytes,
        sourceHash: normal.base[0]!.sourceHash,
        normal: timing,
        allocation: profilePair(profiles.base, profiles.head),
      });
    }
  const summary = {
    scope:
      "bounded repeated-source parse+analysis; generation/report outside inner window, no backend or project cold-time claim",
    timingMetric: "normal inner wall time, separate from instrumented allocations",
    rssScope: "whole process includes source generation; /usr/bin/time raw receipts only",
    cases,
  };
  writeExact(path.join(root, "clone-" + mode + "-summary.json"), summary);
  console.log("cloneComparison=" + render({ mode, comparison: summary }));
}
export function main(): void {
  const mode = process.argv[2] ?? "";
  if (mode === "--self-test") {
    selfTest();
    return;
  }
  const root = path.resolve(process.argv[3] ?? "evidence");
  if (mode.startsWith("allocation-")) allocationOnly(mode.slice("allocation-".length), root);
  else normalSummary(mode, root);
}
if (isMain(import.meta.url)) main();

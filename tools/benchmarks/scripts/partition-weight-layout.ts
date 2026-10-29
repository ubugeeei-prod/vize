/** Ordinary Cargo layout must preserve the previously measured source bytes. */
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { createHash } from "node:crypto";

export const layoutBaseline = "09ff11b5a3440381d4f4558652885ec19520e4f1";
export const oldBenchPath = "crates/vize_canon/benches/partition_weight.rs";
export const newBenchPath = "crates/vize_canon/benches/partition_weight/main.rs";
export const conservedPaths = [
  "crates/vize_canon/benches/partition_weight/baseline.rs",
  "crates/vize_canon/benches/partition_weight/candidate.rs",
  "tools/benchmarks/scripts/partition-weight-kernel-control.ts",
] as const;
type Bodies = { main: string; children: readonly string[] };
export function verifyPartitionBenchLayout(before: Bodies, after: Bodies, manifest: string) {
  let expected = before.main;
  for (const attribute of [
    '#[path = "partition_weight/baseline.rs"]\n',
    '#[path = "partition_weight/candidate.rs"]\n',
  ]) {
    assert.equal(
      expected.split(attribute).length,
      2,
      "both original path attributes must occur exactly once",
    );
    expected = expected.replace(attribute, "");
  }
  assert.equal(
    after.main,
    expected,
    "benchmark body may change only by removing two module overrides",
  );
  assert.deepEqual(
    after.children,
    before.children,
    "source-pinned kernels and source-authenticity checker stay byte-identical",
  );
  assert.equal(before.children.length, 3);
  assert.match(
    manifest,
    /\[\[bench\]\]\s+name = "partition_weight"\s+path = "benches\/partition_weight\/main.rs"\s+harness = false/u,
  );
  return {
    bodyPreserved: true,
    kernelsPreserved: true,
    sourceCheckerPreserved: true,
    ordinaryModules: true,
    normalRecipeChanged: false,
  };
}
export function inspectPartitionBenchLayout() {
  const original = (path: string) =>
    execFileSync("git", ["show", layoutBaseline + ":" + path], { encoding: "utf8" });
  const actual = (path: string) => readFileSync(path, "utf8");
  assert.equal(existsSync(oldBenchPath), false, "obsolete crate root must disappear");
  const before = { main: original(oldBenchPath), children: conservedPaths.map(original) };
  const after = { main: actual(newBenchPath), children: conservedPaths.map(actual) };
  const result = verifyPartitionBenchLayout(before, after, actual("crates/vize_canon/Cargo.toml"));
  const sha = (body: string) => createHash("sha256").update(body).digest("hex");
  return {
    ...result,
    baseline: layoutBaseline,
    source: execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim(),
    newMainSha256: sha(after.main),
    conserved: conservedPaths.map((path, index) => ({
      path,
      sha256: sha(after.children[index] ?? ""),
    })),
  };
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href)
  console.log(JSON.stringify(inspectPartitionBenchLayout()));

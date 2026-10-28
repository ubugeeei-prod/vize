/** Bind uninstrumented normal kernels to the actual old/current source region. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import {
  partitionControlBlob,
  partitionControlSource,
  partitionSourcePath,
} from "./partition-weight-control.ts";
const header =
  "use super::Bins;\nuse vize_canon::batch::VirtualFile;\nuse vize_carton::FxHashMap;\n\npub(super) fn pack(partitioned: &[&VirtualFile], component_files: FxHashMap<usize, Vec<usize>>, servers: usize) -> Option<Bins> {\n";
function one(body: string, needle: string, replacement: string) {
  assert.equal(body.split(needle).length, 2);
  return body.replace(needle, replacement);
}
export function partitionNormalKernel(source: string) {
  const start = source.indexOf("    let weight = |file_indices:"),
    end = source.indexOf("    // Only a useful leaf plan", start);
  assert.ok(start >= 0 && end > start);
  let body = source.slice(start, end);
  if (body.includes("weight_work::")) {
    body = one(body, "        #[cfg(test)]\n        weight_work::record_call();\n", "");
    body = one(
      body,
      "            .filter_map(|&index| {\n                #[cfg(test)]\n                weight_work::record_member(index);\n                partitioned.get(index)\n            })",
      "            .filter_map(|&index| partitioned.get(index))",
    );
  }
  assert.ok(!body.includes("weight_work::"));
  assert.equal(body.split("return no_sharding;").length, 4);
  return header + body.replaceAll("return no_sharding;", "return None;") + "    Some(bins)\n}\n";
}
export function verifyPartitionNormalKernels(
  old: string,
  current: string,
  baseline: string,
  candidate: string,
  rustfmt = process.env.RUSTFMT ?? "rustfmt",
) {
  const bytes = Buffer.from(old);
  assert.equal(
    createHash("sha1")
      .update("blob " + bytes.length + "\0")
      .update(bytes)
      .digest("hex"),
    partitionControlBlob,
  );
  const format = (body: string) => {
    const run = spawnSync(rustfmt, ["--edition", "2024", "--emit", "stdout"], {
      input: body,
      encoding: "utf8",
      timeout: 5000,
    });
    assert.equal(run.error, undefined);
    assert.equal(run.status, 0, run.stderr);
    return run.stdout;
  };
  assert.equal(format(partitionNormalKernel(old)), format(baseline));
  assert.equal(format(partitionNormalKernel(current)), format(candidate));
  return {
    normalKernels: true,
    instrumentation: false,
    baselineSource: partitionControlSource,
    baselineBlob: partitionControlBlob,
    candidateSource: null,
    observerEventSource: process.env.GITHUB_SHA ?? null,
    candidateBlob: createHash("sha1")
      .update("blob " + Buffer.byteLength(current) + "\0")
      .update(current)
      .digest("hex"),
    baselineKernelSha256: createHash("sha256").update(baseline).digest("hex"),
    candidateKernelSha256: createHash("sha256").update(candidate).digest("hex"),
    scope:
      "normal source-pinned weight/sort/binpacking kernel only; fixture/map setup excluded; not full partition/check/cold latency",
  };
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const run = spawnSync("git", ["show", partitionControlSource + ":" + partitionSourcePath], {
    encoding: "utf8",
    timeout: 10000,
  });
  assert.equal(run.error, undefined);
  assert.equal(run.status, 0, run.stderr);
  const current = readFileSync(partitionSourcePath, "utf8"),
    baseline = readFileSync("crates/vize_canon/benches/partition_weight/baseline.rs", "utf8"),
    candidate = readFileSync("crates/vize_canon/benches/partition_weight/candidate.rs", "utf8");
  const receipt = verifyPartitionNormalKernels(run.stdout, current, baseline, candidate);
  assert.throws(() =>
    verifyPartitionNormalKernels(
      run.stdout,
      current,
      baseline,
      candidate.replace("Some(bins)", "None"),
    ),
  );
  assert.throws(() =>
    verifyPartitionNormalKernels(
      run.stdout,
      current,
      baseline.replace("bin.0 += weight(&group)", "bin.0 += 0"),
      candidate,
    ),
  );
  const source = spawnSync("git", ["rev-parse", "HEAD"], { encoding: "utf8", timeout: 5000 });
  assert.equal(source.error, undefined);
  assert.equal(source.status, 0, source.stderr);
  assert.match(source.stdout.trim(), /^[0-9a-f]{40}$/u);
  console.log(
    JSON.stringify({ ...receipt, candidateSource: source.stdout.trim(), rejectedKernelChanges: 2 }),
  );
}

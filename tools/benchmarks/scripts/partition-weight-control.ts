/** Bind the test-only old planner to its immutable source before compiling laws. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

export const partitionControlSource = "b6127a994f7e2a75eb2c89de44d54cf8a0f17fcd";
export const partitionControlBlob = "a6b8c5aeef0865336ec13d75214c0e6c8232e0ef";
export const partitionSourcePath = "crates/vize_canon/src/batch/executor/cli/partition.rs";
export const partitionControlPath =
  "crates/vize_canon/src/batch/executor/cli/partition/weight_work/baseline.rs";
function replaceOne(body: string, needle: string, value: string) {
  assert.equal(body.split(needle).length, 2, "every permitted test hook must occur exactly once");
  return body.replace(needle, value);
}
export function stripPartitionWeightHooks(control: string) {
  const start = control.indexOf("pub(super) fn partition(");
  assert.ok(start >= 0, "source-pinned planner function missing");
  let body = control.slice(start).replace("pub(super) fn partition(", "fn partition(");
  body = replaceOne(body, "    #[cfg(test)]\n    weight_work::start_partition();\n", "");
  body = replaceOne(body, "        #[cfg(test)]\n        weight_work::record_call();\n", "");
  body = replaceOne(
    body,
    "            .filter_map(|&index| {\n                #[cfg(test)]\n                weight_work::record_member(index);\n                partitioned.get(index)\n            })",
    "            .filter_map(|&index| partitioned.get(index))",
  );
  assert.ok(!body.includes("weight_work::"), "unexpected control instrumentation");
  return body;
}
export function verifyPartitionWeightControl(
  source: string,
  control: string,
  rustfmt = process.env.RUSTFMT ?? "rustfmt",
) {
  const bytes = Buffer.from(source);
  const blob = createHash("sha1")
    .update("blob " + bytes.length + "\0")
    .update(bytes)
    .digest("hex");
  assert.equal(blob, partitionControlBlob, "immutable old planner source changed");
  const start = source.indexOf("fn partition("),
    end = source.indexOf("\n/// Quoted module");
  assert.ok(start >= 0 && end > start);
  const expected = source.slice(start, end);
  const normalized = (text: string) => {
    const child = spawnSync(rustfmt, ["--edition", "2024", "--emit", "stdout"], {
      input: text,
      encoding: "utf8",
      timeout: 5000,
    });
    assert.equal(child.error, undefined);
    assert.equal(child.status, 0, child.stderr);
    return child.stdout;
  };
  assert.equal(
    normalized(stripPartitionWeightHooks(control)),
    normalized(expected),
    "the old reference may differ only by the explicit test hooks and visibility",
  );
  return {
    sourceSha: partitionControlSource,
    sourceBlob: blob,
    controlBytes: Buffer.byteLength(control),
    controlSha256: createHash("sha256").update(control).digest("hex"),
    verified: true,
    scope:
      "test-only source-pinned planner; this does not measure latency, RSS or hardware instructions",
  };
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const old = spawnSync("git", ["show", partitionControlSource + ":" + partitionSourcePath], {
    encoding: "utf8",
    timeout: 10_000,
  });
  assert.equal(old.error, undefined);
  assert.equal(old.status, 0, old.stderr);
  const control = readFileSync(partitionControlPath, "utf8");
  const verified = verifyPartitionWeightControl(old.stdout, control);
  assert.throws(() => verifyPartitionWeightControl(old.stdout + " ", control));
  assert.throws(() =>
    verifyPartitionWeightControl(old.stdout, control.replace("weight_work::record_call();", "")),
  );
  assert.throws(() =>
    verifyPartitionWeightControl(
      old.stdout,
      control.replace("bin.0 += weight(&group);", "bin.0 += 0;"),
    ),
  );
  console.log(JSON.stringify({ ...verified, rejectedControlChanges: 3 }));
}

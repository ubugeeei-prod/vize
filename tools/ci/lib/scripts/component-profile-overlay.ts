import assert from "node:assert/strict";
import { join } from "node:path";
import { requiredEnv, textFile, writeJson, fileSha256 } from "../shared/files.ts";

import { spawnSync } from "node:child_process";
import { writeFileSync } from "node:fs";
const probes = [
  [
    "crates/vize_canon/src/virtual_ts/script_facts.rs",
    "pub(super) fn with_bindings<T>",
    "canon.component_binding_observer.with_bindings",
  ],
  [
    "crates/vize_croquis/src/facts/access.rs",
    "pub fn used_component_name_list(",
    "canon.component_binding_observer.used_component_name_list",
  ],
] as const;
const evidence = requiredEnv("EVIDENCE"),
  baseDir = process.argv[2];
assert.ok(baseDir);
function instrument(role: string, source: string) {
  return probes.map(([path, signature, label], index) => {
    const file = join(source, path),
      original = textFile(file);
    const start = original.indexOf(signature),
      brace = original.indexOf(" {\n", start),
      opening = brace + 2,
      end = original.indexOf("\n}", opening);
    assert.ok(start >= 0 && brace >= start && end >= opening, signature);
    const patched =
      original.slice(0, opening) +
      '\n    vize_carton::profile!("' +
      label +
      '", {' +
      original.slice(opening, end) +
      "\n    })" +
      original.slice(end);
    assert.equal(patched.split('"' + label + '"').length, 2);
    const stem = join(evidence, role + "-overlay-" + index);
    const before = stem + ".original.rs.txt",
      after = stem + ".instrumented.rs.txt";
    writeFileSync(before, original);
    writeFileSync(after, patched);
    const diff = spawnSync("diff", ["-u", "--label", path, "--label", path, before, after], {
      encoding: "utf8",
      maxBuffer: 16 * 1024 * 1024,
    });
    assert.equal(diff.status, 1, diff.stderr);
    writeFileSync(stem + ".diff", diff.stdout);
    writeFileSync(file, patched);
    return {
      role,
      path,
      label,
      originalSha256: fileSha256(before),
      overlaySha256: fileSha256(after),
      scope:
        "profiling-only common wrapper; inclusive current-thread function entry count and requested layout allocation traffic; no normal binary contains this observer",
    };
  });
}
const base = instrument("base", join(requiredEnv("RUNNER_TEMP"), baseDir)),
  head = instrument("head", requiredEnv("GITHUB_WORKSPACE"));
for (const [index, row] of base.entries()) {
  assert.equal(row.originalSha256, head[index]!.originalSha256);
  assert.equal(row.overlaySha256, head[index]!.overlaySha256);
}
writeJson(join(evidence, "profiling-overlays.json"), [...base, ...head]);

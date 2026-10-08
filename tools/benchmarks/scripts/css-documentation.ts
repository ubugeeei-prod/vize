import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createReadStream, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { observeCss } from "../../../tests/tooling/support/lsp/css-documentation.ts";

const [baseBinary, headBinary, baseSha, headSha, output] = process.argv.slice(2);
assert.ok(baseBinary && headBinary && baseSha && headSha && output);
assert.match(baseSha, /^[a-f0-9]{40}$/);
assert.match(headSha, /^[a-f0-9]{40}$/);
async function sha256(file: string) {
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(file)) hash.update(chunk);
  return hash.digest("hex");
}
const rows = [];
for (const [label, binary, source] of [
  ["base", baseBinary, baseSha],
  ["head", headBinary, headSha],
] as const) {
  const absolute = resolve(binary);
  const observations = await observeCss(absolute, false, label === "head", 80);
  rows.push({
    label,
    source,
    binary: absolute,
    binarySha256: await sha256(absolute),
    observations: observations.map((row) => {
      const sorted = [...row.samplesMs].sort((a, b) => a - b);
      return {
        ...row,
        medianMs: sorted[Math.floor(sorted.length / 2)],
        p95Ms: sorted[Math.ceil(sorted.length * 0.95) - 1],
      };
    }),
  });
}
writeFileSync(
  output,
  `${JSON.stringify(
    {
      scope:
        "Repeated real stdio completion/resolve/hover with identical inputs and existing base/head CI builds; different response sizes/product behavior are explicit; not a speedup or instruction-count gate",
      warmups: 5,
      samples: 80,
      instructionCounts: null,
      serverAllocations: null,
      unavailableCounters:
        "Existing stdio measurement path has no instruction or server allocation instrumentation.",
      rows,
    },
    null,
    2,
  )}\n`,
);

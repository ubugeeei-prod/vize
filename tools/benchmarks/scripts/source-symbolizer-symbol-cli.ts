/** Offline symbol replay; the observed product and driver identities stay separate. */
import assert from "node:assert/strict";
import { resolve, join } from "node:path";
import { parseArgs } from "node:util";
import { retainedNormalIdentity } from "./source-symbolizer-smoke.ts";
import { renderSymbolCpu } from "./source-symbolizer-symbols.ts";
const { values } = parseArgs({
  options: {
    perf: { type: "string" },
    out: { type: "string" },
    root: { type: "string" },
  },
});
assert.ok(values.perf && values.out && values.root);
const { original, build, binary } = retainedNormalIdentity(resolve(values.root));
const result = renderSymbolCpu({
  binary,
  data: join(resolve(values.root), "record-head/direct-process/perf.data"),
  perf: values.perf,
  out: resolve(values.out),
  totalSamples: original.sampleCount,
  identity: {
    observerSource: process.env.GITHUB_SHA ?? null,
    compiledSource: build.sourceSha,
    build,
    originalCapture: original,
  },
});
assert.ok(result.qualified, "offline symbol self census remains unqualified");
console.log(
  JSON.stringify({
    qualified: result.qualified,
    census: result.census,
    compiledSource: build.sourceSha,
    binarySha256: result.binarySha256,
    sourceLineQualified: false,
    hardwareInstructions: null,
  }),
);

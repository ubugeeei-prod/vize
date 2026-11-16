import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildV1SupportMatrix,
  renderV1SupportMatrix,
} from "../../support/compat/v1/support-matrix.ts";

const script = fileURLToPath(import.meta.url);
const root = path.resolve(path.dirname(script), "../../..");
const output = path.join(root, "docs/release/v1-supported-matrix.json");
const rendered = renderV1SupportMatrix(root);
const matrix = buildV1SupportMatrix(root);

if (process.argv.includes("--check")) {
  assert.equal(readFileSync(output, "utf8"), rendered, "v1 support matrix is stale");
  console.log(`Verified ${path.relative(root, output)}`);
} else {
  writeFileSync(output, rendered);
  console.log(`Generated ${path.relative(root, output)}`);
}

if (process.argv.includes("--enforce-complete")) {
  const blocking = matrix.knownGaps.map(({ id }) => id).join(", ");
  assert.equal(
    matrix.releaseReady,
    true,
    `v1 qualification is incomplete: ${matrix.counts.completeRequiredRows}/${matrix.counts.requiredRows} required rows; blocking gaps: ${blocking}`,
  );
}

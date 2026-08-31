// Croquis consumption matrix generator (Davinci P0-7).
//
// Enumerates the public analysis products of crates/vize_croquis
// (the `pub` fields of the `Croquis` struct in src/croquis.rs, the
// tracker/product types those fields reference, the types re-exported by
// croquis.rs, and the crate-root `pub use` groups in src/lib.rs), then
// resolves consumers across every other workspace crate symbol-aware:
// per-file alias tables built from parsed Rust `use` declarations
// (braces, `as` aliases, `pub use` re-export chains) plus typed-receiver
// field-access counting. A naive text-grep lane runs as a cross-check and
// disagreements are reported in the artifact, never reconciled.
//
// This file is the CLI; the stages live in ./lib:
//   rust-source.mjs        comment/string stripping + `use`-tree parsing
//   croquis-products.mjs   product enumeration from vize_croquis sources
//   crates.mjs             workspace crate + .rs discovery
//   croquis-producers.mjs  workspace table of croquis-value producers
//   croquis-file-index.mjs per-file alias tables (pub-use fixpoint)
//   croquis-analysis.mjs   site counting + naive grep lane
//   croquis-render.mjs     artifact rendering
//
// Usage:
//   rust-script tools/commands/davinci/croquis-consumers.rs --write   # regenerate artifact
//   rust-script tools/commands/davinci/croquis-consumers.rs --check   # diff against committed
//
// Node builtins only. Output is deterministic (stable sort everywhere,
// no timestamps, no absolute paths).

import { existsSync, readFileSync, writeFileSync } from "node:fs";

import { analyzeConsumers } from "./lib/croquis-analysis.mjs";
import { enumerateProducts } from "./lib/croquis-products.mjs";
import { renderArtifact } from "./lib/croquis-render.mjs";
import { ARTIFACT, ARTIFACT_REL, REGEN_COMMAND } from "./lib/paths.mjs";

function generate() {
  const products = enumerateProducts();
  const analysis = analyzeConsumers(products);
  return renderArtifact(products, analysis);
}

function main() {
  const mode = process.argv[2];
  if (mode !== "--write" && mode !== "--check") {
    console.error(
      "usage: rust-script tools/commands/davinci/croquis-consumers.rs --write | --check",
    );
    process.exit(2);
  }
  const generated = generate();
  if (mode === "--write") {
    writeFileSync(ARTIFACT, generated);
    console.log(`wrote ${ARTIFACT_REL}`);
    return;
  }
  // --check
  if (!existsSync(ARTIFACT)) {
    console.error(`stale: ${ARTIFACT_REL} does not exist. Regenerate with: ${REGEN_COMMAND}`);
    process.exit(1);
  }
  const committed = readFileSync(ARTIFACT, "utf8");
  if (committed === generated) {
    console.log(`${ARTIFACT_REL} is up to date`);
    return;
  }
  const committedLines = committed.split("\n");
  const generatedLines = generated.split("\n");
  let firstDiff = -1;
  const max = Math.max(committedLines.length, generatedLines.length);
  for (let i = 0; i < max; i++) {
    if (committedLines[i] !== generatedLines[i]) {
      firstDiff = i;
      break;
    }
  }
  const committedSet = new Set(committedLines);
  const generatedSet = new Set(generatedLines);
  const removed = committedLines.filter((l) => !generatedSet.has(l)).length;
  const added = generatedLines.filter((l) => !committedSet.has(l)).length;
  console.error(`stale: ${ARTIFACT_REL} drifted from the current sources.`);
  console.error(
    `  first differing line: ${firstDiff + 1} (committed ${committedLines.length} lines, regenerated ${generatedLines.length})`,
  );
  if (firstDiff >= 0) {
    console.error(`  - ${(committedLines[firstDiff] ?? "<eof>").slice(0, 160)}`);
    console.error(`  + ${(generatedLines[firstDiff] ?? "<eof>").slice(0, 160)}`);
  }
  console.error(`  lines only in committed: ${removed}, only in regenerated: ${added}`);
  console.error(`  Regenerate with: ${REGEN_COMMAND}`);
  process.exit(1);
}

main();

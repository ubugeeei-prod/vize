// Rule-parity matrix generator (Davinci P0-8).
//
// Walks every rule source file under crates/vize_patina/src/rules/** and
// derives, per rule: its registration surface(s), whether it runs on the SFC
// (`Linter::lint_sfc`, the `lint()` family) and/or JSX (`Linter::lint_jsx`)
// paths, its vize_croquis usage (symbol-aware, per P0-7's use-declaration
// resolution), and a first-cut portability classification
// (neutral-core-candidate / vue-dialect-bound / container-bound) for charter
// #7's fairness metric. Hand-corrections live in
// davinci-road/plan/rule-parity-overrides.toml and are applied last.
//
// This file is the CLI; the stages live in ./lib:
//   rust-source.mjs          comment/string stripping + `use`-tree parsing
//   rule-parity-paths.mjs    repo locations + rule vocabulary
//   rule-parity-rust-text.mjs comment-only stripping + impl/hook shapes
//   rule-parity-dispatch.mjs trait hook inventories + dispatch anchors
//   rule-parity-registry.mjs registration surfaces (registry + name tables)
//   rule-parity-rules.mjs    per-file extraction + classification signals
//   rule-parity-overrides.mjs the hand-correction sidecar
//   rule-parity-matrix.mjs   matrix assembly + file accounting
//   rule-parity-summary.mjs  derived counts
//   rule-parity-derivation.mjs artifact preamble + column derivation prose
//   rule-parity-tables.mjs   per-rule table, overrides, cross-checks
//   rule-parity-render.mjs   artifact rendering
//
// Usage:
//   rust-script tools/commands/davinci/rule-parity.rs --write   # regenerate artifact
//   rust-script tools/commands/davinci/rule-parity.rs --check   # diff against committed
//
// Node builtins only. Output is deterministic (stable sort everywhere,
// no timestamps, no absolute paths).

import { existsSync, readFileSync, writeFileSync } from "node:fs";

import { buildMatrix } from "./lib/rule-parity-matrix.mjs";
import { ARTIFACT, ARTIFACT_REL, REGEN_COMMAND } from "./lib/rule-parity-paths.mjs";
import { renderArtifact } from "./lib/rule-parity-render.mjs";

function generate() {
  return renderArtifact(buildMatrix());
}

function main() {
  const mode = process.argv[2];
  if (mode !== "--write" && mode !== "--check") {
    console.error("usage: rust-script tools/commands/davinci/rule-parity.rs --write | --check");
    process.exit(2);
  }
  const generated = generate();
  if (mode === "--write") {
    writeFileSync(ARTIFACT, generated);
    console.log(`wrote ${ARTIFACT_REL}`);
    return;
  }
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

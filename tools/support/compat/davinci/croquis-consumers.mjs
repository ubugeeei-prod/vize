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
//   croquis-render.mjs     method prose, per-crate grouping, --summary view
//   croquis-shards.mjs     committed index + one shard per consuming crate
//   artifact-set.mjs       byte-exact --write/--check of the shard set
//
// The committed matrix is sharded: docs/davinci/plan/croquis-consumption.md
// (method + product set, which depend only on vize_croquis) and
// docs/davinci/plan/croquis-consumption/<crate>.md (every per-crate fact).
// Cross-crate totals are never committed — they changed with every PR and
// made every open PR conflict — and are printed on demand by --summary.
//
// Usage:
//   rust-script tools/commands/davinci/croquis-consumers.rs --write     # regenerate index + shards
//   rust-script tools/commands/davinci/croquis-consumers.rs --check     # byte-compare against committed
//   rust-script tools/commands/davinci/croquis-consumers.rs --summary   # print cross-crate totals
//
// Node builtins only. Output is deterministic (stable sort everywhere,
// no timestamps, no absolute paths).

import { checkArtifactSet, writeArtifactSet } from "./lib/artifact-set.mjs";
import { analyzeConsumers } from "./lib/croquis-analysis.mjs";
import { enumerateProducts } from "./lib/croquis-products.mjs";
import { gateViolations, renderSummary } from "./lib/croquis-render.mjs";
import { renderCroquisArtifacts } from "./lib/croquis-shards.mjs";
import { REGEN_COMMAND, SHARD_DIR_REL } from "./lib/paths.mjs";

function main() {
  const mode = process.argv[2];
  if (mode !== "--write" && mode !== "--check" && mode !== "--summary") {
    console.error(
      "usage: rust-script tools/commands/davinci/croquis-consumers.rs --write | --check | --summary",
    );
    process.exit(2);
  }
  const products = enumerateProducts();
  const analysis = analyzeConsumers(products);
  const violations = gateViolations(products, analysis);
  if (violations.missing.length > 0 || violations.stale.length > 0) {
    if (violations.missing.length > 0) {
      console.error(
        `croquis products with no external consumer and no demand gate: ${violations.missing.join(", ")}`,
      );
    }
    if (violations.stale.length > 0) {
      console.error(
        `croquis demand gates whose product now has a consumer: ${violations.stale.join(", ")}`,
      );
    }
    if (mode !== "--summary") process.exit(1);
  }
  if (mode === "--summary") {
    process.stdout.write(renderSummary(products, analysis));
    return;
  }
  const set = {
    label: "croquis consumption matrix",
    files: renderCroquisArtifacts(products, analysis),
    ownedDirs: [SHARD_DIR_REL],
    regenCommand: REGEN_COMMAND,
  };
  if (mode === "--write") writeArtifactSet(set);
  else checkArtifactSet(set);
}

main();

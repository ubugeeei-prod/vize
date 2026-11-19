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
// exact disagreements are printed in every --check report, never reconciled.
//
// This file is the CLI; the stages live in ./lib:
//   rust-source.ts         comment/string stripping + `use`-tree parsing
//   croquis-products.ts    product enumeration from vize_croquis sources
//   crates.ts              workspace crate + .rs discovery
//   croquis-producers.ts   workspace table of croquis-value producers
//   croquis-file-index.ts  per-file alias tables (pub-use fixpoint)
//   croquis-analysis.ts    site counting + naive grep lane
//   croquis-render.ts      method prose, per-crate grouping, --summary view
//   croquis-shards.ts      committed index + one shard per consuming crate
//   artifact-set.ts        byte-exact --write/--check of the shard set
//
// The committed matrix is sharded: docs/davinci/plan/croquis-consumption.md
// (method + product set, which depend only on vize_croquis) and
// docs/davinci/plan/croquis-consumption/<crate>.md (resolved consumption and
// non-product imports). Cross-crate totals and naive-grep diagnostics change
// on independent source edits; every --check prints their fresh report and
// --summary prints the same source-qualified report on demand.
//
// Usage:
//   node tools/support/compat/davinci/croquis-consumers.ts --write     # regenerate index + shards
//   node tools/support/compat/davinci/croquis-consumers.ts --check     # byte-compare against committed
//   node tools/support/compat/davinci/croquis-consumers.ts --summary   # print cross-crate totals
//
// Node builtins only. Output is deterministic (stable sort everywhere,
// no timestamps, no absolute paths).

import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { checkArtifactSet, writeArtifactSet } from "./lib/artifact-set.ts";
import { analyzeConsumers } from "./lib/croquis-analysis.ts";
import { enumerateProducts } from "./lib/croquis-products.ts";
import { gateViolations, renderSummary } from "./lib/croquis-render.ts";
import { renderCroquisArtifacts } from "./lib/croquis-shards.ts";
import { ARTIFACT_REL, REGEN_COMMAND, SHARD_DIR_REL, repoRoot } from "./lib/paths.ts";

interface ArtifactFile {
  readonly relPath: string;
  readonly text: string;
}

function splitCroquisArtifacts(all: readonly ArtifactFile[]): {
  files: ArtifactFile[];
  runFiles: ArtifactFile[];
} {
  return {
    files: all.filter((file) => file.relPath === ARTIFACT_REL),
    runFiles: all.filter((file) => file.relPath !== ARTIFACT_REL),
  };
}

function main() {
  const mode = process.argv[2];
  if (mode !== "--write" && mode !== "--check" && mode !== "--summary") {
    console.error(
      "usage: node tools/support/compat/davinci/croquis-consumers.ts --write | --check | --summary",
    );
    process.exit(2);
  }
  const products = enumerateProducts();
  const analysis = analyzeConsumers(products);
  if (mode === "--check" || mode === "--summary") {
    const revision = execFileSync("git", ["rev-parse", "HEAD"], {
      cwd: repoRoot,
      encoding: "utf8",
    }).trim();
    // Both orphan and staleness failures intentionally exit below. Finish the
    // complete report synchronously before either can truncate piped stdout.
    writeFileSync(
      1,
      `Analyzed working tree at HEAD \`${revision}\`.\n\n${renderSummary(products, analysis)}`,
    );
  }
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
    return;
  }
  const set = {
    label: "croquis consumption matrix",
    // The per-crate shards record line-level facts; they are run artifacts.
    ...splitCroquisArtifacts(renderCroquisArtifacts(products, analysis)),
    ownedDirs: [SHARD_DIR_REL],
    regenCommand: REGEN_COMMAND,
  };
  if (mode === "--write") writeArtifactSet(set);
  else checkArtifactSet(set);
}

main();

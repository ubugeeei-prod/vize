// Storage summary generator (Davinci storage boundary).
//
// The reviewed per-file ledger davinci-road/plan/storage-inventory.tsv is the
// storage ratchet: tests/tooling/davinci-storage-policy.test.ts holds every
// row to strict equality with the measured stage sources. This command derives
// the ledger's aggregates (retained alloc Vec totals, per-category and
// per-scope counts) into davinci-road/plan/storage-summary.md so they are
// never hand-copied, and a rebase that conflicts on them is resolved by
// regenerating.
//
// Usage:
//   rust-script tools/commands/davinci/storage-summary.rs --write   # regenerate
//   rust-script tools/commands/davinci/storage-summary.rs --check   # diff committed

import { readFileSync } from "node:fs";
import path from "node:path";

import { parseStorageInventory } from "../../../../tests/tooling/davinci-storage-inventory.ts";
import {
  STORAGE_INVENTORY_REL,
  STORAGE_SUMMARY_REGEN,
  STORAGE_SUMMARY_REL,
  renderStorageSummary,
} from "../../../../tests/tooling/davinci-storage-summary.ts";
import { checkArtifactSet, writeArtifactSet } from "./lib/artifact-set.mjs";
import { repoRoot } from "./lib/paths.mjs";

function main() {
  const mode = process.argv[2];
  if (mode !== "--write" && mode !== "--check") {
    console.error("usage: rust-script tools/commands/davinci/storage-summary.rs --write | --check");
    process.exit(2);
  }
  const rows = parseStorageInventory(
    readFileSync(path.join(repoRoot, STORAGE_INVENTORY_REL), "utf8"),
  );
  const set = {
    label: STORAGE_SUMMARY_REL,
    files: [{ relPath: STORAGE_SUMMARY_REL, text: renderStorageSummary(rows) }],
    regenCommand: STORAGE_SUMMARY_REGEN,
  };
  if (mode === "--write") writeArtifactSet(set);
  else checkArtifactSet(set);
}

main();

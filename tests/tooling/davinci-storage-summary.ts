// Derived aggregates of the reviewed per-file storage ledger
// (davinci-road/plan/storage-inventory.tsv). The per-file rows are the ratchet
// — the storage policy test holds them to strict equality with the measured
// sources — and every aggregate below is a pure function of those rows, so it
// is generated (`rust-script tools/commands/davinci/storage-summary.rs
// --write`) instead of hand-copied into the plan and the test.

import { formatTable } from "../../legacy-tools/davinci/lib/markdown.mjs";
import {
  summarizeAllocVecCategories,
  summarizeKind,
  summarizeScopes,
  type InventoryRow,
  type StorageSummary,
} from "./davinci-storage-inventory.ts";
import { storageKinds, type StorageKind } from "./davinci-storage-scan.ts";

export const STORAGE_INVENTORY_REL = "davinci-road/plan/storage-inventory.tsv";
export const STORAGE_SUMMARY_REL = "davinci-road/plan/storage-summary.md";
export const STORAGE_SUMMARY_REGEN =
  "rust-script tools/commands/davinci/storage-summary.rs --write";

export const storageTypeNames: Record<StorageKind, string> = {
  allocVec: "alloc::vec::Vec",
  allocString: "alloc::string::String",
  s0String: "vize_s0::String",
  arenaVec: "vize_s0::Vec",
  smallVec: "vize_s0::SmallVec",
};

function counts(summary: StorageSummary): string[] {
  return [String(summary.files), String(summary.directPaths), String(summary.boundUses)];
}

/** The generated storage summary page, byte-compared by the storage policy test. */
export function renderStorageSummary(rows: readonly InventoryRow[]): string {
  const allocVec = summarizeKind(rows, "allocVec");
  const categoryTable = formatTable(
    ["Category", "Files", "Direct paths", "Bound uses"],
    ["left", "right", "right", "right"],
    Object.entries(summarizeAllocVecCategories(rows)).map(([category, summary]) => [
      category,
      ...counts(summary),
    ]),
  );
  const scopeTable = formatTable(
    ["Scope", "Type", "Files", "Direct paths", "Bound uses"],
    ["left", "left", "right", "right", "right"],
    Object.entries(summarizeScopes(rows)).flatMap(([scope, kinds]) =>
      storageKinds.map((kind) => [scope, `\`${storageTypeNames[kind]}\``, ...counts(kinds[kind])]),
    ),
  );
  return `<!-- GENERATED FILE - do not edit by hand.
     Regenerate: ${STORAGE_SUMMARY_REGEN}
     Verify:     rust-script tools/commands/davinci/storage-summary.rs --check
     Source:     ${STORAGE_INVENTORY_REL} (the reviewed per-file ratchet) -->

# Davinci storage summary

Aggregates of the per-file [\`storage-inventory.tsv\`](./storage-inventory.tsv)
ledger behind the [storage boundary](./storage-boundary.md). Every number here
is derived from the ledger rows, so a change that moves a count updates its
file row and regenerates this page; the storage policy test holds the rows to
strict equality with the sources and this page to byte equality with the rows.

## Retained \`alloc::vec::Vec\`

The library trees in the reviewed inventory contain ${allocVec.files} production files,
${allocVec.directPaths} direct \`alloc::vec::Vec\` paths, and ${allocVec.boundUses} bound \`Vec\`/\`StdVec\` uses.

${categoryTable}
## Owned storage by scope

${scopeTable}`;
}

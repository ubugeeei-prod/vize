// Derive run artifacts from the reviewed per-file storage ratchet. The ratchet
// remains committed and is checked independently against production sources.
import { readFileSync } from "node:fs";
import path from "node:path";

import { parseStorageInventory } from "../../../../tests/tooling/davinci-storage-inventory.ts";
import {
  STORAGE_INVENTORY_REL,
  renderStorageSummary,
} from "../../../../tests/tooling/davinci-storage-summary.ts";
import { aggregateMain } from "./lib/aggregate-artifact.mjs";
import { repoRoot } from "./lib/paths.mjs";

aggregateMain(
  "storage-summary.md",
  () =>
    renderStorageSummary(
      parseStorageInventory(readFileSync(path.join(repoRoot, STORAGE_INVENTORY_REL), "utf8")),
    ),
  "usage: rust-script tools/commands/davinci/storage-summary.rs --write | --check | --summary [--out-dir <dir>]",
);

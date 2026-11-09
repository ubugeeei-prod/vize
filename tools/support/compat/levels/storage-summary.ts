// Derive run artifacts from the reviewed per-file storage ratchet. The ratchet
// remains committed and is checked independently against production sources.
import { readFileSync } from "node:fs";
import path from "node:path";

import { parseStorageInventory } from "../../../../tests/tooling/davinci-storage-inventory.ts";
import {
  STORAGE_INVENTORY_REL,
  renderStorageSummary,
} from "../../../../tests/tooling/davinci-storage-summary.ts";
import { aggregateMain } from "./lib/aggregate-artifact.ts";
import { repoRoot } from "./lib/paths.ts";

aggregateMain(
  "storage-summary.md",
  () =>
    renderStorageSummary(
      parseStorageInventory(readFileSync(path.join(repoRoot, STORAGE_INVENTORY_REL), "utf8")),
    ),
  "usage: node tools/support/compat/davinci/storage-summary.ts --write | --check | --summary [--out-dir <dir>]",
);

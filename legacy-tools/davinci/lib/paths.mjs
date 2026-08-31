// Repository locations and artifact identity shared by the croquis
// consumption generator's modules. Keeping them in one place keeps the
// regeneration command quoted in the artifact header in sync with the
// command the CLI actually accepts.

import path from "node:path";
import { fileURLToPath } from "node:url";

export const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
  "..",
);
export const CRATES_DIR = path.join(repoRoot, "crates");
export const CROQUIS_CRATE_DIR = "vize_croquis";
export const CROQUIS_CRATE_NAME = "vize_croquis";
export const CROQUIS_RS = path.join(CRATES_DIR, CROQUIS_CRATE_DIR, "src", "croquis.rs");
export const LIB_RS = path.join(CRATES_DIR, CROQUIS_CRATE_DIR, "src", "lib.rs");
export const ARTIFACT_REL = "davinci-road/plan/croquis-consumption.md";
export const ARTIFACT = path.join(repoRoot, ARTIFACT_REL);
export const REGEN_COMMAND = "rust-script tools/commands/davinci/croquis-consumers.rs --write";

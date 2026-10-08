import fs from "node:fs";
import path from "node:path";
import {
  binaryRelativePath,
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../differential/build-receipt.ts";
import { repoRoot } from "../_helpers/realworld-patch.ts";

// Run the complete original pinned-project laws against this job's fresh CLI.
const binary = path.join(repoRoot, binaryRelativePath());
validateBuildReceipt(
  JSON.parse(fs.readFileSync(`${binary}.differential-build.json`, "utf8")),
  expectedBuildIdentity(repoRoot),
);
process.env.VIZE_TEST_BIN = binary;
process.env.VIZE_LSP_BIN = binary;
process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD = "1";

await import("../snapshots/check/create-vue-patch-oracle.ts");
await import("../snapshots/check/vue-router-patch-oracle.ts");

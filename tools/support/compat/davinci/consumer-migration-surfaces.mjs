// Consumer migration surface inventory for the Davinci rollout plan.
//
// The scan is intentionally observational: it records where the compiler,
// linter, typechecker, typechecker content-mapper, formatter, and LSP still
// name Davinci/S0/S1/S2, legacy AST/parser/Croquis crates, or raw OXC crates.
// It does not change runtime wiring and is safe to merge before any rollout.
//
// The committed inventory is sharded so parallel PRs stop conflicting on it:
// davinci-road/plan/consumer-migration-surfaces.md (method, legend, scopes,
// shard layout — configuration only) plus one TSV per (consumer, crate) under
// davinci-road/plan/consumer-migration-surfaces/. Cross-file totals are never
// committed; --summary prints them.
//
// Usage:
//   rust-script tools/commands/davinci/consumer-migration-surfaces.rs --write
//   rust-script tools/commands/davinci/consumer-migration-surfaces.rs --check
//   rust-script tools/commands/davinci/consumer-migration-surfaces.rs --summary

import { checkArtifactSet, writeArtifactSet } from "./lib/artifact-set.mjs";
import {
  SHARD_DIR_REL,
  renderConsumerMigrationShards,
  renderConsumerMigrationSurfaces,
} from "./lib/consumer-migration-render.mjs";
import { scanConsumerMigrationSurfaces } from "./lib/consumer-migration-scan.mjs";
import {
  SUMMARY_COMMAND,
  renderConsumerMigrationSummary,
} from "./lib/consumer-migration-summary.mjs";

const ARTIFACT_REL = "davinci-road/plan/consumer-migration-surfaces.md";
const REGEN_COMMAND = "rust-script tools/commands/davinci/consumer-migration-surfaces.rs --write";

function main() {
  const mode = process.argv[2];
  if (mode !== "--write" && mode !== "--check" && mode !== "--summary") {
    console.error(
      "usage: rust-script tools/commands/davinci/consumer-migration-surfaces.rs --write | --check | --summary",
    );
    process.exit(2);
  }

  const scan = scanConsumerMigrationSurfaces();
  if (mode === "--summary") {
    process.stdout.write(renderConsumerMigrationSummary(scan));
    return;
  }
  const set = {
    label: "consumer migration surface inventory",
    files: [
      {
        relPath: ARTIFACT_REL,
        text: renderConsumerMigrationSurfaces(scan, {
          regenCommand: REGEN_COMMAND,
          summaryCommand: SUMMARY_COMMAND,
        }),
      },
      ...renderConsumerMigrationShards(scan),
    ],
    ownedDirs: [SHARD_DIR_REL],
    regenCommand: REGEN_COMMAND,
  };
  if (mode === "--write") writeArtifactSet(set);
  else checkArtifactSet(set);
}

main();

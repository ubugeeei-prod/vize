// CI and release publish this bundle for the checkout they actually validated.
// Node builtins only; no Rust compilation or dependency installation is needed.
import { spawnSync } from "node:child_process";
import { existsSync, readdirSync } from "node:fs";
import path from "node:path";

import { emitAggregate, parseAggregateArgs } from "./lib/aggregate-artifact.mjs";
import { repoRoot } from "./lib/paths.mjs";

const generators = [
  ["rule-parity.md", "rule-parity.mjs"],
  ["storage-summary.md", "storage-summary.mjs"],
  ["analysis-consumption-summary.md", "croquis-consumers.mjs"],
  ["consumer-migration-summary.md", "consumer-migration-surfaces.mjs"],
  ["sourcelocation-summary.md", "sourcelocation-inventory.mjs"],
];
const usage =
  "usage: node tools/support/compat/davinci/generated-ledgers.mjs --write | --check [--out-dir <dir>]";

function main() {
  let args;
  try {
    args = parseAggregateArgs(process.argv.slice(2), usage);
    if (args.mode === "--summary") throw new Error(usage);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 2;
    return;
  }
  if (args.mode === "--check" && existsSync(args.outDir)) {
    const wanted = new Set(generators.map(([name]) => name));
    for (const name of readdirSync(args.outDir)) {
      if (!wanted.has(name)) {
        console.error(`stale: unexpected run artifact ${path.join(args.outDir, name)}`);
        process.exitCode = 1;
      }
    }
  }
  for (const [name, generator] of generators) {
    const result = spawnSync(
      process.execPath,
      [path.join(repoRoot, "tools/support/compat/davinci", generator), "--summary"],
      { cwd: repoRoot, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 },
    );
    if (result.error || result.status !== 0 || result.stderr) {
      // A summary CLI may print diagnostics while returning success. Treat those
      // as a failed source gate instead of publishing an apparently clean report.
      console.error(`${generator} failed: ${result.error?.message ?? result.stderr}`);
      process.exitCode = 1;
      return;
    }
    emitAggregate(name, result.stdout, args);
  }
}

main();

// CI and release publish this bundle for the checkout they actually validated.
// Node builtins only; no Rust compilation or dependency installation is needed.
import { spawn } from "node:child_process";
import { existsSync, readdirSync } from "node:fs";
import path from "node:path";

import { emitAggregate, parseAggregateArgs } from "./lib/aggregate-artifact.ts";
import { repoRoot } from "./lib/paths.ts";

const generators: readonly (readonly [name: string, generator: string])[] = [
  ["rule-parity.md", "rule-parity.ts"],
  ["storage-summary.md", "storage-summary.ts"],
  ["analysis-consumption-summary.md", "croquis-consumers.ts"],
  ["consumer-migration-summary.md", "consumer-migration-surfaces.ts"],
  ["sourcelocation-summary.md", "sourcelocation-inventory.ts"],
];
const usage =
  "usage: node tools/support/compat/davinci/generated-ledgers.ts --write | --check [--out-dir <dir>]";

// The generators are independent read-only scans; run them together and emit
// their aggregates in the declared order, stopping at the first failure as before.
interface Summary {
  readonly error?: Error;
  readonly status: number | null;
  readonly stdout: string;
  readonly stderr: string;
}

function summarize(generator: string): Promise<Summary> {
  return new Promise((resolve) => {
    const child = spawn(
      process.execPath,
      [path.join(repoRoot, "tools/support/compat/davinci", generator), "--summary"],
      { cwd: repoRoot },
    );
    const stdout: Buffer[] = [];
    const stderr: Buffer[] = [];
    child.stdout.on("data", (chunk: Buffer) => stdout.push(chunk));
    child.stderr.on("data", (chunk: Buffer) => stderr.push(chunk));
    child.on("error", (error) => resolve({ error, status: null, stdout: "", stderr: "" }));
    child.on("close", (status) =>
      resolve({
        status,
        stdout: Buffer.concat(stdout).toString("utf8"),
        stderr: Buffer.concat(stderr).toString("utf8"),
      }),
    );
  });
}

async function main() {
  let args;
  try {
    args = parseAggregateArgs(process.argv.slice(2), usage);
    if (args.mode === "--summary") throw new Error(usage);
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 2;
    return;
  }
  if (args.mode === "--check" && existsSync(args.outDir)) {
    const wanted = new Set(generators.map(([name]) => name));
    // Subdirectories hold the per-crate inventory shards of other generators.
    for (const entry of readdirSync(args.outDir, { withFileTypes: true })) {
      const { name } = entry;
      if (!entry.isDirectory() && !wanted.has(name)) {
        console.error(`stale: unexpected run artifact ${path.join(args.outDir, name)}`);
        process.exitCode = 1;
      }
    }
  }
  const results = await Promise.all(generators.map(([, generator]) => summarize(generator)));
  for (const [index, [name, generator]] of generators.entries()) {
    const result = results[index];
    if (!result || result.error || result.status !== 0 || result.stderr) {
      // A summary CLI may print diagnostics while returning success. Treat those
      // as a failed source gate instead of publishing an apparently clean report.
      console.error(
        `${generator} failed: ${result?.error?.message ?? result?.stderr ?? "missing result"}`,
      );
      process.exitCode = 1;
      return;
    }
    emitAggregate(name, result.stdout, args);
  }
}

await main();

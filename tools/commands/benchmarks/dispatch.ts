import { spawnSync } from "node:child_process";

import { isEntrypoint, repositoryRoot, runEntrypoint } from "../../support/command-runtime.ts";

const benchmarkTasks = new Map([
  ["run", "tools/benchmarks/scripts/run.ts"],
  ["generate", "tools/benchmarks/scripts/generate.mjs"],
  ["lint", "tools/benchmarks/scripts/lint.ts"],
  ["fmt", "tools/benchmarks/scripts/fmt.ts"],
  ["check", "tools/benchmarks/scripts/check.ts"],
  ["vite", "tools/benchmarks/scripts/vite.ts"],
  ["musea", "tools/benchmarks/scripts/musea.mjs"],
  ["compare-tools", "tools/benchmarks/scripts/compare-tools.mjs"],
]);

function printUsage(): void {
  console.error(
    `Usage: node tools/commands/benchmarks/dispatch.ts <${[...benchmarkTasks.keys()].join("|")}> [args...]`,
  );
}

export function dispatchBenchmark(arguments_: readonly string[]): number {
  const [task, ...forwarded] = arguments_;
  if (task === undefined) {
    printUsage();
    return 1;
  }
  const script = benchmarkTasks.get(task);
  if (script === undefined) {
    console.error(`Unknown benchmark task: ${task}`);
    return 1;
  }

  const result = spawnSync("node", [script, ...forwarded], {
    cwd: repositoryRoot(import.meta.url),
    stdio: "inherit",
  });
  if (result.error !== undefined) throw new Error(`failed to run node ${script}: ${result.error}`);
  return result.status ?? 1;
}

if (isEntrypoint(import.meta.url)) {
  runEntrypoint(() => dispatchBenchmark(process.argv.slice(2)));
}

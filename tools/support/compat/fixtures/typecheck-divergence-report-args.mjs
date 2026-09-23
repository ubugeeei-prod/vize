import { resolve } from "node:path";

import { parseBudgetMode } from "./typecheck-divergence-budget.mjs";

export function parseArgs(argv, { repoRoot, defaultRegistry }) {
  const args = {
    budgetMode: "enforce",
    registry: defaultRegistry,
    reportDir: null,
    shardCount: 1,
    shardIndex: 0,
    vizeBin: null,
    vueTscBin: null,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const value = () => {
      if (argv[index + 1] == null) throw new Error(`${arg} requires a value`);
      return argv[++index];
    };
    if (arg === "--budget-mode") args.budgetMode = parseBudgetMode(value());
    else if (arg === "--registry") args.registry = resolve(repoRoot, value());
    else if (arg === "--report-dir") args.reportDir = resolve(repoRoot, value());
    else if (arg === "--shard-count") args.shardCount = integer(value(), arg, 1);
    else if (arg === "--shard-index") args.shardIndex = integer(value(), arg, 0);
    else if (arg === "--vize-bin") args.vizeBin = value();
    else if (arg === "--vue-tsc-bin") args.vueTscBin = value();
    else throw new Error(`Unknown argument: ${arg}`);
  }
  if (args.reportDir == null) throw new Error("--report-dir is required");
  if (args.vueTscBin == null) throw new Error("--vue-tsc-bin is required");
  if (args.shardIndex >= args.shardCount) {
    throw new Error("--shard-index must be less than --shard-count");
  }
  return args;
}

function integer(value, name, minimum) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed < minimum) {
    throw new Error(`${name} must be an integer >= ${minimum}`);
  }
  return parsed;
}

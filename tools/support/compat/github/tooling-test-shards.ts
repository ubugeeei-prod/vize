import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

export function toolingTestFiles(cwd = fileURLToPath(new URL("../../../../", import.meta.url))) {
  return ["tests/tooling", "tests/tooling/davinci", "tests/tooling/release"]
    .flatMap((directory) =>
      readdirSync(join(cwd, directory))
        .filter((name) => /\.test\.(?:ts|mjs)$/.test(name))
        .map((name) => `${directory}/${name}`),
    )
    .sort((left, right) => (left < right ? -1 : left > right ? 1 : 0));
}

// Partition the selected PR files or the complete merge suite. Actions runners own separate
// checkouts; every runner still executes its own files serially. Measured timings only decide
// where a file runs and how many runners a plan needs, never whether it runs.
export const MAX_TOOLING_SHARDS = 16;
export const TARGET_TOOLING_SHARD_MS = 100_000;
export const TOOLING_TIMINGS_SCHEMA = "vize-tooling-test-timings-v1";
// The table names every tooling test, so it lives beside them.
export const TOOLING_TIMINGS_PATH = new URL(
  "../../../../tests/tooling/fixtures/tooling-test-timings.json",
  import.meta.url,
);
// run-tooling-tests.mjs prints one line with this prefix after each shard's tests.
export const TOOLING_TIMINGS_LOG_MARKER = "vize-tooling-test-timings";

type ToolingTier = "pr" | "merge";
type ToolingShardPlan = {
  tier: ToolingTier;
  tests: readonly string[];
  leadSteps?: readonly string[];
};
export type ToolingTimings = {
  schema: typeof TOOLING_TIMINGS_SCHEMA;
  defaultMs: number;
  leadSteps: Record<string, number>;
  tiers: Record<ToolingTier, Record<string, number>>;
};

// Matrix entry 1 alone also runs these steps of `pr-tooling-scripts`. A PR runs a capture
// only when its planner output is true; the merge queue runs every one of them.
export const TOOLING_LEAD_STEPS = [
  {
    id: "native-ssr",
    step: "Test fresh native SSR modules in the merge queue",
    output: "native-ssr-capture",
  },
  {
    id: "native-scoped-css",
    step: "Verify current-source native scoped CSS and real Vue DOM",
    output: "native-ssr-capture",
  },
  {
    id: "native-js-setup",
    step: "Test source-bound native JS setup modules and runtime",
    output: "native-setup-capture",
  },
  {
    id: "native-vapor",
    step: "Capture original native Vapor modules and runtime",
    output: "native-vapor-capture",
  },
  {
    id: "native-attribute-values-7502",
    step: "Capture original attribute values across native targets",
    output: "native-attribute-values-7502-capture",
  },
  {
    id: "native-selected-template",
    step: "Execute original selected-template DOM modules",
    output: "merge-only",
  },
  {
    id: "historical-sfc-documents",
    step: "Join original historical Vue interpolation Docs to pinned whole goldens",
    output: "always",
  },
] as const;

export function toolingLeadSteps(tier: ToolingTier, outputs: Record<string, boolean>) {
  return TOOLING_LEAD_STEPS.filter(
    ({ output }) =>
      output === "always" || tier === "merge" || (output !== "merge-only" && outputs[output]),
  ).map(({ id }) => id);
}

const isDuration = (value: unknown): value is number =>
  Number.isSafeInteger(value) && (value as number) >= 0;
const isDurationMap = (value: unknown): value is Record<string, number> =>
  typeof value === "object" &&
  value !== null &&
  !Array.isArray(value) &&
  Object.values(value).every(isDuration);

export function validateToolingTimings(value: unknown): ToolingTimings {
  const timings = value as ToolingTimings;
  if (
    timings?.schema !== TOOLING_TIMINGS_SCHEMA ||
    !isDuration(timings.defaultMs) ||
    timings.defaultMs === 0 ||
    !isDurationMap(timings.leadSteps) ||
    !TOOLING_LEAD_STEPS.every(({ id }) => Object.hasOwn(timings.leadSteps, id)) ||
    Object.keys(timings.leadSteps).length !== TOOLING_LEAD_STEPS.length ||
    !isDurationMap(timings.tiers?.pr) ||
    !isDurationMap(timings.tiers?.merge) ||
    Object.keys(timings.tiers).length !== 2
  ) {
    throw new Error("invalid tooling test timing table");
  }
  return timings;
}

let committedTimings: ToolingTimings | undefined;
function toolingTimings() {
  committedTimings ??= validateToolingTimings(
    JSON.parse(readFileSync(TOOLING_TIMINGS_PATH, "utf8")),
  );
  return committedTimings;
}

// Files measured in neither tier (new files) get a deliberately high estimate: an
// overestimate costs one more runner, an underestimate lengthens the critical path.
export function estimateToolingTestMs(tier: ToolingTier, file: string) {
  const { tiers, defaultMs } = toolingTimings();
  return tiers[tier][file] ?? tiers[tier === "pr" ? "merge" : "pr"][file] ?? defaultMs;
}

function validatePlan(plan: ToolingShardPlan) {
  const lead = plan.leadSteps ?? [];
  const known = new Set<string>(TOOLING_LEAD_STEPS.map(({ id }) => id));
  if (
    !["pr", "merge"].includes(plan.tier) ||
    !Array.isArray(plan.tests) ||
    !Array.isArray(lead) ||
    new Set(lead).size !== lead.length ||
    lead.some((id) => !known.has(id))
  ) {
    throw new Error("invalid tooling shard plan");
  }
  return lead.reduce((sum, id) => sum + toolingTimings().leadSteps[id], 0);
}

// Longest-processing-time packing: heaviest file first onto the least-loaded runner. Ties
// resolve by plan order and runner index, so every runner derives the same partition.
// Runner 1 starts with its lead steps' time; it receives files only once the others catch up.
function pack(plan: ToolingShardPlan, total: number, leadMs: number) {
  const loads = Array.from({ length: total }, (_, index) => (index === 0 ? leadMs : 0));
  const members: number[][] = Array.from({ length: total }, () => []);
  const order = plan.tests
    .map((file, index) => ({ index, ms: estimateToolingTestMs(plan.tier, file) }))
    .sort((left, right) => right.ms - left.ms || left.index - right.index);
  for (const { index, ms } of order) {
    let target = 0;
    for (let shard = 1; shard < total; shard += 1) {
      if (loads[shard] < loads[target]) target = shard;
    }
    loads[target] += ms;
    members[target].push(index);
  }
  return members.map((indexes, shard) => ({
    files: indexes.sort((left, right) => left - right).map((index) => plan.tests[index]),
    testMs: loads[shard] - (shard === 0 ? leadMs : 0),
  }));
}

function assertExactPartition(plan: ToolingShardPlan, shards: readonly { files: string[] }[]) {
  const selected = shards.flatMap(({ files }) => files);
  const planned = new Set(plan.tests);
  if (
    selected.length !== plan.tests.length ||
    new Set(selected).size !== selected.length ||
    selected.some((file) => !planned.has(file))
  ) {
    throw new Error("tooling shards must contain every planned test file exactly once");
  }
}

// Use the fewest runners whose estimated test time stays within the target (or within the
// single heaviest file, which no partition can split), up to the runner cap.
export function toolingShards(plan: ToolingShardPlan) {
  const leadMs = validatePlan(plan);
  const limit = Math.min(MAX_TOOLING_SHARDS, Math.max(1, plan.tests.length));
  const budget = plan.tests.reduce(
    (largest, file) => Math.max(largest, estimateToolingTestMs(plan.tier, file)),
    TARGET_TOOLING_SHARD_MS,
  );
  for (let total = 1; ; total += 1) {
    const shards = pack(plan, total, leadMs);
    if (total === limit || shards.every(({ testMs }) => testMs <= budget)) {
      assertExactPartition(plan, shards);
      return shards;
    }
  }
}

export function toolingShardMatrix(plan: ToolingShardPlan) {
  const shards = toolingShards(plan);
  return {
    include: shards.map(({ files }, index) => {
      // Name the worker after its costliest actual file. Display labels never
      // participate in packing or in the numeric coordinates used by workers.
      const primary = files
        .slice()
        .sort(
          (left, right) =>
            estimateToolingTestMs(plan.tier, right) - estimateToolingTestMs(plan.tier, left),
        )[0];
      const name = primary
        ? primary.replace(/^tests\/tooling\//, "").replace(/\.test\.(?:ts|mjs)$/, "") +
          (files.length > 1 ? ` + ${files.length - 1} files` : "")
        : plan.leadSteps?.length
          ? "runtime captures"
          : "no selected test files";
      return { index: index + 1, total: shards.length, name };
    }),
  };
}

export function selectToolingShard(plan: ToolingShardPlan, shard: unknown = "") {
  if (shard === "") {
    validatePlan(plan);
    return plan.tests;
  }
  const match = typeof shard === "string" ? /^(\d+)\/(\d+)$/.exec(shard) : null;
  const index = Number(match?.[1]);
  const total = Number(match?.[2]);
  const valid =
    match !== null &&
    Number.isSafeInteger(index) &&
    Number.isSafeInteger(total) &&
    index >= 1 &&
    index <= total;
  const shards = valid ? toolingShards(plan) : [];
  if (!valid || total !== shards.length) {
    throw new Error("expected a complete tooling shard index/count from the planned matrix");
  }
  return shards[index - 1].files;
}

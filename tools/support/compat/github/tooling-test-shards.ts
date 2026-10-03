// Partition the selected PR files or the complete merge suite. Actions runners own separate
// checkouts; every runner still executes its own files serially.
export const MAX_TOOLING_SHARDS = 4;

type ToolingShardPlan = { tier: "pr" | "merge"; tests: readonly string[] };

export function toolingShardMatrix(plan: ToolingShardPlan) {
  if (!["pr", "merge"].includes(plan.tier) || !Array.isArray(plan.tests)) {
    throw new Error("invalid tooling shard plan");
  }
  const total = Math.min(MAX_TOOLING_SHARDS, Math.max(1, plan.tests.length));
  return { include: Array.from({ length: total }, (_, index) => ({ index: index + 1, total })) };
}

export function selectToolingShard(plan: ToolingShardPlan, shard: unknown = "") {
  if (shard === "") return plan.tests;
  const match = typeof shard === "string" ? /^(\d+)\/(\d+)$/.exec(shard) : null;
  const index = Number(match?.[1]);
  const total = Number(match?.[2]);
  if (
    !match ||
    !Number.isSafeInteger(index) ||
    !Number.isSafeInteger(total) ||
    index < 1 ||
    index > total ||
    total !== toolingShardMatrix(plan).include.length
  ) {
    throw new Error("expected a complete tooling shard index/count from the planned matrix");
  }
  return plan.tests.filter((_, fileIndex) => fileIndex % total === index - 1);
}

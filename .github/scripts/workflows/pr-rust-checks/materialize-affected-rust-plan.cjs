require("node:fs").writeFileSync(
  process.env.RUNNER_TEMP + "/rust-plan.json",
  process.env.RUST_PLAN_JSON,
);

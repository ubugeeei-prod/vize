import { writeFileSync } from "node:fs";

const [outputPath] = process.argv.slice(2);
if (!outputPath || !process.env.RUST_FAST_PLAN_JSON) {
  throw new Error("missing fast Rust plan or output path");
}
const plan = JSON.parse(process.env.RUST_FAST_PLAN_JSON);
if (
  plan.schemaVersion !== 1 ||
  plan.mode !== "fast" ||
  plan.scope !== "affected" ||
  !Array.isArray(plan.packages) ||
  plan.packages.length === 0 ||
  plan.packages.some((name) => !/^[A-Za-z0-9][A-Za-z0-9_-]*$/.test(name)) ||
  new Set(plan.packages).size !== plan.packages.length ||
  JSON.stringify(plan.cargoArgs) !==
    JSON.stringify(plan.packages.flatMap((name) => ["--package", name]))
) {
  throw new Error("invalid fast Rust source plan");
}
writeFileSync(outputPath, `${JSON.stringify(plan)}\n`);

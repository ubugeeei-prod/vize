import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { fastStructuralTests } from "./plan-tooling-fast-pr.mjs";

const root = fileURLToPath(new URL("../../../../", import.meta.url));

export function fastToolingTestArgs(plan) {
  if (
    plan.version !== 1 ||
    plan.mode !== "fast" ||
    !Array.isArray(plan.tests) ||
    new Set(plan.tests).size !== plan.tests.length ||
    plan.tests.some((file) => !fastStructuralTests.includes(file))
  ) {
    throw new Error("invalid fast PR tooling plan");
  }
  return ["--test", "--test-concurrency=1", ...plan.tests];
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [planPath] = process.argv.slice(2);
  if (process.argv.length !== 3 || !planPath) throw new Error("expected fast PR tooling plan");
  if (process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD !== "1" || !process.env.VIZE_LSP_BIN) {
    throw new Error("fast PR tooling requires a source-built CLI");
  }
  const args = fastToolingTestArgs(JSON.parse(readFileSync(planPath, "utf8")));
  if (args.length === 2) {
    process.stdout.write("No changed structural tooling tests; source-built CLI smoke passed.\n");
  } else {
    const result = spawnSync(process.execPath, args, {
      cwd: root,
      env: { ...process.env, VIZE_TEST_REQUIRE_TSGO: "1" },
      stdio: "inherit",
    });
    if (result.error) throw result.error;
    process.exitCode = result.status ?? 1;
  }
}

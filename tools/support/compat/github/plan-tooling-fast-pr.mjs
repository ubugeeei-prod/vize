import { appendFileSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { changedPaths } from "./plan-source-checks.mjs";

const root = fileURLToPath(new URL("../../../../", import.meta.url));

// These contracts read Rust source and Cargo metadata. Their measured #7132
// execution needs Node and installed packages, but no NAPI, MoonBit, Rust
// Script, fixture submodule, or plugin image. New test files fail to full prep.
export const fastStructuralTests = [
  "tests/tooling/davinci-ssr-l4-bridge.test.ts",
  "tests/tooling/davinci-stage-dependencies.test.ts",
];

function isAuditedFastInput(path) {
  return (
    path === "Cargo.lock" ||
    path.startsWith("crates/") ||
    path.startsWith("docs/davinci/") ||
    fastStructuralTests.includes(path)
  );
}

// Rust/JS/browser PR gates already cover source behavior, and the protected
// merge tier executes every tooling file. The fast PR lane validates the
// source-built CLI plus any directly changed audited structural contracts.
// Unsupported or unknown inputs retain the existing broad PR tooling lane.
export function planToolingFastPr(paths) {
  const fast = paths.length > 0 && paths.every(isAuditedFastInput);
  return {
    version: 1,
    mode: fast ? "fast" : "full",
    tests: fast ? fastStructuralTests.filter((file) => paths.includes(file)) : [],
    reason: fast ? "audited Rust and Davinci structural inputs" : "broad PR tooling fallback",
  };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [base, head, output] = process.argv.slice(2);
  if (
    process.argv.length !== 5 ||
    ![base, head].every((sha) => /^[0-9a-f]{40}$/.test(sha ?? "")) ||
    !output
  ) {
    throw new Error("expected base SHA, head SHA, and output path");
  }
  const paths = /^0+$/.test(base) ? [] : changedPaths(base, head, root);
  const plan = planToolingFastPr(paths);
  mkdirSync(dirname(output), { recursive: true });
  writeFileSync(output, `${JSON.stringify(plan, null, 2)}\n`);
  if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, `mode=${plan.mode}\n`);
  process.stdout.write(`PR tooling: ${plan.mode}; ${plan.tests.length} changed structural tests; ${plan.reason}.\n`);
}

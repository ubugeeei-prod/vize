import { appendFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { changedPaths } from "./plan-source-checks.ts";
import { aggregateCheckNeedsResults } from "./require-needs-success.mjs";

const inputs = [
  /^\.gitmodules$/,
  /^\.github\/(?:workflows|actions)\//,
  /^(?:Cargo\.(?:toml|lock)|rust-toolchain\.toml)$/,
  /^\.cargo\//,
  /^davinci\//,
  /^crates\/vize_(?:atelier_|armature\/|carton\/|croquis\/|relief\/)/,
  /^tests\/_fixtures\//,
  /^tests\/tooling\/(?:davinci-dom-corpus|davinci-canonical-corpus|fixture-)/,
  /^tests\/tooling\/fixtures\/canonical-observer-logs\//,
  /^tests\/tooling\/support\/canonical-corpus-worker-inventory\.mjs$/,
  /^tools\/(?:commands\/fixtures|support\/compat\/fixtures|benchmarks)\//,
  /^tools\/support\/compat\/davinci\/lib\/corpus-baseline-contract\.mjs$/,
  /^tools\/support\/compat\/github\/(?:canonical-corpus-(?:selection|identity|inventory|hydration|observer|workers)|comparison-base|plan-source-checks|require-needs-success|prepare-vue-benchmarks)\.mjs$/,
  /^tools\/support\/compat\/github\/(?:comparison-base|plan-source-checks)\.ts$/,
];

export function canonicalCorpusRequired(paths, eventName) {
  if (!["pull_request", "merge_group"].includes(eventName))
    throw new Error("expected PR or merge-group canonical corpus context");
  if (!Array.isArray(paths)) throw new Error("expected changed paths");
  if (paths.length === 0) return true;
  for (const path of paths) {
    if (
      typeof path !== "string" ||
      path.length === 0 ||
      path.startsWith("/") ||
      path.split("/").some((part) => part === "." || part === "..") ||
      path.includes("\0")
    )
      throw new Error("expected repository-relative changed path");
  }
  return paths.some((path) => inputs.some((pattern) => pattern.test(path)));
}

export function requireCanonicalCorpusSuccess(needs) {
  const plan = needs?.["pr-source-plan"];
  const result = needs?.["canonical-corpus"]?.result;
  if (plan?.result !== "success") throw new Error("canonical corpus plan did not succeed");
  const required = plan.outputs?.["canonical-corpus"];
  if (required !== "true" && required !== "false")
    throw new Error("missing canonical corpus selection");
  if (required === "true" && result !== "success")
    throw new Error(`required canonical corpus did not succeed: ${result}`);
  if (required === "false" && result !== "success" && result !== "skipped")
    throw new Error(`optional canonical corpus did not succeed: ${result}`);
}

export function canonicalSourceReport(needs, eventName) {
  requireCanonicalCorpusSuccess(needs);
  const original = Object.fromEntries(
    Object.entries(needs).filter(([job]) => job !== "canonical-corpus"),
  );
  return aggregateCheckNeedsResults(original, eventName);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const args = process.argv.slice(2);
  if (args.length === 1 && args[0] === "--check") {
    const report = canonicalSourceReport(
      JSON.parse(process.env.NEEDS_JSON ?? "null"),
      process.env.GITHUB_EVENT_NAME,
    );
    process.stdout.write(`${report.message}\n`);
    process.exitCode = report.exitCode;
  } else {
    const [base, head, eventName] = args;
    if (
      args.length !== 3 ||
      !/^[0-9a-f]{40}$/.test(base ?? "") ||
      !/^[0-9a-f]{40}$/.test(head ?? "") ||
      /^0+$/.test(head)
    )
      throw new Error("expected full comparison and candidate identities");
    const paths = /^0+$/.test(base) ? [] : changedPaths(base, head);
    const required = canonicalCorpusRequired(paths, eventName);
    if (process.env.GITHUB_OUTPUT)
      appendFileSync(process.env.GITHUB_OUTPUT, `canonical-corpus=${required}\n`);
    process.stdout.write(`Canonical corpus: ${required}; changed paths: ${paths.length}\n`);
  }
}

// TS-45 results table: `node tests/editor-conformance/report.ts <dir> [--require a,b,…]`
// renders every `<client>.json` in `<dir>` as one Markdown table (also appended
// to $GITHUB_STEP_SUMMARY in CI) and exits non-zero unless every required
// client ran, every step passed, and every corrupted expectation was rejected.
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

import type { Evaluation, Scenario } from "./conformance.ts";
import { loadScenario } from "./support/context.ts";

export type ClientResult = {
  id: string;
  client: string;
  version: string;
  mode: "editor" | "replay";
  seconds: number;
  driverError: string | null;
  evaluation: Evaluation;
  corruptedExpectationsAccepted: string[];
};

export function clientPassed(result: ClientResult): boolean {
  return (
    result.driverError == null &&
    result.evaluation.steps.every((step) => step.pass) &&
    result.corruptedExpectationsAccepted.length === 0
  );
}

export function renderTable(scenario: Scenario, results: ClientResult[]): string {
  const header = results.map((result) => `${result.client} ${result.version} (${result.mode})`);
  const rows = scenario.steps.map((step) => {
    const cells = results.map((result) => {
      const verdict = result.evaluation.steps.find((candidate) => candidate.id === step.id);
      return verdict == null ? "missing" : verdict.pass ? "pass" : "**FAIL**";
    });
    return `| ${step.label} | ${cells.join(" | ")} |`;
  });
  const control = results.map((result) =>
    result.corruptedExpectationsAccepted.length === 0
      ? `rejected ${scenario.steps.length}/${scenario.steps.length}`
      : `**accepted ${result.corruptedExpectationsAccepted.join(", ")}**`,
  );
  const summary = results.map((result) =>
    clientPassed(result) ? "**conformant**" : "**not conformant**",
  );
  return [
    `| ${scenario.suite} step | ${header.join(" | ")} |`,
    `| --- | ${results.map(() => "---").join(" | ")} |`,
    ...rows,
    `| corrupted expectations | ${control.join(" | ")} |`,
    `| JSON-RPC messages judged | ${results.map((result) => result.evaluation.messages).join(" | ")} |`,
    `| wall time | ${results.map((result) => `${result.seconds}s`).join(" | ")} |`,
    `| verdict | ${summary.join(" | ")} |`,
  ].join("\n");
}

function main(argv: string[]): number {
  const dir = path.resolve(argv[0] ?? ".");
  const requireFlag = argv.indexOf("--require");
  const required = requireFlag >= 0 ? argv[requireFlag + 1].split(",") : [];
  const results = fs
    .readdirSync(dir)
    .filter((name) => name.endsWith(".json"))
    .map((name) => JSON.parse(fs.readFileSync(path.join(dir, name), "utf8")) as ClientResult)
    .sort((left, right) => left.id.localeCompare(right.id));
  const scenario = loadScenario();
  const missing = required.filter((id) => !results.some((result) => result.id === id));
  const lines = [
    `### ${scenario.suite} multi-client LSP conformance`,
    "",
    "One scenario, one set of exact expectations, judged from each client's recorded JSON-RPC transcript against the real `vize lsp`. `editor` = the real editor drives the server; `replay` = the client's own initialize parameters and request shapes replayed (no headless mode).",
    "",
    renderTable(scenario, results),
    ...(missing.length > 0 ? ["", `**Missing clients:** ${missing.join(", ")}`] : []),
  ];
  const markdown = `${lines.join("\n")}\n`;
  process.stdout.write(markdown);
  if (process.env.GITHUB_STEP_SUMMARY) fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, markdown);
  return missing.length === 0 && results.length > 0 && results.every(clientPassed) ? 0 : 1;
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.exitCode = main(process.argv.slice(2));
}

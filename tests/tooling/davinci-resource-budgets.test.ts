// TS-44 baselines (Davinci P5-11a): `budgets.toml [resource]` and its gate,
// `tools/commands/davinci/resource-budgets.rs`. The tool is exercised on the
// committed file and on injected mutations: a number without methodology and
// a loosened or dropped ceiling must each fail, and a measurement over a
// ceiling must fail the check.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { parseTomlLite } from "../../tools/support/compat/davinci/toml-lite.mjs";
import { readRepoFile, root as repoRoot } from "./support/github-workflows.ts";

const tool = path.join(repoRoot, "tools/commands/davinci/resource-budgets.rs");
const budgetsText = readRepoFile("davinci-road", "plan", "budgets.toml");
const presetFields = [
  "machine",
  "project",
  "server_build",
  "command",
  "runs",
  "recorded_at",
  "recorded_run",
];
const metricFields = ["baseline", "ceiling", "headroom", "unit", "statistic", "sampler"];

type Metric = {
  baseline: number;
  ceiling: number;
  headroom: number;
  unit: string;
  statistic: string;
  sampler: string;
};
const resource = (
  parseTomlLite(budgetsText) as { resource: Record<string, Record<string, unknown>> }
).resource;
const presets = Object.entries(resource);

function runTool(args: string[]) {
  const result = spawnSync("rust-script", [tool, ...args], { cwd: repoRoot, encoding: "utf8" });
  return { status: result.status, output: `${result.stdout}\n${result.stderr}` };
}

function withBudgets(text: string, body: (file: string) => void) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-p5-11a-"));
  try {
    const file = path.join(dir, "budgets.toml");
    fs.writeFileSync(file, text);
    body(file);
  } finally {
    fs.rmSync(dir, { force: true, recursive: true });
  }
}

/** The committed budgets with one metric line of the first preset rewritten. */
function mutateMetric(key: string, rewrite: (line: string) => string): string {
  const lines = budgetsText.split("\n");
  const at = lines.findIndex((line) => line.startsWith(`${key} = {`));
  assert.ok(at >= 0, `budgets.toml pins ${key}`);
  lines[at] = rewrite(lines[at]);
  return lines.join("\n");
}

const [firstPreset, firstTable] = presets[0] ?? ["", {}];
const firstMetric = Object.keys(firstTable).find((key) => key !== "methodology") ?? "";
const methodologyOf = (table: Record<string, unknown>) =>
  (table.methodology ?? {}) as Record<string, unknown>;

test("[resource] pins numbers with their methodology beside every one", () => {
  assert.ok(presets.length > 0, "[resource] pins at least one machine preset");
  for (const [preset, table] of presets) {
    const methodology = methodologyOf(table);
    for (const field of presetFields) {
      assert.ok(methodology[field] != null, `${preset} records ${field}`);
    }
    assert.equal(typeof methodology.runs, "number", `${preset}.runs is a count`);
    assert.match(
      String(methodology.recorded_run),
      /^https:\/\/github\.com\/ubugeeei-prod\/vize\/actions\/runs\/\d+$/u,
    );
    const metrics = Object.entries(table).filter(([key]) => key !== "methodology");
    assert.ok(metrics.length >= 4, `${preset} pins the contract's metrics`);
    for (const [key, value] of metrics) {
      const metric = value as Metric;
      for (const field of metricFields)
        assert.ok(metric[field as keyof Metric] != null, `${preset}.${key}.${field}`);
      for (const field of ["baseline", "ceiling", "headroom"] as const) {
        assert.equal(typeof metric[field], "number", `${preset}.${key}.${field} is a number`);
      }
      assert.ok(metric.baseline <= metric.ceiling, `${preset}.${key}: baseline <= ceiling`);
      assert.ok(
        metric.ceiling <= Math.ceil(metric.baseline * (1 + metric.headroom)),
        `${preset}.${key}: ceiling within headroom`,
      );
    }
  }
});

test("the recorded command is the one CI measures with", () => {
  const workflow = parse(
    readRepoFile(".github", "workflows", "davinci-resource-budgets.yml"),
  ) as any;
  assert.equal(Object.hasOwn(workflow.on, "push"), false);
  assert.ok(workflow.on.schedule);
  assert.equal(Object.hasOwn(workflow.on, "workflow_dispatch"), true);
  const steps = workflow.jobs.resource.steps as Array<{ name?: string; run?: string }>;
  const measure = steps.find((step) => step.name === "Measure the resident server")!.run!;
  for (const [preset, table] of presets) {
    const recorded = String(methodologyOf(table).command);
    assert.equal(workflow.env.RESOURCE_PRESET, preset);
    for (const flag of ["--measure", "--runs", "--keystrokes", "--idle-seconds"]) {
      const pick = (text: string) => new RegExp(`${flag}(?: (\\S+))?`, "u").exec(text)?.[0];
      assert.equal(pick(recorded), pick(measure), `${preset}: ${flag} matches CI`);
    }
    assert.equal(Number(/--runs (\d+)/u.exec(measure)![1]), methodologyOf(table).runs);
  }
  const names = steps.map((step) => step.name);
  for (const name of [
    "Every number carries its methodology",
    "No ceiling is loosened against the base",
    "The measurement holds every pinned ceiling",
  ]) {
    assert.ok(names.indexOf(name) > names.indexOf("Measure the resident server"), name);
  }
});

test("the committed budgets validate", () => {
  const result = runTool(["--validate"]);
  assert.equal(result.status, 0, result.output);
});

test("a number without methodology is rejected", () => {
  withBudgets(
    mutateMetric(firstMetric, () => `${firstMetric} = 1`),
    (file) => {
      const result = runTool(["--validate", "--budgets", file]);
      assert.notEqual(result.status, 0);
      assert.match(
        result.output,
        new RegExp(`${firstPreset}\\.${firstMetric}: a bare number has no methodology`, "u"),
      );
    },
  );
  withBudgets(budgetsText.replace(/^methodology = .*\n/mu, ""), (file) => {
    const result = runTool(["--validate", "--budgets", file]);
    assert.notEqual(result.status, 0);
    assert.match(
      result.output,
      new RegExp(`${firstPreset}: missing the \`methodology\` table`, "u"),
    );
  });
  withBudgets(
    mutateMetric(firstMetric, (line) => line.replace(/, sampler = "[^"]*"/u, "")),
    (file) => {
      const result = runTool(["--validate", "--budgets", file]);
      assert.notEqual(result.status, 0);
      assert.match(
        result.output,
        new RegExp(`${firstMetric}: missing methodology field \`sampler\``, "u"),
      );
    },
  );
});

test("an injected loosening fails the ratchet; a tightening passes", () => {
  const ceiling = (resource[firstPreset][firstMetric] as Metric).ceiling;
  const base = path.join(repoRoot, "davinci-road/plan/budgets.toml");
  withBudgets(
    mutateMetric(firstMetric, (line) =>
      line.replace(/ceiling = [\d.]+/u, `ceiling = ${ceiling + 1}`),
    ),
    (file) => {
      const result = runTool(["--ratchet", base, "--budgets", file]);
      assert.notEqual(result.status, 0);
      assert.match(
        result.output,
        new RegExp(`${firstMetric}\\.ceiling: loosened from ${ceiling} to ${ceiling + 1}`, "u"),
      );
    },
  );
  withBudgets(budgetsText.replace(new RegExp(`^${firstMetric} = .*\\n`, "mu"), ""), (file) => {
    const result = runTool(["--ratchet", base, "--budgets", file]);
    assert.notEqual(result.status, 0);
    assert.match(result.output, new RegExp(`${firstMetric}: pinned metric removed`, "u"));
  });
  withBudgets(
    mutateMetric(firstMetric, (line) =>
      line.replace(/ceiling = [\d.]+/u, `ceiling = ${Math.max(0, ceiling - 1)}`),
    ),
    (file) => {
      assert.equal(runTool(["--ratchet", base, "--budgets", file]).status, 0);
    },
  );
});

test("a measurement over a ceiling, or missing a pinned metric, fails the check", () => {
  const measurement = (over: string | null, drop: string | null) => ({
    preset: firstPreset,
    metrics: Object.fromEntries(
      Object.entries(resource[firstPreset])
        .filter(([key]) => key !== "methodology" && key !== drop)
        .map(([key, value]) => [
          key,
          { value: (value as Metric).ceiling + (key === over ? 1 : 0) },
        ]),
    ),
  });
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-p5-11a-"));
  try {
    const write = (name: string, value: object) => {
      fs.writeFileSync(path.join(dir, name), JSON.stringify(value));
      return path.join(dir, name);
    };
    assert.equal(
      runTool(["--check", "--measurement", write("at.json", measurement(null, null))]).status,
      0,
    );
    const over = runTool([
      "--check",
      "--measurement",
      write("over.json", measurement(firstMetric, null)),
    ]);
    assert.notEqual(over.status, 0);
    assert.match(over.output, new RegExp(`${firstMetric}: measured .* exceeds the ceiling`, "u"));
    const missing = runTool([
      "--check",
      "--measurement",
      write("missing.json", measurement(null, firstMetric)),
    ]);
    assert.notEqual(missing.status, 0);
    assert.match(missing.output, new RegExp(`${firstMetric}: pinned but not measured`, "u"));
  } finally {
    fs.rmSync(dir, { force: true, recursive: true });
  }
});

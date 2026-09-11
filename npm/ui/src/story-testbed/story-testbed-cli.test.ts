import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { test } from "vite-plus/test";

import { runUiStoryTestbedCli } from "../../scripts/story-testbed.ts";
import { uiFamilyCatalog } from "../catalog/family-catalog.ts";
import {
  UI_STORY_TESTBED_SCHEMA_VERSION,
  uiStoryTestbedBrowserSuiteNames,
  uiStoryTestbedHarnessHookNames,
  uiStoryTestbedSurfaces,
  type UiStoryTestbedEntry,
  type UiStoryTestbedFamilySummary,
  type UiStoryTestbedPlanItem,
} from "./story-testbed.ts";

interface ListJsonOutput {
  readonly schemaVersion: number;
  readonly command: "list";
  readonly familyCount: number;
  readonly families: readonly UiStoryTestbedFamilySummary[];
}

interface PlanJsonOutput {
  readonly schemaVersion: number;
  readonly command: "plan";
  readonly surface: string | null;
  readonly planCount: number;
  readonly plan: readonly UiStoryTestbedPlanItem[];
}

interface InfoJsonOutput {
  readonly schemaVersion: number;
  readonly command: "info";
  readonly family: UiStoryTestbedEntry;
}

interface CheckJsonOutput {
  readonly schemaVersion: number;
  readonly command: "check";
  readonly familyCount: number;
  readonly planCount: number;
  readonly surfaces: readonly string[];
  readonly browserSuites: readonly string[];
  readonly harnessHooks: readonly string[];
  readonly violationCount: number;
}

const uiRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const repoRoot = path.resolve(uiRoot, "../..");

function runCli(args: readonly string[]) {
  let stdout = "";
  let stderr = "";
  const exitCode = runUiStoryTestbedCli(args, {
    stdout: {
      write(chunk) {
        stdout += chunk;
      },
    },
    stderr: {
      write(chunk) {
        stderr += chunk;
      },
    },
  });

  return { exitCode, stdout, stderr };
}

function parseJson<Output>(source: string): Output {
  return JSON.parse(source) as Output;
}

test("CLI emits deterministic machine-readable story-testbed output", () => {
  const helpOutput = runCli(["--help"]);
  assert.equal(helpOutput.exitCode, 0);
  assert.match(helpOutput.stdout, /repository checkout only/);
  assert.match(helpOutput.stdout, /not a published @vizejs\/ui package bin/);
  assert.match(helpOutput.stdout, /issue #4898/);

  const checkOutput = runCli(["check", "--format", "json"]);
  assert.equal(checkOutput.exitCode, 0);
  assert.equal(checkOutput.stderr, "");
  assert.ok(checkOutput.stdout.length < 2_000, "check output must stay compact for CI logs");
  const checked = parseJson<CheckJsonOutput>(checkOutput.stdout);
  assert.equal(checked.schemaVersion, UI_STORY_TESTBED_SCHEMA_VERSION);
  assert.equal(checked.command, "check");
  assert.equal(checked.familyCount, uiFamilyCatalog.length);
  assert.equal(checked.planCount, uiFamilyCatalog.length * uiStoryTestbedSurfaces.length);
  assert.deepEqual(checked.surfaces, uiStoryTestbedSurfaces);
  assert.deepEqual(checked.browserSuites, uiStoryTestbedBrowserSuiteNames);
  assert.deepEqual(checked.harnessHooks, uiStoryTestbedHarnessHookNames);
  assert.equal(checked.violationCount, 0);

  const badFormatOutput = runCli(["check", "--format=yaml"]);
  assert.equal(badFormatOutput.exitCode, 1);
  assert.match(badFormatOutput.stderr, /Unsupported output format "yaml"/);
  assert.doesNotMatch(badFormatOutput.stderr, /Unsupported output format "--format=yaml"/);

  const listOutput = runCli(["list", "--format", "json"]);
  assert.equal(listOutput.exitCode, 0);
  assert.equal(listOutput.stderr, "");
  const listed = parseJson<ListJsonOutput>(listOutput.stdout);
  assert.equal(listed.schemaVersion, UI_STORY_TESTBED_SCHEMA_VERSION);
  assert.equal(listed.command, "list");
  assert.equal(listed.familyCount, uiFamilyCatalog.length);
  assert.deepEqual(
    listed.families.map((family) => family.canonicalName),
    uiFamilyCatalog.map((entry) => entry.canonicalName),
  );

  const planOutput = runCli(["plan", "vitest-browser"]);
  assert.equal(planOutput.exitCode, 0);
  assert.equal(planOutput.stderr, "");
  const plan = parseJson<PlanJsonOutput>(planOutput.stdout);
  assert.equal(plan.schemaVersion, UI_STORY_TESTBED_SCHEMA_VERSION);
  assert.equal(plan.command, "plan");
  assert.equal(plan.surface, "vitest-browser");
  assert.equal(plan.planCount, uiFamilyCatalog.length);
  assert.ok(plan.plan.every((item) => item.runner === "vitest-browser"));

  const shorthandPlanOutput = runCli(["plan", "--surface=playwright-vrt", "--jsonl"]);
  assert.equal(shorthandPlanOutput.exitCode, 0);
  assert.equal(shorthandPlanOutput.stderr, "");
  const planLines = shorthandPlanOutput.stdout.trim().split("\n");
  assert.equal(planLines.length, uiFamilyCatalog.length);
  assert.ok(
    planLines.every(
      (line) => parseJson<{ readonly surface: string }>(line).surface === "playwright-vrt",
    ),
  );

  const infoOutput = runCli(["info", "./button"]);
  assert.equal(infoOutput.exitCode, 0);
  assert.equal(infoOutput.stderr, "");
  const info = parseJson<InfoJsonOutput>(infoOutput.stdout);
  assert.equal(info.schemaVersion, UI_STORY_TESTBED_SCHEMA_VERSION);
  assert.equal(info.command, "info");
  assert.equal(info.family.canonicalName, "button");
  assert.equal(info.family.storyFile, "src/families/actions/button/button.art.vue");
});

test("CLI check resolves source files when invoked from the repository root", () => {
  const stdout = execFileSync(
    process.execPath,
    ["npm/ui/scripts/story-testbed.ts", "check", "--format", "json"],
    { cwd: repoRoot, encoding: "utf8" },
  );
  const checked = parseJson<CheckJsonOutput>(stdout);

  assert.equal(checked.violationCount, 0);
});

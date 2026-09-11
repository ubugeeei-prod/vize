import { readdirSync } from "node:fs";
import path from "node:path";

import {
  UI_STORY_TESTBED_SCHEMA_VERSION,
  auditUiStoryTestbedInventory,
  createUiStoryTestbedManifest,
  formatUiStoryTestbedViolations,
  getUiStoryTestbedFamilyInfo,
  listUiStoryTestbedFamilies,
  listUiStoryTestbedPlan,
  uiStoryTestbedBrowserSuiteNames,
  uiStoryTestbedHarnessHookNames,
  uiStoryTestbedScreenshotReview,
  type UiStoryTestbedEntry,
  type UiStoryTestbedFamilySummary,
  type UiStoryTestbedPlanItem,
  type UiStoryTestbedSurface,
  type UiStoryTestbedViolation,
} from "../src/story-testbed/story-testbed.ts";
import {
  parseStoryTestbedArgs,
  parseStoryTestbedSurface,
  type StoryTestbedOutputFormat,
} from "./story-testbed-args.ts";

interface WritableOutput {
  write(chunk: string): void;
}

interface CliIo {
  readonly stdout: WritableOutput;
  readonly stderr: WritableOutput;
}

interface CheckJsonOutput {
  readonly schemaVersion: typeof UI_STORY_TESTBED_SCHEMA_VERSION;
  readonly command: "check";
  readonly familyCount: number;
  readonly planCount: number;
  readonly surfaces: readonly UiStoryTestbedSurface[];
  readonly browserSuites: readonly string[];
  readonly harnessHooks: readonly string[];
  readonly screenshotReview: typeof uiStoryTestbedScreenshotReview;
  readonly violationCount: number;
}

type CliRecord =
  | CheckJsonOutput
  | {
      readonly schemaVersion: typeof UI_STORY_TESTBED_SCHEMA_VERSION;
      readonly command: "list";
      readonly family: UiStoryTestbedFamilySummary;
    }
  | {
      readonly schemaVersion: typeof UI_STORY_TESTBED_SCHEMA_VERSION;
      readonly command: "plan";
      readonly surface: UiStoryTestbedSurface;
      readonly item: UiStoryTestbedPlanItem;
    }
  | {
      readonly schemaVersion: typeof UI_STORY_TESTBED_SCHEMA_VERSION;
      readonly command: "info";
      readonly family: UiStoryTestbedEntry;
    };

const usage = `Usage (repository checkout only; not a published @vizejs/ui package bin):
  cd npm/ui
  node scripts/story-testbed.ts check [--format json|jsonl]
  node scripts/story-testbed.ts list [--format json|jsonl]
  node scripts/story-testbed.ts plan [surface] [--surface <surface>] [--format json|jsonl]
  node scripts/story-testbed.ts info <name-or-subpath> [--format json|jsonl]

Surfaces: musea-story, vue-test-utils, vitest-browser, playwright-vrt.

This CLI imports package source files from the checkout. Commands are read-only and describe the Musea, Vue Test Utils, Vitest browser, and Playwright VRT harness plan tracked by issue #4898.
`;

const mutatingCommands = new Set([
  "init",
  "add",
  "add-many",
  "remove",
  "diff",
  "update",
  "doctor",
  "audit",
  "approve",
]);

function collectSourceFiles(directory: string, relativeDirectory = "src"): ReadonlySet<string> {
  const files: string[] = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const entryPath = path.join(directory, entry.name);
    const relativePath = path.posix.join(relativeDirectory, entry.name);
    if (entry.isDirectory()) {
      files.push(...collectSourceFiles(entryPath, relativePath));
      continue;
    }
    if (entry.isFile()) files.push(relativePath);
  }

  return new Set(files.sort((left, right) => left.localeCompare(right)));
}

function writeJson(output: WritableOutput, value: unknown): void {
  output.write(`${JSON.stringify(value, null, 2)}\n`);
}

function writeJsonl(output: WritableOutput, records: readonly CliRecord[]): void {
  for (const record of records) output.write(`${JSON.stringify(record)}\n`);
}

function writeError(io: CliIo, message: string): number {
  io.stderr.write(`${message}\n\n${usage}`);
  return 1;
}

function writeViolations(io: CliIo, violations: readonly UiStoryTestbedViolation[]): void {
  const formatted = formatUiStoryTestbedViolations(violations);
  if (formatted.length > 0) io.stderr.write(`${formatted}\n`);
}

function queryFromArgs(args: readonly string[]): string {
  return args.join(" ").trim();
}

function writeRecords(
  format: StoryTestbedOutputFormat,
  io: CliIo,
  records: readonly CliRecord[],
  value: unknown,
): void {
  if (format === "jsonl") writeJsonl(io.stdout, records);
  else writeJson(io.stdout, value);
}

function runCheckCommand(format: StoryTestbedOutputFormat, io: CliIo): number {
  const manifest = createUiStoryTestbedManifest();
  const plan = listUiStoryTestbedPlan({}, manifest);
  const existingFiles = collectSourceFiles(path.resolve("src"));
  const violations = auditUiStoryTestbedInventory(manifest.families, { existingFiles });
  const output: CheckJsonOutput = {
    schemaVersion: UI_STORY_TESTBED_SCHEMA_VERSION,
    command: "check",
    familyCount: manifest.families.length,
    planCount: plan.length,
    surfaces: manifest.surfaces,
    browserSuites: uiStoryTestbedBrowserSuiteNames,
    harnessHooks: uiStoryTestbedHarnessHookNames,
    screenshotReview: uiStoryTestbedScreenshotReview,
    violationCount: violations.length,
  };

  writeRecords(format, io, [output], output);
  writeViolations(io, violations);
  return violations.length === 0 ? 0 : 1;
}

function runListCommand(format: StoryTestbedOutputFormat, io: CliIo): number {
  const families = listUiStoryTestbedFamilies();
  writeRecords(
    format,
    io,
    families.map((family) => ({
      schemaVersion: UI_STORY_TESTBED_SCHEMA_VERSION,
      command: "list",
      family,
    })),
    {
      schemaVersion: UI_STORY_TESTBED_SCHEMA_VERSION,
      command: "list",
      familyCount: families.length,
      families,
    },
  );
  return 0;
}

function runPlanCommand(
  args: readonly string[],
  surface: UiStoryTestbedSurface | null,
  format: StoryTestbedOutputFormat,
  io: CliIo,
): number {
  if (args.length > 2) return writeError(io, "The plan command accepts at most one surface");

  const positionalSurface = args[1];
  const selectedSurface =
    positionalSurface == null ? surface : parseStoryTestbedSurface(positionalSurface);
  if (positionalSurface != null && selectedSurface == null) {
    return writeError(io, `Unsupported story-testbed surface "${positionalSurface}"`);
  }
  if (surface != null && positionalSurface != null && surface !== selectedSurface) {
    return writeError(io, "The positional surface and --surface option disagree");
  }

  const plan = listUiStoryTestbedPlan(selectedSurface == null ? {} : { surface: selectedSurface });
  writeRecords(
    format,
    io,
    plan.map((item) => ({
      schemaVersion: UI_STORY_TESTBED_SCHEMA_VERSION,
      command: "plan",
      surface: item.surface,
      item,
    })),
    {
      schemaVersion: UI_STORY_TESTBED_SCHEMA_VERSION,
      command: "plan",
      surface: selectedSurface,
      planCount: plan.length,
      plan,
    },
  );
  return 0;
}

function runInfoCommand(
  args: readonly string[],
  surface: UiStoryTestbedSurface | null,
  format: StoryTestbedOutputFormat,
  io: CliIo,
): number {
  if (surface != null) return writeError(io, "The info command does not accept --surface");

  const target = queryFromArgs(args.slice(1));
  if (target.length === 0) return writeError(io, "The info command requires a family name");

  const family = getUiStoryTestbedFamilyInfo(target);
  if (family == null) return writeError(io, `Unknown UI story-testbed family "${target}"`);

  writeRecords(
    format,
    io,
    [{ schemaVersion: UI_STORY_TESTBED_SCHEMA_VERSION, command: "info", family }],
    { schemaVersion: UI_STORY_TESTBED_SCHEMA_VERSION, command: "info", family },
  );
  return 0;
}

export function runUiStoryTestbedCli(
  args: readonly string[],
  io: CliIo = { stdout: process.stdout, stderr: process.stderr },
): number {
  const parsed = parseStoryTestbedArgs(args);
  if (parsed.help) {
    io.stdout.write(usage);
    return 0;
  }
  if (parsed.error != null) return writeError(io, parsed.error);

  const command = parsed.positional[0];
  if (command == null) {
    io.stdout.write(usage);
    return 0;
  }
  if (mutatingCommands.has(command)) {
    return writeError(
      io,
      `Command "${command}" is read-only in this story-testbed slice; family story content remains tracked by issue #4898 follow-up PRs.`,
    );
  }

  if (command === "check") {
    if (parsed.positional.length !== 1)
      return writeError(io, "The check command does not accept a query");
    if (parsed.surface != null)
      return writeError(io, "The check command does not accept --surface");
    return runCheckCommand(parsed.format, io);
  }
  if (command === "list") {
    if (parsed.positional.length !== 1)
      return writeError(io, "The list command does not accept a query");
    if (parsed.surface != null) return writeError(io, "The list command does not accept --surface");
    return runListCommand(parsed.format, io);
  }
  if (command === "plan") {
    return runPlanCommand(parsed.positional, parsed.surface, parsed.format, io);
  }
  if (command === "info") {
    return runInfoCommand(parsed.positional, parsed.surface, parsed.format, io);
  }

  return writeError(io, `Unsupported command "${command}"`);
}

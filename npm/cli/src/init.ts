import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import { atomicWriteFile, requireSharedConfig } from "./setup/config.js";
import { initHelp, parseInitArgs, type InitArgs } from "./init/args.js";
import { detectProject, withFramework, type ProjectDetection } from "./init/detect.js";
import { failedInstallMessage } from "./init/install-failure.js";
import { planInit, type InitCommand, type InitPlan } from "./init/plan.js";
import {
  confirm,
  createPromptDeps,
  isNonInteractive,
  selectFeatures,
  type PromptDeps,
} from "./init/prompt.js";
import {
  renderBlocked,
  renderCancelled,
  renderDetection,
  renderEditors,
  renderNonInteractiveRefusal,
  renderPlan,
} from "./init/report.js";
import { defaultSelection, offerFeatures, type FeatureSelection } from "./init/select.js";

export type { InitCommand, InitPlan } from "./init/plan.js";
export type { ProjectDetection } from "./init/detect.js";

/** Injectable dependencies and host controls for project initialization. */
export interface InitOptions {
  readonly root: string;
  /** Command-line arguments to parse. @default [] */
  readonly args?: readonly string[];
  /** Command executor used for planned dependency operations. @default built-in synchronous runner */
  readonly runCommand?: (command: InitCommand) => void;
  /** File writer used for generated configuration. @default atomic project writer */
  readonly writeFile?: (filename: string, source: string) => void;
  /** Output sink for detection, plan, and completion reports. @default process.stdout.write */
  readonly output?: (chunk: string) => void;
  /** Prompt implementation supplied by an embedding host. @default interactive terminal prompts */
  readonly promptDeps?: PromptDeps;
  /** Input stream used to determine interactivity and collect answers. @default process.stdin */
  readonly stdin?: NodeJS.ReadableStream;
}

/**
 * Resolves the feature selection from detection, flags, and -- when the terminal
 * allows it -- the user.
 *
 * A non-TTY stdin without `--yes` returns `null`: refusing is the only correct
 * answer, because prompting would hang a CI job forever.
 */
export async function resolveSelection(
  detection: ProjectDetection,
  args: InitArgs,
  deps: {
    readonly output: (chunk: string) => void;
    readonly stdin: NodeJS.ReadableStream;
    readonly promptDeps?: PromptDeps;
  },
): Promise<FeatureSelection | null> {
  const offers = offerFeatures(detection);
  const base = defaultSelection(offers);
  // An explicit flag survives even when detection says the feature is
  // unavailable: the planner then reports it as blocked, with the reason. That
  // is more useful than dropping the flag the user typed.
  const withOverrides: Record<string, boolean> = { ...base };
  for (const [id, enabled] of Object.entries(args.overrides)) {
    withOverrides[id] = enabled;
  }
  const selection = withOverrides as FeatureSelection;

  if (args.yes) {
    return selection;
  }
  if (deps.promptDeps === undefined && isNonInteractive(deps.stdin)) {
    deps.output(renderNonInteractiveRefusal());
    return null;
  }
  // Only a prompt this function created may be closed here; an injected one
  // belongs to the caller.
  const owned =
    deps.promptDeps === undefined
      ? createPromptDeps({ input: deps.stdin, output: process.stdout as NodeJS.WritableStream })
      : null;
  const promptDeps = deps.promptDeps ?? owned!;
  try {
    const chosen = await selectFeatures(offers, selection, promptDeps);
    if (chosen !== null && (await confirm("Apply this selection?", promptDeps))) {
      return chosen;
    }
    deps.output(renderCancelled());
    return null;
  } finally {
    owned?.close?.();
  }
}

/**
 * Runs `init` end to end.
 *
 * Detection is reported before anything is decided, the plan is reported before
 * anything is written, and a blocked feature is reported as NOT configured
 * rather than quietly downgraded.
 */
export async function initProject(options: InitOptions): Promise<InitPlan | null> {
  const args = parseInitArgs(options.args ?? []);
  const output = options.output ?? ((chunk: string) => process.stdout.write(chunk));
  const root = path.resolve(args.root ?? options.root);
  const detection = withFramework(detectProject(root), args.bundlerOverride);
  output(renderDetection(detection));
  requireSharedConfig(root, detection.viteConfigs);

  const selection = await resolveSelection(detection, args, {
    output,
    stdin: options.stdin ?? process.stdin,
    promptDeps: options.promptDeps,
  });
  if (selection === null) {
    return null;
  }

  const plan = planInit({
    detection,
    selection,
    install: args.install,
    packageManager: args.packageManager ?? undefined,
  });
  output(renderPlan(plan, args.dryRun));
  output(renderBlocked(plan));
  if (args.dryRun) {
    return plan;
  }

  writePlannedFiles(plan, options.writeFile ?? writeProjectFile);
  const runCommand = options.runCommand ?? runInitCommand;
  for (const command of plan.commands) {
    runCommand(command);
  }
  if (selection.editor) {
    output(renderEditors());
  }
  return plan;
}

/** Runs the `init` command and maps refusal or blocked features to a failing exit code. */
export async function runInitCli(args: readonly string[]): Promise<void> {
  if (parseInitArgs(args).help) {
    process.stdout.write(initHelp());
    return;
  }
  const plan = await initProject({ root: process.cwd(), args });
  if (plan === null) {
    process.exitCode = 1;
    return;
  }
  if (plan.features.some((feature) => feature.outcome === "blocked")) {
    process.exitCode = 1;
  }
}

/**
 * Default writer.
 *
 * Creates the parent directory first so `.vscode/extensions.json` works in a
 * project that has never had a `.vscode` folder, then reuses `setup`'s atomic
 * write so a crash mid-run cannot leave a half-written config behind.
 */
function writeProjectFile(filename: string, source: string): void {
  fs.mkdirSync(path.dirname(filename), { recursive: true });
  atomicWriteFile(filename, source);
}

function writePlannedFiles(
  plan: InitPlan,
  writeFile: (filename: string, source: string) => void,
): void {
  const written: string[] = [];
  for (const file of plan.files) {
    try {
      writeFile(file.filename, file.source);
    } catch (error) {
      if (written.length === 0) {
        throw error;
      }
      throw new Error(
        `init partially completed: wrote ${written.join(", ")} before ` +
          `${path.relative(plan.root, file.filename)} failed. Run init again to finish.`,
        { cause: error },
      );
    }
    written.push(path.relative(plan.root, file.filename));
  }
}

function runInitCommand(command: InitCommand): void {
  const isWindowsBatch = process.platform === "win32" && /\.(cmd|bat)$/iu.test(command.command);
  const result = spawnSync(command.command, [...command.args], {
    cwd: command.cwd,
    encoding: "utf8",
    shell: isWindowsBatch,
  });
  if (result.error != null) {
    throw new Error(
      failedInstallMessage(command.command, command.args, result.error.message, "spawn-error"),
      { cause: result.error },
    );
  }
  if (result.status === 0 && result.signal === null) {
    if (result.stdout) process.stdout.write(result.stdout);
    if (result.stderr) process.stderr.write(result.stderr);
    return;
  }
  throw new Error(
    failedInstallMessage(
      command.command,
      command.args,
      `${result.stderr ?? ""}\n${result.stdout ?? ""}`,
      result.signal ?? result.status,
    ),
  );
}

import fs from "node:fs";
import path from "node:path";

import type { ProjectDetection } from "./detect.js";
import { resolveLintTarget } from "./lint-target.js";
import { planBundler } from "./plan-bundler.js";
import { planEditorFile } from "./plan-editor.js";
import { planLint } from "./plan-lint.js";
import { planScripts, planVizeConfig } from "./plan-project.js";
import {
  createPlanDraft,
  skipped,
  type FeatureResult,
  type InitCommand,
  type InitPlan,
  type PlanInitOptions,
} from "./plan-types.js";
import type { FeatureId } from "./select.js";

export type {
  FeatureOutcome,
  FeatureResult,
  InitCommand,
  InitPlan,
  PlanInitOptions,
} from "./plan-types.js";

/** Dev dependencies each feature needs. */
const FEATURE_DEPENDENCIES: Readonly<Record<FeatureId, readonly string[]>> = {
  lint: ["oxlint", "oxlint-plugin-vize"],
  bundler: [],
  fmt: ["vize"],
  typecheck: ["vize"],
  editor: [],
};

const INSTALL_ARGS: Readonly<Record<string, readonly string[]>> = {
  pnpm: ["add", "-D"],
  yarn: ["add", "-D"],
  bun: ["add", "-D"],
  npm: ["install", "-D"],
  vp: ["add", "-D"],
};

const FEATURE_ORDER: readonly FeatureId[] = ["lint", "bundler", "fmt", "typecheck", "editor"];

/**
 * Builds the full plan without touching the filesystem.
 *
 * Planning is separated from execution so `--dry-run`, the interactive
 * confirmation and the tests all inspect the same object the writer consumes;
 * a plan that is correct in `--dry-run` and wrong on disk is not possible.
 */
export function planInit(options: PlanInitOptions): InitPlan {
  const { detection, selection } = options;
  const draft = createPlanDraft();

  const viteSource = readSingleViteConfig(detection);
  const lintTarget = resolveLintTarget({ detection, viteSource });

  let viteDraft = viteSource;
  if (selection.lint) {
    viteDraft = planLint(detection, lintTarget, viteDraft, draft);
    addAll(draft.dependencies, FEATURE_DEPENDENCIES.lint);
  } else {
    draft.features.push(skipped("lint"));
  }

  if (selection.bundler) {
    viteDraft = planBundler(detection, viteDraft, draft);
  } else {
    draft.features.push(skipped("bundler"));
  }

  // One write for the Vite config, however many features touched it, so the
  // plugin edit and the lint edit cannot overwrite one another.
  if (viteSource !== null && viteDraft !== null && viteDraft !== viteSource) {
    const filename = detection.viteConfigs[0]!;
    draft.files.push({ filename: path.join(detection.root, filename), source: viteDraft });
    draft.updatedFiles.push(filename);
  }

  planVizeConfig(detection, selection, draft);
  for (const id of ["fmt", "typecheck"] as const) {
    if (selection[id]) {
      addAll(draft.dependencies, FEATURE_DEPENDENCIES[id]);
    }
  }

  draft.features.push(
    selection.editor
      ? planEditorFile(detection, draft.files, draft.createdFiles, draft.updatedFiles)
      : skipped("editor"),
  );

  const addedScripts = planScripts(detection, selection, draft);
  return {
    root: detection.root,
    detection,
    lintTarget,
    features: sortFeatures(draft.features),
    files: draft.files,
    createdFiles: draft.createdFiles,
    updatedFiles: draft.updatedFiles,
    addedScripts,
    commands: planCommands(detection, draft.dependencies, options),
  };
}

/**
 * The install commands, as a list so callers can assert on them.
 *
 * Exactly one command is emitted, or none when every dependency is already
 * declared -- which is what makes a second `init` run a no-op.
 */
function planCommands(
  detection: ProjectDetection,
  dependencies: ReadonlySet<string>,
  options: PlanInitOptions,
): readonly InitCommand[] {
  if (!options.install) {
    return [];
  }
  const missing = [...dependencies].filter((name) => !detection.dependencies.has(name)).sort();
  if (missing.length === 0) {
    return [];
  }
  const command = resolveInstaller(detection, options.packageManager);
  return [{ command, args: [...INSTALL_ARGS[command]!, ...missing], cwd: detection.root }];
}

/**
 * Installer used for the one install command.
 *
 * A Vite+ project gets `vp add`, matching `setup` and the project's own
 * workflow. Otherwise the package manager comes from the same lockfile rules
 * `detect_package_manager` uses on the Rust side, defaulting to npm when
 * nothing identifies one.
 */
export function resolveInstaller(detection: ProjectDetection, override?: string): string {
  if (override !== undefined) {
    return override;
  }
  if (detection.usesVitePlus) {
    return "vp";
  }
  return detection.packageManager ?? "npm";
}

function readSingleViteConfig(detection: ProjectDetection): string | null {
  if (detection.viteConfigs.length !== 1) {
    return null;
  }
  return fs.readFileSync(path.join(detection.root, detection.viteConfigs[0]!), "utf8");
}

function addAll(target: Set<string>, values: readonly string[]): void {
  for (const value of values) {
    target.add(value);
  }
}

function sortFeatures(features: readonly FeatureResult[]): readonly FeatureResult[] {
  return [...features].sort(
    (left, right) => FEATURE_ORDER.indexOf(left.id) - FEATURE_ORDER.indexOf(right.id),
  );
}

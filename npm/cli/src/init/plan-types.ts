import type { PlannedFile } from "../setup/config.js";
import type { ProjectDetection } from "./detect.js";
import type { LintTarget } from "./lint-target.js";
import type { FeatureId, FeatureSelection } from "./select.js";

/**
 * Shared plan vocabulary.
 *
 * Lives apart from `plan.ts` so the per-feature planners can name these types
 * without importing the orchestrator that calls them.
 */

export type FeatureOutcome =
  /** The feature was selected and something was written for it. */
  | "configured"
  /** The feature was selected and is already wired up. A re-run lands here. */
  | "unchanged"
  /** The feature was not selected, or the project cannot support it. */
  | "skipped"
  /** Selected, but a user-owned file has to be edited by hand. Nothing written. */
  | "blocked";

export interface FeatureResult {
  readonly id: FeatureId;
  readonly outcome: FeatureOutcome;
  readonly detail: string;
  /** Snippet the user must paste when `outcome` is `blocked`. */
  readonly snippet: string | null;
}

export interface InitCommand {
  readonly command: string;
  readonly args: readonly string[];
  readonly cwd: string;
}

export interface InitPlan {
  readonly root: string;
  readonly detection: ProjectDetection;
  readonly lintTarget: LintTarget;
  readonly features: readonly FeatureResult[];
  readonly files: readonly PlannedFile[];
  readonly createdFiles: readonly string[];
  readonly updatedFiles: readonly string[];
  readonly addedScripts: readonly string[];
  readonly commands: readonly InitCommand[];
}

export interface PlanInitOptions {
  readonly detection: ProjectDetection;
  readonly selection: FeatureSelection;
  readonly install: boolean;
  readonly packageManager?: string;
}

/** Mutable accumulators the per-feature planners append to. */
export interface PlanDraft {
  readonly files: PlannedFile[];
  readonly createdFiles: string[];
  readonly updatedFiles: string[];
  readonly features: FeatureResult[];
  readonly dependencies: Set<string>;
}

export function createPlanDraft(): PlanDraft {
  return {
    files: [],
    createdFiles: [],
    updatedFiles: [],
    features: [],
    dependencies: new Set<string>(),
  };
}

export function skipped(id: FeatureId, detail = "not selected"): FeatureResult {
  return { id, outcome: "skipped", detail, snippet: null };
}

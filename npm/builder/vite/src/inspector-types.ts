export interface VizeInspectorLintPlanRequest {
  /** Project-relative files whose effective lint rules should be explained. */
  files: string[];
  /** Ask the integration to rebuild its plan before resolving the files. */
  fresh: boolean;
}

export type VizeInspectorLintPlanProvider = (request: VizeInspectorLintPlanRequest) => unknown;

export interface VizeInspectorOptions {
  /** Optional development-only lint-plan payload provider. */
  lintPlan?: VizeInspectorLintPlanProvider;
}

/* eslint-disable */

/** Batch type check result for NAPI */
export interface BatchTypeCheckResultNapi {
  filesChecked: number;
  filesWithErrors: number;
  totalErrors: number;
  totalWarnings: number;
  timeMs: number;
}

/** Related location for diagnostic (NAPI) */
export interface RelatedLocationNapi {
  message: string;
  start: number;
  end: number;
  filename?: string;
}

/** Type check capabilities result */
export interface TypeCheckCapabilitiesNapi {
  mode: string;
  description: string;
  checks: Array<TypeCheckCapabilityNapi>;
  notes: Array<string>;
}

/** Type check capabilities info */
export interface TypeCheckCapabilityNapi {
  name: string;
  description: string;
  severity: string;
}

/** Type diagnostic for NAPI */
export interface TypeDiagnosticNapi {
  severity: string;
  message: string;
  start: number;
  end: number;
  code?: string;
  help?: string;
  related: Array<RelatedLocationNapi>;
}

/** Type check options for NAPI */
export interface TypeCheckOptionsNapi {
  filename?: string;
  strict?: boolean;
  includeVirtualTs?: boolean;
  checkProps?: boolean;
  checkEmits?: boolean;
  checkTemplateBindings?: boolean;
  checkReactivity?: boolean;
  checkSetupContext?: boolean;
  checkInvalidExports?: boolean;
  checkFallthroughAttrs?: boolean;
  legacyVue2?: boolean;
}

/** Type check result for NAPI */
export interface TypeCheckResultNapi {
  diagnostics: Array<TypeDiagnosticNapi>;
  virtualTs?: string;
  errorCount: number;
  warningCount: number;
  analysisTimeMs?: number;
}

/** Get type checking capabilities info */
export declare function getTypeCheckCapabilities(): TypeCheckCapabilitiesNapi;

/**
 * Perform type checking on a Vue SFC
 *
 * This performs AST-based type analysis without requiring a TypeScript compiler.
 * For full type checking, use the CLI with Corsa integration.
 */
export declare function typeCheck(
  source: string,
  options?: TypeCheckOptionsNapi | undefined | null,
): TypeCheckResultNapi;

/** Batch type check SFC files matching a glob pattern (native multithreading) */
export declare function typeCheckBatch(
  pattern: string,
  options?: TypeCheckOptionsNapi | undefined | null,
): BatchTypeCheckResultNapi;

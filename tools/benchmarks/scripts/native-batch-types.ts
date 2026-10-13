/** Types for `native-batch.ts`, kept apart so that file stays a detectable rename. */

/** The `@vizejs/native` batch entry point; its result is validated, not trusted. */
export type NativeBatchCompiler = {
  compileSfcBatch: (pattern: string, options?: { threads: number }) => unknown;
};
export type NativeBatchResult = { success: number; failed: number; timeMs: number };
export type NativeBatchVariant = {
  id: string;
  label: string;
  files: number;
  measure: () => number;
};

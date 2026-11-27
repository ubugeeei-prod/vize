export { keyDiffLines, parseUnifiedDiff } from "./diff-parse.ts";
export type { DiffLine, DiffLineInput, DiffLineKind } from "./diff-parse.ts";
export type { DiffViewProps } from "./types.ts";

/** Unified diff rendered as a real table of context, add, and remove lines. */
export { default as DiffView } from "./diff-view.vue";

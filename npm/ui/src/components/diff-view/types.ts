import type { DiffLineInput } from "./diff-parse.ts";

/** Public props for the diff table. */
export interface DiffViewProps {
  /**
   * Accessible name of the table.
   *
   * @default "Diff"
   */
  readonly label?: string;

  /**
   * Unified diff text. Ignored when `lines` is set.
   *
   * @default ""
   */
  readonly patch?: string;

  /**
   * Structured lines. They win over `patch`.
   *
   * @default undefined
   */
  readonly lines?: readonly DiffLineInput[];
}

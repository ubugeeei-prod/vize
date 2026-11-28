/** Outcome presented by a result page. */
export type ResultStatus = "success" | "error" | "info" | "warning" | "not-found" | "empty";

/** Public props for the result status. */
export interface ResultProps {
  /**
   * Outcome. Only `error` uses `role="alert"`. Every other status uses `role="status"`.
   *
   * @default "info"
   */
  readonly status?: ResultStatus;

  /**
   * Primary heading shown to the user.
   */
  readonly title: string;

  /**
   * Supporting sentence. Omitted from the DOM when empty.
   *
   * @default undefined
   */
  readonly description?: string;
}

/** State passed to the default slot. */
export interface ResultSlotProps {
  readonly status: ResultStatus;
}

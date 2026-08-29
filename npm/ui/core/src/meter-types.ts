/** Native-friendly Meter range token derived from low/high thresholds. */
export type MeterRange = "high" | "low" | "medium";

/** Stable state token exposed by the Meter primitive. */
export type MeterState = "empty" | "full" | "high" | "low" | "medium" | "optimum";

/** State exposed to the default Meter slot and component instance. */
export interface MeterSlotState {
  /** Current finite value clamped to the normalized min/max range. */
  readonly value: number;

  /** Finite lower bound. */
  readonly min: number;

  /** Finite upper bound, always greater than `min`. */
  readonly max: number;

  /** Optional low threshold clamped to the normalized range. */
  readonly low: number | null;

  /** Optional high threshold clamped to the normalized range. */
  readonly high: number | null;

  /** Optional optimum threshold clamped to the normalized range. */
  readonly optimum: number | null;

  /** Completion percentage from 0 to 100. */
  readonly percent: number;

  /** Current threshold range. */
  readonly range: MeterRange;

  /** Whether the current range contains the optimum threshold. */
  readonly optimal: boolean;

  /** Whether raw inputs had to be repaired before reaching the native element. */
  readonly invalid: boolean;

  /** Stable state token for styling and tests. */
  readonly state: MeterState;
}

/** Public component instance exposed by the Meter primitive. */
export interface MeterExpose extends MeterSlotState {
  /** Rendered native meter element. */
  readonly element: HTMLMeterElement | null;
}

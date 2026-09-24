/** Values accepted by the native `aria-invalid` attribute. */
export type PinInputAriaInvalid = boolean | "grammar" | "spelling";

/** Accepted character class. */
export type PinInputType = "alphanumeric" | "numeric";

/** State published through the PinInput `data-state` contract. */
export type PinInputState = "complete" | "disabled" | "empty" | "incomplete";

type BuildPinTuple<
  Length extends number,
  Accumulator extends readonly string[] = readonly [],
> = Accumulator["length"] extends Length
  ? Accumulator
  : BuildPinTuple<Length, readonly [...Accumulator, string]>;

/**
 * Characters of a complete code as a fixed-length tuple when `length` is a
 * literal (for example `readonly [string, string, string, string]` for 4),
 * or `readonly string[]` when the length is only known at runtime.
 */
export type PinInputCharacters<Length extends number> = number extends Length
  ? readonly string[]
  : `${Length}` extends `-${string}` | `${string}.${string}`
    ? readonly string[]
    : BuildPinTuple<Length>;

/** State exposed to the PinInput slot and instance. */
export interface PinInputSlotState {
  /** Entered characters joined, without gaps. */
  readonly value: string;

  /** Number of fields. */
  readonly length: number;

  /** Field indexes to render, `[0, 1, ..., length - 1]`. */
  readonly indexes: readonly number[];

  /** Whether every field is filled. */
  readonly complete: boolean;

  /** Whether the fields are disabled. */
  readonly disabled: boolean;

  /** Stable state token. */
  readonly state: PinInputState;
}

/** Public instance API of PinInput. */
export interface PinInputExpose extends PinInputSlotState {
  /** Rendered group element. */
  readonly root: HTMLDivElement | null;

  /** Focus a field (default: the first empty one). */
  readonly focus: (index?: number) => void;

  /** Replace the code (filtered and truncated); returns whether it changed. */
  readonly setValue: (value: string) => boolean;

  /** Clear every field; returns whether the value changed. */
  readonly clear: () => boolean;
}

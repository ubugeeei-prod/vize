import type { PlainDate } from "../calendar/plain-date.ts";
import type {
  DateSegmentType,
  EditableSegmentType,
  FieldSegmentType,
  TimeSegmentType,
} from "./field-segments.ts";
import type { FieldSegmentState, SegmentedFieldState } from "./field-segment-runtime.ts";

export type {
  DateSegmentType,
  EditableSegmentType,
  FieldSegmentState,
  FieldSegmentType,
  SegmentedFieldState,
  TimeSegmentType,
};

/** Placeholder text per editable segment, for example `{ year: "jjjj" }`. */
export type FieldSegmentPlaceholders = Partial<Record<EditableSegmentType, string>>;

/** State shared by segmented field slots and instances. */
export interface SegmentedFieldSlotState<Value> {
  /** Committed value, or `null` while empty or incomplete. */
  readonly value: Value | null;

  /** Rendered segments in locale order, literals included. */
  readonly segments: readonly FieldSegmentState[];

  /** Value-state token: `empty`, `partial`, `complete`, or `invalid`. */
  readonly state: SegmentedFieldState;

  /** Whether the value violates `min`, `max`, availability, or `ariaInvalid`. */
  readonly invalid: boolean;

  /** Whether segments are disabled. */
  readonly disabled: boolean;

  /** Whether segments are focusable but immutable. */
  readonly readOnly: boolean;

  /** Whether a value is required. */
  readonly required: boolean;

  /** Resolved BCP 47 locale that decided segment order. */
  readonly locale: string;
}

/** State exposed to the DateField default slot. */
export type DateFieldSlotState = SegmentedFieldSlotState<PlainDate>;

/** Public instance surface shared by DateField and TimeField. */
export interface SegmentedFieldExpose<Value> extends SegmentedFieldSlotState<Value> {
  /** Rendered group element. */
  readonly root: HTMLDivElement | null;

  /** Focus a segment, or the first empty segment (else the first segment); returns whether focus moved. */
  readonly focus: (segment?: EditableSegmentType, options?: FocusOptions) => boolean;

  /** Replace the value and every segment; returns whether the value changed. */
  readonly setValue: (value: Value | null) => boolean;

  /** Clear every segment; returns whether the value changed. */
  readonly clear: () => boolean;

  /** Restore `defaultValue`; returns whether the value changed. */
  readonly reset: () => boolean;
}

/** Public instance exposed by DateField. */
export type DateFieldExpose = SegmentedFieldExpose<PlainDate>;

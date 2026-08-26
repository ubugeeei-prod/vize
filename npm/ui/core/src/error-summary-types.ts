import type { ComputedRef, MaybeRefOrGetter, ShallowRef } from "vue";

/** One invalid field listed by the error summary. */
export interface ErrorSummaryField {
  /** Document id of the focusable invalid control. */
  readonly id: string;

  /** Human-readable error text for the summary link. */
  readonly message: string;

  /**
   * Accessible field label, prefixed onto the default link text.
   *
   * @default undefined
   */
  readonly label?: string;
}

/** Options shared by {@link createErrorSummary} and {@link useErrorSummary}. */
export interface ErrorSummaryOptions {
  /**
   * Invalid fields in document order. Field ids must be unique.
   *
   * @default []
   */
  readonly fields?: MaybeRefOrGetter<readonly ErrorSummaryField[] | undefined>;

  /**
   * Summary container that receives focus when invalid fields appear.
   *
   * @default undefined
   */
  readonly element?: MaybeRefOrGetter<HTMLElement | null | undefined>;

  /**
   * Move focus into the summary when invalid fields appear.
   *
   * @default true
   */
  readonly autoFocus?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Resolve one field's focusable control.
   *
   * @default Document lookup by field id
   */
  readonly resolveField?: (field: ErrorSummaryField) => HTMLElement | null;

  /**
   * Called after a repair settles focus, with the restored element or `null`.
   *
   * @default undefined
   */
  readonly onRestore?: (target: HTMLElement | null) => void;
}

/** Focus-managing controller behind the error summary component. */
export interface ErrorSummaryController {
  /** Current invalid fields in document order. */
  readonly fields: Readonly<ShallowRef<readonly ErrorSummaryField[]>>;

  /** Whether any field is invalid. */
  readonly hasErrors: ComputedRef<boolean>;

  /**
   * Capture the previously focused element and move focus to the summary.
   *
   * @returns Whether the summary element existed and received focus.
   */
  readonly focusSummary: () => boolean;

  /** Move focus to one listed invalid control. `null` when unknown or absent. */
  readonly focusField: (id: string) => HTMLElement | null;

  /**
   * Restore focus to the element focused before the summary took it.
   *
   * @returns Whether a connected capture target regained focus.
   */
  readonly restoreFocus: () => boolean;

  /** Stop watching fields and release the captured focus target. */
  readonly dispose: () => void;
}

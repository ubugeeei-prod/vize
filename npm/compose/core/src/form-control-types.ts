/**
 * Public control surface returned by {@link useForm}.
 */

import type { ComputedRef, Ref, ShallowRef, WritableComputedRef } from "vue";

import type { FormResetOptions, FormResult, SetFieldValueOptions } from "./form-options.ts";
import type {
  DirtyFields,
  FormArrayItem,
  FormArrayPath,
  FormErrorRecord,
  FormErrors,
  FormPath,
  FormPathValue,
  ValidatorResult,
} from "./form-path-types.ts";

/** Reactive controls for one field, returned by `field`. */
export interface FormFieldControls<Value, Path extends string = string> {
  /** Dotted path of the field (also a suitable `name` attribute). */
  readonly path: Path;

  /** Writable value bound to the form state (use with `v-model`). */
  readonly value: WritableComputedRef<Value>;

  /** Current error messages. */
  readonly errors: ComputedRef<readonly string[]>;

  /** First error message. */
  readonly error: ComputedRef<string | undefined>;

  /** Whether the value differs from its initial value. */
  readonly dirty: ComputedRef<boolean>;

  /** Whether the field was blurred or touched programmatically. */
  readonly touched: ComputedRef<boolean>;

  /** Whether a validation of this field is pending. */
  readonly validating: ComputedRef<boolean>;

  /** Mark the field touched (wire to `blur`); validates when configured. */
  readonly onBlur: () => void;

  /**
   * Validate this field.
   *
   * @returns Whether the field is valid (`false` also when superseded).
   */
  readonly validate: () => Promise<boolean>;

  /** Restore the initial value and clear errors and touched state. */
  readonly reset: () => void;
}

/** One entry of a field array, with a stable key for `v-for`. */
export interface FormArrayEntry<Item, Path extends string = string> {
  /** Stable key that follows the item through moves. */
  readonly key: number;

  /** Current index. */
  readonly index: number;

  /** Dotted path of the item, e.g. `items.2`. */
  readonly path: `${Path}.${number}`;

  /** Current item value. */
  readonly value: Item;
}

/** Typed operations on an array field, returned by `fieldArray`. */
export interface FormFieldArrayControls<Item, Path extends string = string> {
  /** Entries with stable keys. */
  readonly entries: ComputedRef<readonly FormArrayEntry<Item, Path>[]>;

  /** Append items. */
  readonly append: (...items: Item[]) => void;

  /** Prepend items. */
  readonly prepend: (...items: Item[]) => void;

  /** Insert an item at `index`. */
  readonly insert: (index: number, item: Item) => void;

  /** Remove the item at `index`, shifting errors and touched state. */
  readonly remove: (index: number) => void;

  /** Move an item from one index to another, carrying its state along. */
  readonly move: (from: number, to: number) => void;

  /** Swap two items. */
  readonly swap: (first: number, second: number) => void;

  /** Replace all items and reset their state. */
  readonly replace: (items: readonly Item[]) => void;
}

/** Reactive state and actions returned by `useForm`. */
export interface FormControls<Values extends object, Output> {
  /** Current values. Mutations are tracked (dirty state derives from them). */
  readonly values: Ref<Values>;

  /** Current errors keyed by path; `""` holds form-level errors. */
  readonly errors: Readonly<ShallowRef<FormErrorRecord>>;

  /** Whether no errors are currently recorded. */
  readonly valid: ComputedRef<boolean>;

  /** Whether any value differs from its initial value. */
  readonly dirty: ComputedRef<boolean>;

  /** Nested flags for values that differ from the initial values. */
  readonly dirtyFields: ComputedRef<DirtyFields<Values>>;

  /** Whether any validation is pending. */
  readonly validating: ComputedRef<boolean>;

  /** Whether a submission is in progress. */
  readonly submitting: Readonly<Ref<boolean>>;

  /** Number of submit attempts. Disabled submits are not counted. */
  readonly submitCount: Readonly<Ref<number>>;

  /**
   * Whether the latest finished submit was valid and its handler settled.
   * A superseded submit leaves this flag unchanged.
   */
  readonly isSubmitSuccessful: Readonly<Ref<boolean>>;

  /** Whether value writes, field-array edits, and submission are ignored. */
  readonly disabled: ComputedRef<boolean>;

  /**
   * Typed controls for one field.
   *
   * @param path Dotted path.
   * @returns Field controls whose value type follows the path.
   */
  readonly field: <Path extends FormPath<Values>>(
    path: Path,
  ) => FormFieldControls<FormPathValue<Values, Path>, Path>;

  /**
   * Typed operations for an array field.
   *
   * @param path Dotted path of an array.
   * @returns Field array controls.
   */
  readonly fieldArray: <Path extends FormArrayPath<Values>>(
    path: Path,
  ) => FormFieldArrayControls<FormArrayItem<Values, Path>, Path>;

  /**
   * Read the value at a path.
   *
   * @param path Dotted path.
   * @returns The value.
   */
  readonly getValue: <Path extends FormPath<Values>>(path: Path) => FormPathValue<Values, Path>;

  /**
   * Read a path as a computed ref that tracks later edits.
   *
   * @param path Dotted path.
   * @returns The value at that path.
   */
  readonly watch: <Path extends FormPath<Values>>(
    path: Path,
  ) => ComputedRef<FormPathValue<Values, Path>>;

  /**
   * Write the value at a path. Does nothing while `disabled` is true.
   *
   * @param path Dotted path.
   * @param value New value.
   * @param options Touch and validation behavior.
   */
  readonly setValue: <Path extends FormPath<Values>>(
    path: Path,
    value: FormPathValue<Values, Path>,
    options?: SetFieldValueOptions,
  ) => void;

  /**
   * Replace (or shallowly merge) the values without touching initial values.
   * Does nothing while `disabled` is true.
   *
   * @param values New values.
   * @param options `merge: true` shallowly merges top-level keys.
   */
  readonly setValues: (values: Partial<Values>, options?: { readonly merge?: boolean }) => void;

  /**
   * Whether a field was touched.
   *
   * @param path Dotted path.
   * @returns Touched state.
   */
  readonly isTouched: (path: FormPath<Values>) => boolean;

  /**
   * Replace errors (for example with server-side errors).
   *
   * @param errors Errors keyed by path.
   */
  readonly setErrors: (errors: FormErrors<Values>) => void;

  /**
   * Set one path's errors without replacing the others.
   *
   * @param path Dotted path. `""` is the form-level error.
   * @param message Messages for that path. An empty result clears it.
   */
  readonly setError: (path: FormPath<Values> | "", message: ValidatorResult) => void;

  /**
   * Clear errors of one path, or all errors.
   *
   * @param path Dotted path; omit to clear everything.
   */
  readonly clearErrors: (path?: FormPath<Values> | "") => void;

  /**
   * Run every validator, aborting any older validation still in flight.
   *
   * @returns The discriminated result; never rejects on validation failure.
   */
  readonly validate: () => Promise<FormResult<Output>>;

  /**
   * Validate and, when valid, call `handler` (or the `onSubmit` option).
   * A newer submit aborts an older one's pending validation. Does nothing
   * while `disabled` is true.
   *
   * @param handler Submit handler overriding `onSubmit`.
   * @returns The validation result.
   */
  readonly submit: (
    handler?: (value: Output, context: { readonly signal: AbortSignal }) => unknown,
  ) => Promise<FormResult<Output>>;

  /**
   * Create a `submit` event listener that prevents the native submission.
   *
   * @param handler Submit handler overriding `onSubmit`.
   * @returns Event listener for `<form @submit>`.
   */
  readonly handleSubmit: (
    handler?: (value: Output, context: { readonly signal: AbortSignal }) => unknown,
  ) => (event?: Event) => Promise<FormResult<Output>>;

  /**
   * Restore initial values (or install new ones). Omitted keep-flags clear
   * that piece of state. Still runs while `disabled` is true.
   *
   * @param values New initial values.
   * @param options State to retain.
   */
  readonly reset: (values?: Values, options?: FormResetOptions) => void;
}

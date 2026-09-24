import { computed, ref, shallowRef, toRaw, triggerRef } from "vue";
import type { ComputedRef, Ref, ShallowRef, WritableComputedRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import { validateStandardSchema } from "./standard-schema.ts";
import type { NormalizedSchemaIssue, StandardSchemaV1 } from "./standard-schema.ts";

/** Values treated as leaves: paths never descend into them. */
export type FormLeaf =
  | string
  | number
  | bigint
  | boolean
  | symbol
  | null
  | undefined
  | Date
  | RegExp
  | Blob
  | Map<unknown, unknown>
  | Set<unknown>
  | ((...arguments_: never[]) => unknown);

type Decrement = [never, 0, 1, 2, 3, 4, 5, 6, 7, 8];

/**
 * Every dotted path into `Values`, as template-literal keys.
 *
 * Objects contribute their string keys, arrays and tuples contribute
 * `${number}` segments, and {@link FormLeaf} values end a path. Recursion
 * is bounded at eight levels to keep type checking fast.
 */
export type FormPath<Values, Depth extends number = 8> = [Depth] extends [never]
  ? never
  : Values extends FormLeaf
    ? never
    : Values extends readonly (infer Item)[]
      ? `${number}` | `${number}.${FormPath<Item, Decrement[Depth]>}`
      : {
          [Key in keyof Values & string]:
            | Key
            | (FormPath<Values[Key], Decrement[Depth]> extends infer Rest extends string
                ? [Rest] extends [never]
                  ? never
                  : `${Key}.${Rest}`
                : never);
        }[keyof Values & string];

/** Value type found at `Path` inside `Values`. */
export type FormPathValue<Values, Path extends string> = Path extends `${infer Head}.${infer Rest}`
  ? Values extends readonly (infer Item)[]
    ? Head extends `${number}`
      ? FormPathValue<Item, Rest>
      : never
    : Head extends keyof Values
      ? FormPathValue<Values[Head], Rest>
      : never
  : Values extends readonly (infer Item)[]
    ? Path extends `${number}`
      ? Item
      : never
    : Path extends keyof Values
      ? Values[Path]
      : never;

/** Paths whose value is an array (usable with {@link FormControls.fieldArray}). */
export type FormArrayPath<Values> = {
  [Path in FormPath<Values>]: FormPathValue<Values, Path> extends readonly unknown[] ? Path : never;
}[FormPath<Values>];

/** Item type of the array at `Path`. */
export type FormArrayItem<Values, Path extends string> =
  FormPathValue<Values, Path> extends readonly (infer Item)[] ? Item : never;

/**
 * Error messages keyed by dotted path; `""` holds form-level errors.
 *
 * Known paths are typed for autocompletion; the string index signature
 * stays because schemas and servers may report paths that are not fields.
 */
export type FormErrors<Values> = {
  readonly [Path in FormPath<Values> | ""]?: readonly string[];
} & { readonly [path: string]: readonly string[] | undefined };

/** Current errors as recorded: messages keyed by dotted path (`""` = form level). */
export type FormErrorRecord = Readonly<Record<string, readonly string[]>>;

/** Result of a validator: messages, one message, or nothing when valid. */
export type ValidatorResult = string | readonly string[] | null | undefined | void;

/** Context passed to field validators. */
export interface FieldValidatorContext<Values> {
  /** Current form values (`undefined` for standalone fields). */
  readonly values: Values;

  /** Dotted path of the validated field. */
  readonly path: string;

  /** Aborted when a newer validation supersedes this one. */
  readonly signal: AbortSignal;
}

/** Plain validator for one field value. */
export type FieldValidator<Value, Values = undefined> = (
  value: Value,
  context: FieldValidatorContext<Values>,
) => ValidatorResult | Promise<ValidatorResult>;

/** Plain validator for the whole form, returning errors keyed by path. */
export type FormValidator<Values> = (
  values: Values,
  context: { readonly signal: AbortSignal },
) => FormErrors<Values> | undefined | void | Promise<FormErrors<Values> | undefined | void>;

/** Per-field validators keyed by path. */
export type FormFieldValidators<Values> = {
  readonly [Path in FormPath<Values>]?: FieldValidator<FormPathValue<Values, Path>, Values>;
};

/** When validation runs automatically. */
export type FormValidationTrigger = "change" | "blur" | "submit";

/** Discriminated outcome of {@link FormControls.validate} and submission. */
export type FormResult<Output> =
  | {
      /** Every validator passed. */
      readonly status: "valid";
      /** Schema output, or the values when no schema is configured. */
      readonly value: Output;
    }
  | {
      /** At least one validator failed. */
      readonly status: "invalid";
      /** Errors keyed by path. */
      readonly errors: FormErrorRecord;
    }
  | {
      /** A newer validation started before this one finished. */
      readonly status: "superseded";
    };

/** Options for {@link useForm}. */
export interface UseFormOptions<Values extends object, Output> {
  /** Initial values, or a factory producing fresh initial values. */
  readonly initialValues: Values | (() => Values);

  /**
   * Standard Schema v1 validator for the whole form. Its output becomes the
   * submitted value.
   *
   * @default undefined
   */
  readonly schema?: StandardSchemaV1<NoInfer<Values>, Output>;

  /**
   * Plain whole-form validator.
   *
   * @default undefined
   */
  readonly validate?: NoInfer<FormValidator<Values>>;

  /**
   * Plain per-field validators keyed by path.
   *
   * @default {}
   */
  readonly validators?: NoInfer<FormFieldValidators<Values>>;

  /**
   * When fields validate before the first submit.
   *
   * @default "submit"
   */
  readonly validateOn?: FormValidationTrigger;

  /**
   * When fields re-validate after the first submit.
   *
   * @default "change"
   */
  readonly revalidateOn?: FormValidationTrigger;

  /**
   * Default submit handler for {@link FormControls.submit}.
   *
   * @default undefined
   */
  readonly onSubmit?: (value: Output, context: { readonly signal: AbortSignal }) => unknown;
}

/** Options for {@link FormControls.setValue}. */
export interface SetFieldValueOptions {
  /**
   * Mark the field touched.
   *
   * @default false
   */
  readonly touch?: boolean;

  /**
   * Validate the field after setting it.
   *
   * @default true while the active trigger is "change"
   */
  readonly validate?: boolean;
}

/** Reactive controls for one field, returned by {@link FormControls.field}. */
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

/** Typed operations on an array field, returned by {@link FormControls.fieldArray}. */
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

/** Reactive state and actions returned by {@link useForm}. */
export interface FormControls<Values extends object, Output> {
  /** Current values. Mutations are tracked (dirty state derives from them). */
  readonly values: Ref<Values>;

  /** Current errors keyed by path; `""` holds form-level errors. */
  readonly errors: Readonly<ShallowRef<FormErrorRecord>>;

  /** Whether no errors are currently recorded. */
  readonly valid: ComputedRef<boolean>;

  /** Whether any value differs from its initial value. */
  readonly dirty: ComputedRef<boolean>;

  /** Whether any validation is pending. */
  readonly validating: ComputedRef<boolean>;

  /** Whether a submission is in progress. */
  readonly submitting: Readonly<Ref<boolean>>;

  /** Number of submit attempts. */
  readonly submitCount: Readonly<Ref<number>>;

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
   * Write the value at a path.
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
   * A newer submit aborts an older one's pending validation.
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
   * Restore initial values (or install new ones) and clear all state.
   *
   * @param values New initial values.
   */
  readonly reset: (values?: Values) => void;
}

type Container = Record<string, unknown> | unknown[];

function isContainer(value: unknown): value is Container {
  if (Array.isArray(value)) return true;
  if (typeof value !== "object" || value === null) return false;
  const prototype: unknown = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function splitPath(path: string): readonly string[] {
  return path === "" ? [] : path.split(".");
}

function readChild(container: Container, key: string): unknown {
  return Array.isArray(container) ? container[Number(key)] : container[key];
}

function writeChild(container: Container, key: string, value: unknown): void {
  if (Array.isArray(container)) container[Number(key)] = value;
  else container[key] = value;
}

/** Read a dotted path. */
function readPath(root: unknown, path: string): unknown {
  let current = root;
  for (const key of splitPath(path)) {
    if (!isContainer(current)) return undefined;
    current = readChild(current, key);
  }
  return current;
}

/** Write a dotted path, creating intermediate containers. */
function writePath(root: unknown, path: string, value: unknown): void {
  const keys = splitPath(path);
  const last = keys.at(-1);
  if (last === undefined || !isContainer(root)) return;
  let current: Container = root;
  for (let index = 0; index < keys.length - 1; index += 1) {
    const key = keys[index] ?? "";
    let next = readChild(current, key);
    if (!isContainer(next)) {
      next = /^\d+$/.test(keys[index + 1] ?? "") ? [] : {};
      writeChild(current, key, next);
    }
    if (!isContainer(next)) return;
    current = next;
  }
  writeChild(current, last, value);
}

function cloneValue<Value>(value: Value): Value {
  return structuredClone(toRaw(value));
}

function deepEqual(left: unknown, right: unknown): boolean {
  if (Object.is(left, right)) return true;
  if (left instanceof Date && right instanceof Date) return left.getTime() === right.getTime();
  if (Array.isArray(left) && Array.isArray(right)) {
    return (
      left.length === right.length && left.every((item, index) => deepEqual(item, right[index]))
    );
  }
  if (isContainer(left) && isContainer(right) && !Array.isArray(left) && !Array.isArray(right)) {
    const keys = new Set([...Object.keys(left), ...Object.keys(right)]);
    for (const key of keys) if (!deepEqual(left[key], right[key])) return false;
    return true;
  }
  return false;
}

function toMessages(result: ValidatorResult): readonly string[] {
  if (result === undefined || result === null || result === "") return [];
  return typeof result === "string" ? [result] : result.filter((message) => message !== "");
}

function unknownMessages(result: unknown): readonly string[] {
  if (typeof result === "string") return result === "" ? [] : [result];
  if (!Array.isArray(result)) return [];
  return result.filter(
    (message): message is string => typeof message === "string" && message !== "",
  );
}

/** Rewrite `${arrayPath}.${index}…` keys through an index mapping (undefined drops). */
function remapKeys<Entry>(
  entries: Iterable<[string, Entry]>,
  arrayPath: string,
  mapIndex: (index: number) => number | undefined,
): [string, Entry][] {
  const prefix = `${arrayPath}.`;
  const result: [string, Entry][] = [];
  for (const [key, entry] of entries) {
    if (!key.startsWith(prefix)) {
      result.push([key, entry]);
      continue;
    }
    const rest = key.slice(prefix.length);
    const dot = rest.indexOf(".");
    const indexText = dot === -1 ? rest : rest.slice(0, dot);
    const mapped = /^\d+$/.test(indexText) ? mapIndex(Number(indexText)) : Number.NaN;
    if (mapped === undefined) continue;
    if (Number.isNaN(mapped)) {
      result.push([key, entry]);
      continue;
    }
    result.push([`${prefix}${String(mapped)}${dot === -1 ? "" : rest.slice(dot)}`, entry]);
  }
  return result;
}

/**
 * Fully typed form state with validation, field arrays, and submission.
 *
 * The value type is inferred from `initialValues`; every path is a
 * template-literal key, so `field("address.city")` returns controls whose
 * `value` is typed as the city's type and misspelled paths do not compile.
 * Validation combines a Standard Schema v1 validator (Zod, Valibot, ArkType,
 * … via the `~standard` protocol), a whole-form validator, and per-field
 * validators; all may be asynchronous. A newer validation aborts older ones
 * through their `signal` and their results are discarded, so a slow check
 * can never overwrite fresher errors.
 *
 * This composable owns the *values*. `@vizejs/ui/form` owns accessible
 * error presentation (error summaries, focus management, native constraint
 * validation); feed {@link FormControls.errors} into it to render them.
 *
 * Server rendering: pure state, no host access; validation never runs
 * automatically during setup, so server and client render the same markup.
 * Pending validations are aborted when the owning scope stops.
 *
 * @example
 * ```ts
 * const form = useForm({
 *   initialValues: { name: "", address: { city: "" }, tags: [] as string[] },
 *   validators: { name: (name) => (name ? undefined : "Required") },
 *   onSubmit: (values) => api.save(values),
 * });
 * const city = form.field("address.city"); // city.value: Ref<string>
 * const tags = form.fieldArray("tags");
 * tags.append("vue");
 * ```
 *
 * @typeParam Values Form values, inferred from `initialValues`.
 * @typeParam Output Submitted value, inferred from `schema` (defaults to `Values`).
 * @param options Initial values, validators, triggers, and submit handler.
 * @returns Form state and actions.
 */
export function useForm<Values extends object, Output = Values>(
  options: UseFormOptions<Values, Output>,
): FormControls<Values, Output> {
  const createInitial = (): Values =>
    typeof options.initialValues === "function"
      ? options.initialValues()
      : cloneValue(options.initialValues);
  let initial = createInitial();
  // Form values are plain data, for which `UnwrapRef<Values>` equals `Values`;
  // a deep `ref` keeps nested edits (including `v-model` on nested paths) reactive.
  const values = ref(cloneValue(initial)) as Ref<Values>;
  const errors = shallowRef<Record<string, readonly string[]>>({});
  const touched = shallowRef(new Set<string>());
  const pending = ref(new Map<string, number>());
  const submitting = ref(false);
  const submitCount = ref(0);
  const arrayKeys = new Map<string, number[]>();
  let nextKey = 0;
  let formController: AbortController | undefined;
  const fieldControllers = new Map<string, AbortController>();
  const validators: readonly (readonly [string, unknown])[] = Object.entries(
    options.validators ?? {},
  );
  const runValidator = (
    validator: unknown,
    path: string,
    snapshot: Values,
    signal: AbortSignal,
  ): Promise<readonly string[]> => {
    if (typeof validator !== "function") return Promise.resolve([]);
    const outcome: unknown = Reflect.apply(validator, undefined, [
      readPath(snapshot, path),
      { values: snapshot, path, signal },
    ]);
    return Promise.resolve(outcome).then(unknownMessages);
  };

  const trigger = (): FormValidationTrigger =>
    submitCount.value > 0 ? (options.revalidateOn ?? "change") : (options.validateOn ?? "submit");

  const beginPending = (path: string): void => {
    pending.value.set(path, (pending.value.get(path) ?? 0) + 1);
  };
  const endPending = (path: string): void => {
    const count = (pending.value.get(path) ?? 1) - 1;
    if (count <= 0) pending.value.delete(path);
    else pending.value.set(path, count);
  };

  const setErrorsFor = (path: string, messages: readonly string[]): void => {
    const next = { ...errors.value };
    if (messages.length === 0) Reflect.deleteProperty(next, path);
    else next[path] = messages;
    errors.value = next;
  };

  const readFieldValidator = (path: string): unknown =>
    validators.find(([candidate]) => candidate === path)?.[1];

  /** Collect every error source; resolves to errors keyed by path. */
  const collectErrors = async (
    signal: AbortSignal,
  ): Promise<{ readonly errors: Record<string, string[]>; readonly output: unknown }> => {
    const snapshot = values.value;
    const collected: Record<string, string[]> = {};
    const add = (path: string, messages: readonly string[]): void => {
      if (messages.length === 0) return;
      (collected[path] ??= []).push(...messages);
    };
    let output: unknown = snapshot;
    const tasks: Promise<void>[] = [];
    if (options.schema) {
      const schema = options.schema;
      tasks.push(
        validateStandardSchema(schema, cloneValue(snapshot)).then((result) => {
          if (result.status === "valid") output = result.value;
          else for (const issue of result.issues) add(issue.path, [issue.message]);
        }),
      );
    }
    if (options.validate) {
      const validate = options.validate;
      tasks.push(
        Promise.resolve(validate(snapshot, { signal })).then((result) => {
          if (!result) return;
          for (const [path, messages] of Object.entries(result)) {
            if (messages !== undefined) add(path, toMessages(messages));
          }
        }),
      );
    }
    for (const [path, validator] of validators) {
      tasks.push(
        runValidator(validator, path, snapshot, signal).then((messages) => add(path, messages)),
      );
    }
    await Promise.all(tasks);
    return { errors: collected, output };
  };

  const validate = async (): Promise<FormResult<Output>> => {
    formController?.abort(new DOMException("A newer validation started.", "AbortError"));
    for (const controller of fieldControllers.values()) controller.abort();
    fieldControllers.clear();
    const controller = new AbortController();
    formController = controller;
    beginPending("");
    try {
      const outcome = await collectErrors(controller.signal);
      if (controller.signal.aborted) return { status: "superseded" };
      errors.value = outcome.errors;
      if (Object.keys(outcome.errors).length > 0) {
        return { status: "invalid", errors: { ...outcome.errors } };
      }
      // Without a schema the output is the values themselves (`Output`
      // defaults to `Values`); with a schema it is the schema's parsed output.
      return { status: "valid", value: outcome.output as Output };
    } finally {
      endPending("");
      if (formController === controller) formController = undefined;
    }
  };

  const schemaIssuesFor = async (path: string): Promise<readonly string[]> => {
    if (!options.schema) return [];
    const result = await validateStandardSchema(options.schema, cloneValue(values.value));
    if (result.status === "valid") return [];
    return result.issues
      .filter((issue: NormalizedSchemaIssue) => issue.path === path)
      .map((issue) => issue.message);
  };

  const validateField = async (path: string): Promise<boolean> => {
    fieldControllers.get(path)?.abort();
    const controller = new AbortController();
    fieldControllers.set(path, controller);
    beginPending(path);
    try {
      const validator = readFieldValidator(path);
      const snapshot = values.value;
      const [own, fromSchema, fromForm] = await Promise.all([
        runValidator(validator, path, snapshot, controller.signal),
        schemaIssuesFor(path),
        options.validate
          ? Promise.resolve(options.validate(snapshot, { signal: controller.signal })).then(
              (result) => {
                const messages: unknown = result ? Reflect.get(result, path) : undefined;
                return Array.isArray(messages)
                  ? messages.filter((message): message is string => typeof message === "string")
                  : [];
              },
            )
          : Promise.resolve([]),
      ]);
      if (controller.signal.aborted) return false;
      const messages = [...fromSchema, ...fromForm, ...own];
      setErrorsFor(path, messages);
      return messages.length === 0;
    } finally {
      endPending(path);
      if (fieldControllers.get(path) === controller) fieldControllers.delete(path);
    }
  };

  const touch = (path: string): void => {
    if (touched.value.has(path)) return;
    const next = new Set(touched.value);
    next.add(path);
    touched.value = next;
  };

  const setValueAt = (path: string, value: unknown, setOptions: SetFieldValueOptions = {}) => {
    writePath(values.value, path, value);
    if (setOptions.touch) touch(path);
    if (setOptions.validate ?? trigger() === "change") void validateField(path);
  };

  const syncKeys = (path: string, length: number): number[] => {
    let keys = arrayKeys.get(path);
    if (!keys) {
      keys = [];
      arrayKeys.set(path, keys);
    }
    while (keys.length < length) keys.push(nextKey++);
    if (keys.length > length) keys.length = length;
    return keys;
  };

  const readArray = (path: string): unknown[] => {
    const current = readPath(values.value, path);
    if (Array.isArray(current)) return current;
    const created: unknown[] = [];
    writePath(values.value, path, created);
    return created;
  };

  const remapState = (path: string, mapIndex: (index: number) => number | undefined): void => {
    errors.value = Object.fromEntries(remapKeys(Object.entries(errors.value), path, mapIndex));
    touched.value = new Set(
      remapKeys(
        [...touched.value].map((key): [string, true] => [key, true]),
        path,
        mapIndex,
      ).map(([key]) => key),
    );
  };

  const keysVersion = shallowRef(0);
  const bumpKeys = (): void => {
    keysVersion.value += 1;
    triggerRef(keysVersion);
  };

  const field = <Path extends FormPath<Values>>(
    path: Path,
  ): FormFieldControls<FormPathValue<Values, Path>, Path> => {
    const key: string = path;
    // `readPath` walks the same dotted path the type-level `FormPathValue`
    // resolves, so the runtime value has the declared type.
    const read = (): FormPathValue<Values, Path> =>
      readPath(values.value, key) as FormPathValue<Values, Path>;
    return {
      path,
      value: computed({
        get: read,
        set: (next) => setValueAt(key, next),
      }),
      errors: computed(() => errors.value[key] ?? []),
      error: computed(() => errors.value[key]?.[0]),
      dirty: computed(() => !deepEqual(readPath(values.value, key), readPath(initial, key))),
      touched: computed(() => touched.value.has(key)),
      validating: computed(() => pending.value.has(key) || pending.value.has("")),
      onBlur: () => {
        touch(key);
        if (trigger() === "blur") void validateField(key);
      },
      validate: () => validateField(key),
      reset: () => {
        writePath(values.value, key, cloneValue(readPath(initial, key)));
        setErrorsFor(key, []);
        const next = new Set(touched.value);
        next.delete(key);
        touched.value = next;
      },
    };
  };

  const fieldArray = <Path extends FormArrayPath<Values>>(
    path: Path,
  ): FormFieldArrayControls<FormArrayItem<Values, Path>, Path> => {
    type Item = FormArrayItem<Values, Path>;
    const key: string = path;
    const afterChange = (): void => {
      bumpKeys();
      if (trigger() === "change") void validateField(key);
    };
    const insertAt = (index: number, items: readonly Item[]): void => {
      const array = readArray(key);
      const keys = syncKeys(key, array.length);
      const at = Math.max(0, Math.min(index, array.length));
      remapState(key, (old) => (old >= at ? old + items.length : old));
      array.splice(at, 0, ...items);
      keys.splice(at, 0, ...items.map(() => nextKey++));
      afterChange();
    };
    return {
      entries: computed(() => {
        void keysVersion.value;
        const array = readPath(values.value, key);
        if (!Array.isArray(array)) return [];
        const keys = syncKeys(key, array.length);
        return array.map((value: unknown, index) => ({
          key: keys[index] ?? index,
          index,
          path: `${path}.${index}` as const,
          // Items of the array at `Path` have the declared item type.
          value: value as Item,
        }));
      }),
      append: (...items) => insertAt(Number.MAX_SAFE_INTEGER, items),
      prepend: (...items) => insertAt(0, items),
      insert: (index, item) => insertAt(index, [item]),
      remove: (index) => {
        const array = readArray(key);
        if (index < 0 || index >= array.length) return;
        const keys = syncKeys(key, array.length);
        remapState(key, (old) => (old === index ? undefined : old > index ? old - 1 : old));
        array.splice(index, 1);
        keys.splice(index, 1);
        afterChange();
      },
      move: (from, to) => {
        const array = readArray(key);
        if (from < 0 || from >= array.length || to < 0 || to >= array.length || from === to) return;
        const keys = syncKeys(key, array.length);
        remapState(key, (old) => {
          if (old === from) return to;
          if (from < to && old > from && old <= to) return old - 1;
          if (from > to && old >= to && old < from) return old + 1;
          return old;
        });
        const [item] = array.splice(from, 1);
        array.splice(to, 0, item);
        const [movedKey] = keys.splice(from, 1);
        keys.splice(to, 0, movedKey ?? nextKey++);
        afterChange();
      },
      swap: (first, second) => {
        const array = readArray(key);
        if (first < 0 || second < 0 || first >= array.length || second >= array.length) return;
        const keys = syncKeys(key, array.length);
        remapState(key, (old) => (old === first ? second : old === second ? first : old));
        const firstItem = array[first];
        array[first] = array[second];
        array[second] = firstItem;
        const firstKey = keys[first] ?? nextKey++;
        keys[first] = keys[second] ?? nextKey++;
        keys[second] = firstKey;
        afterChange();
      },
      replace: (items) => {
        remapState(key, () => undefined);
        writePath(values.value, key, [...items]);
        arrayKeys.delete(key);
        afterChange();
      },
    };
  };

  const clearErrors = (path?: string): void => {
    if (path === undefined) errors.value = {};
    else setErrorsFor(path, []);
  };

  const submitWith = async (
    handler?: (value: Output, context: { readonly signal: AbortSignal }) => unknown,
  ): Promise<FormResult<Output>> => {
    submitCount.value += 1;
    submitting.value = true;
    try {
      const result = await validate();
      const signal = formController?.signal ?? new AbortController().signal;
      if (result.status === "valid") {
        const onSubmit = handler ?? options.onSubmit;
        if (onSubmit) await onSubmit(result.value, { signal });
      } else if (result.status === "invalid") {
        const next = new Set(touched.value);
        for (const path of Object.keys(result.errors)) if (path !== "") next.add(path);
        touched.value = next;
      }
      return result;
    } finally {
      submitting.value = false;
    }
  };

  const reset = (next?: Values): void => {
    formController?.abort();
    for (const controller of fieldControllers.values()) controller.abort();
    fieldControllers.clear();
    initial = next === undefined ? createInitial() : cloneValue(next);
    values.value = cloneValue(initial);
    errors.value = {};
    touched.value = new Set();
    pending.value.clear();
    submitCount.value = 0;
    arrayKeys.clear();
    bumpKeys();
  };

  tryOnScopeDispose(() => {
    formController?.abort();
    for (const controller of fieldControllers.values()) controller.abort();
  });

  return {
    values,
    errors,
    valid: computed(() => Object.keys(errors.value).length === 0),
    dirty: computed(() => !deepEqual(values.value, initial)),
    validating: computed(() => pending.value.size > 0),
    submitting,
    submitCount,
    field,
    fieldArray,
    getValue: <Path extends FormPath<Values>>(path: Path) =>
      // Same dotted-path walk as `FormPathValue` (see `field`).
      readPath(values.value, path) as FormPathValue<Values, Path>,
    setValue: (path, value, setOptions) => setValueAt(path, value, setOptions),
    setValues: (next, setOptions = {}) => {
      if (setOptions.merge) {
        values.value = { ...values.value, ...cloneValue(next) };
      } else {
        values.value = { ...cloneValue(initial), ...cloneValue(next) };
      }
      arrayKeys.clear();
      bumpKeys();
    },
    isTouched: (path) => touched.value.has(path),
    setErrors: (next) => {
      const normalized: Record<string, readonly string[]> = {};
      for (const [path, messages] of Object.entries(next)) {
        if (messages !== undefined && messages.length > 0) normalized[path] = messages;
      }
      errors.value = normalized;
    },
    clearErrors,
    validate,
    submit: submitWith,
    handleSubmit: (handler) => (event) => {
      event?.preventDefault();
      return submitWith(handler);
    },
    reset,
  };
}

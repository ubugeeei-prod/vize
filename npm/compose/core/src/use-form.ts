import { computed, ref, shallowRef, toRaw, toValue, triggerRef } from "vue";
import type { Ref } from "vue";

import { dirtyFieldsOf, valuesEqual, valuesKeepingDirty } from "./form-dirty.ts";
import { readPath, writePath } from "./form-value-paths.ts";
import { tryOnScopeDispose } from "./scope.ts";
import { validateStandardSchema } from "./standard-schema.ts";
import type { NormalizedSchemaIssue } from "./standard-schema.ts";
import type {
  FormControls,
  FormFieldArrayControls,
  FormFieldControls,
} from "./form-control-types.ts";
import type {
  FormResetOptions,
  FormResult,
  FormValidationTrigger,
  SetFieldValueOptions,
  UseFormOptions,
} from "./form-options.ts";
import type {
  DirtyFields,
  FormArrayItem,
  FormArrayPath,
  FormPath,
  FormPathValue,
  ValidatorResult,
} from "./form-path-types.ts";

export type * from "./form-control-types.ts";
export type * from "./form-options.ts";
export type * from "./form-path-types.ts";

function cloneValue<Value>(value: Value): Value {
  return structuredClone(toRaw(value));
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
 * can never overwrite fresher errors. `criteriaMode: "first"` keeps one
 * message per path, in source order: schema, form validator, field validator.
 * `dirtyFields` mirrors edited branches, `watch` reads one path, and
 * `setError` updates a single path. `disabled` ignores writes and submission.
 * `reset` can keep dirty values, errors, touched fields, the submit count,
 * or the success flag.
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
  const isSubmitSuccessful = ref(false);
  const disabled = computed(() => toValue(options.disabled) === true);
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
    // Separate buckets keep message order stable: schema, form, then field.
    // A null prototype keeps paths such as `constructor` as ordinary keys.
    const schemaMessages: Record<string, string[]> = Object.create(null);
    const formMessages: Record<string, string[]> = Object.create(null);
    const fieldMessages: Record<string, string[]> = Object.create(null);
    const add = (
      bucket: Record<string, string[]>,
      path: string,
      messages: readonly string[],
    ): void => {
      if (messages.length === 0) return;
      (bucket[path] ??= []).push(...messages);
    };
    let output: unknown = snapshot;
    const tasks: Promise<void>[] = [];
    if (options.schema) {
      const schema = options.schema;
      tasks.push(
        validateStandardSchema(schema, cloneValue(snapshot)).then((result) => {
          if (result.status === "valid") output = result.value;
          else for (const issue of result.issues) add(schemaMessages, issue.path, [issue.message]);
        }),
      );
    }
    if (options.validate) {
      const validate = options.validate;
      tasks.push(
        Promise.resolve(validate(snapshot, { signal })).then((result) => {
          if (!result) return;
          for (const [path, messages] of Object.entries(result)) {
            if (messages !== undefined) add(formMessages, path, toMessages(messages));
          }
        }),
      );
    }
    for (const [path, validator] of validators) {
      tasks.push(
        runValidator(validator, path, snapshot, signal).then((messages) =>
          add(fieldMessages, path, messages),
        ),
      );
    }
    await Promise.all(tasks);
    const collected: Record<string, string[]> = Object.create(null);
    const firstOnly = options.criteriaMode === "first";
    for (const bucket of [schemaMessages, formMessages, fieldMessages]) {
      for (const [path, messages] of Object.entries(bucket)) {
        const target = (collected[path] ??= []);
        if (firstOnly && target.length > 0) continue;
        target.push(...(firstOnly ? messages.slice(0, 1) : messages));
      }
    }
    return { errors: { ...collected }, output };
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
      const combined = [...fromSchema, ...fromForm, ...own];
      const messages = options.criteriaMode === "first" ? combined.slice(0, 1) : combined;
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
    if (disabled.value) return;
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
      dirty: computed(() => !valuesEqual(readPath(values.value, key), readPath(initial, key))),
      touched: computed(() => touched.value.has(key)),
      validating: computed(() => pending.value.has(key) || pending.value.has("")),
      onBlur: () => {
        touch(key);
        if (trigger() === "blur") void validateField(key);
      },
      validate: () => validateField(key),
      reset: () => {
        if (disabled.value) return;
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
      if (disabled.value) return;
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
        if (disabled.value) return;
        const array = readArray(key);
        if (index < 0 || index >= array.length) return;
        const keys = syncKeys(key, array.length);
        remapState(key, (old) => (old === index ? undefined : old > index ? old - 1 : old));
        array.splice(index, 1);
        keys.splice(index, 1);
        afterChange();
      },
      move: (from, to) => {
        if (disabled.value) return;
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
        if (disabled.value) return;
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
        if (disabled.value) return;
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
    if (disabled.value) return { status: "invalid", errors: {} };
    submitCount.value += 1;
    submitting.value = true;
    try {
      const result = await validate();
      const signal = formController?.signal ?? new AbortController().signal;
      if (result.status === "valid") {
        try {
          const onSubmit = handler ?? options.onSubmit;
          if (onSubmit) await onSubmit(result.value, { signal });
          isSubmitSuccessful.value = true;
        } catch (error) {
          isSubmitSuccessful.value = false;
          throw error;
        }
      } else if (result.status === "invalid") {
        isSubmitSuccessful.value = false;
        const next = new Set(touched.value);
        for (const path of Object.keys(result.errors)) if (path !== "") next.add(path);
        touched.value = next;
      }
      return result;
    } finally {
      submitting.value = false;
    }
  };

  const reset = (next?: Values, keep: FormResetOptions = {}): void => {
    formController?.abort();
    for (const controller of fieldControllers.values()) controller.abort();
    fieldControllers.clear();
    const previousInitial = initial;
    const base = next === undefined ? createInitial() : cloneValue(next);
    initial = cloneValue(base);
    values.value = keep.keepDirtyValues
      ? cloneValue(valuesKeepingDirty(base, cloneValue(values.value), previousInitial))
      : cloneValue(initial);
    if (!keep.keepErrors) errors.value = {};
    if (!keep.keepTouched) touched.value = new Set();
    pending.value.clear();
    if (!keep.keepSubmitCount) submitCount.value = 0;
    if (!keep.keepIsSubmitSuccessful) isSubmitSuccessful.value = false;
    if (!keep.keepDirtyValues) arrayKeys.clear();
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
    dirty: computed(() => !valuesEqual(values.value, initial)),
    dirtyFields: computed(() => dirtyFieldsOf(values.value, initial) as DirtyFields<Values>),
    validating: computed(() => pending.value.size > 0),
    submitting,
    submitCount,
    isSubmitSuccessful,
    disabled,
    field,
    fieldArray,
    getValue: <Path extends FormPath<Values>>(path: Path) =>
      // Same dotted-path walk as `FormPathValue` (see `field`).
      readPath(values.value, path) as FormPathValue<Values, Path>,
    watch: <Path extends FormPath<Values>>(path: Path) =>
      computed(() => readPath(values.value, path) as FormPathValue<Values, Path>),
    setValue: (path, value, setOptions) => setValueAt(path, value, setOptions),
    setValues: (next, setOptions = {}) => {
      if (disabled.value) return;
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
    setError: (path, message) => setErrorsFor(path, toMessages(message)),
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

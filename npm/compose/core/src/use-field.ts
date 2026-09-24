import { computed, ref, shallowRef, toRaw, watch } from "vue";
import type { ComputedRef, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import { isStandardSchema, validateStandardSchema } from "./standard-schema.ts";
import type { StandardSchemaV1 } from "./standard-schema.ts";

/** Result of a standalone field validator: messages, one message, or nothing. */
export type FieldRuleResult = string | readonly string[] | null | undefined | void;

/** Context passed to standalone field validators. */
export interface FieldRuleContext {
  /** Aborted when a newer validation supersedes this one. */
  readonly signal: AbortSignal;
}

/** Plain validator for a standalone field. */
export type FieldRule<Value> = (
  value: Value,
  context: FieldRuleContext,
) => FieldRuleResult | Promise<FieldRuleResult>;

/** A plain validator or a Standard Schema v1 validator for the value. */
export type FieldRuleSource<Value> = FieldRule<Value> | StandardSchemaV1<Value, unknown>;

/** When a standalone field validates automatically. */
export type FieldValidationTrigger = "change" | "blur" | "manual";

/** Options for {@link useField}. */
export interface UseFieldOptions<Value> {
  /**
   * Validators run in order; their messages are concatenated.
   *
   * @default []
   */
  readonly rules?: FieldRuleSource<Value> | readonly FieldRuleSource<Value>[];

  /**
   * When validation runs automatically.
   *
   * @default "change"
   */
  readonly validateOn?: FieldValidationTrigger;

  /**
   * Field name exposed for `name` attributes and error summaries.
   *
   * @default ""
   */
  readonly name?: string;
}

/** Reactive state and actions returned by {@link useField}. */
export interface FieldControls<Value> {
  /** Field name. */
  readonly name: string;

  /** Writable value (use with `v-model`). */
  readonly value: Ref<Value>;

  /** Current error messages. */
  readonly errors: Readonly<ShallowRef<readonly string[]>>;

  /** First error message. */
  readonly error: ComputedRef<string | undefined>;

  /** Whether no errors are recorded. */
  readonly valid: ComputedRef<boolean>;

  /** Whether the value differs from the initial value. */
  readonly dirty: ComputedRef<boolean>;

  /** Whether the field was blurred. */
  readonly touched: Readonly<Ref<boolean>>;

  /** Whether a validation is pending. */
  readonly validating: Readonly<Ref<boolean>>;

  /** Mark the field touched (wire to `blur`); validates when configured. */
  readonly onBlur: () => void;

  /**
   * Run every rule, aborting an older validation still in flight.
   *
   * @returns Whether the value is valid (`false` also when superseded).
   */
  readonly validate: () => Promise<boolean>;

  /**
   * Replace errors (for example with server errors).
   *
   * @param errors New messages.
   */
  readonly setErrors: (errors: readonly string[]) => void;

  /**
   * Restore the initial value (or install a new one) and clear state.
   *
   * @param value New initial value.
   */
  readonly reset: (value?: Value) => void;
}

function messagesOf(result: FieldRuleResult): readonly string[] {
  if (result === undefined || result === null || result === "") return [];
  return typeof result === "string" ? [result] : result.filter((message) => message !== "");
}

function sameValue(left: unknown, right: unknown): boolean {
  if (Object.is(left, right)) return true;
  if (left instanceof Date && right instanceof Date) return left.getTime() === right.getTime();
  if (typeof left !== "object" || typeof right !== "object" || left === null || right === null) {
    return false;
  }
  if (Array.isArray(left) !== Array.isArray(right)) return false;
  const leftKeys = Object.keys(left);
  const rightKeys = Object.keys(right);
  return (
    leftKeys.length === rightKeys.length &&
    leftKeys.every((key) => sameValue(Reflect.get(left, key), Reflect.get(right, key)))
  );
}

/**
 * A standalone validated field, independent of any form.
 *
 * Rules may be plain (a)synchronous validators or Standard Schema v1
 * validators (anything with a `~standard` property). A newer validation
 * aborts older ones through their `signal`, and stale results are ignored.
 * With `validateOn: "change"` validation runs after each value change,
 * never during setup, so server-rendered markup matches the first client
 * render. For multi-field state use `useForm`.
 *
 * Server rendering: pure state; nothing validates during setup. Pending
 * validations are aborted when the owning scope stops.
 *
 * @example
 * ```ts
 * const email = useField("", {
 *   name: "email",
 *   rules: [(value) => (value.includes("@") ? undefined : "Invalid email")],
 * });
 * ```
 *
 * @typeParam Value Field value, inferred from `initialValue`.
 * @param initialValue Initial value, or a factory returning it.
 * @param options Rules, trigger, and name.
 * @default options {}
 * @returns Field state and actions.
 */
export function useField<Value>(
  initialValue: Value | (() => Value),
  options: UseFieldOptions<NoInfer<Value>> = {},
): FieldControls<Value> {
  const createInitial = (): Value =>
    initialValue instanceof Function ? initialValue() : structuredClone(toRaw(initialValue));
  let initial = createInitial();
  // Field values are plain data, for which `UnwrapRef<Value>` equals `Value`.
  const value = ref(structuredClone(toRaw(initial))) as Ref<Value>;
  const errors = shallowRef<readonly string[]>([]);
  const touched = ref(false);
  const validating = ref(false);
  const trigger = options.validateOn ?? "change";
  const rules: readonly FieldRuleSource<Value>[] =
    options.rules === undefined
      ? []
      : Array.isArray(options.rules)
        ? options.rules
        : [options.rules];
  let controller: AbortController | undefined;

  const runRule = async (
    rule: FieldRuleSource<Value>,
    current: Value,
    signal: AbortSignal,
  ): Promise<readonly string[]> => {
    if (isStandardSchema(rule)) {
      const result = await validateStandardSchema(rule, current);
      return result.status === "valid" ? [] : result.issues.map((issue) => issue.message);
    }
    return messagesOf(await rule(current, { signal }));
  };

  const validate = async (): Promise<boolean> => {
    controller?.abort(new DOMException("A newer validation started.", "AbortError"));
    const active = new AbortController();
    controller = active;
    validating.value = true;
    try {
      const current = value.value;
      const results = await Promise.all(rules.map((rule) => runRule(rule, current, active.signal)));
      if (active.signal.aborted) return false;
      errors.value = results.flat();
      return errors.value.length === 0;
    } finally {
      if (controller === active) {
        controller = undefined;
        validating.value = false;
      }
    }
  };

  // The value installed by `reset`; its own change must not trigger validation.
  let resetValue: unknown;
  if (trigger === "change") {
    watch(
      value,
      (current) => {
        if (toRaw(current) === resetValue && sameValue(current, initial)) return;
        void validate();
      },
      { deep: true },
    );
  }

  tryOnScopeDispose(() => controller?.abort());

  return {
    name: options.name ?? "",
    value,
    errors,
    error: computed(() => errors.value[0]),
    valid: computed(() => errors.value.length === 0),
    dirty: computed(() => !sameValue(value.value, initial)),
    touched,
    validating,
    onBlur: () => {
      touched.value = true;
      if (trigger === "blur") void validate();
    },
    validate,
    setErrors: (next) => {
      errors.value = [...next];
    },
    reset: (next) => {
      controller?.abort();
      controller = undefined;
      initial = next === undefined ? createInitial() : structuredClone(toRaw(next));
      const fresh = structuredClone(toRaw(initial));
      resetValue = fresh;
      value.value = fresh;
      errors.value = [];
      touched.value = false;
      validating.value = false;
    },
  };
}

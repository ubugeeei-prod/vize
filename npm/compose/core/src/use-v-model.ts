import { computed, ref, shallowRef, toRaw, watch } from "vue";
import type { Ref, WritableComputedRef } from "vue";

/** Emit function able to send `event` with a value of type `Value`. */
export type VModelEmit<Event extends string, Value> = (event: Event, value: Value) => void;

/** Options for {@link useVModel}. */
export interface UseVModelOptions<
  Value,
  Event extends string = string,
  Default extends Exclude<Value, undefined> | undefined = Exclude<Value, undefined>,
> {
  /**
   * Keep a local copy that works even when the parent does not bind
   * `v-model`: writes update the copy and emit, prop changes overwrite it.
   *
   * @default false
   */
  readonly passive?: boolean;

  /**
   * Event name to emit.
   *
   * @default `update:${key}`
   */
  readonly eventName?: Event;

  /**
   * In passive mode, store the local copy deeply reactive and also emit on
   * nested mutations of it.
   *
   * @default false
   */
  readonly deep?: boolean;

  /**
   * Value read while the prop is `undefined`.
   *
   * @default undefined
   */
  readonly defaultValue?: Default;

  /**
   * In passive mode, copy prop values before storing them locally so local
   * mutations never touch the parent's object: `true` uses
   * `structuredClone` on the raw (unproxied) value, a function is used as
   * the cloner.
   *
   * @default false
   */
  readonly clone?: boolean | ((value: Value) => Value);

  /**
   * Veto emitting a value.
   *
   * @default undefined (always emit)
   */
  readonly shouldEmit?: (value: Value) => boolean;
}

/** Model value type: `undefined` is removed when a default is configured. */
export type VModelValue<Value, Default> = [Default] extends [undefined]
  ? Value
  : Exclude<Value, undefined>;

/**
 * Two-way binding helper for a component prop.
 *
 * Reads the prop and emits `update:<key>` (or `eventName`) on write, typed
 * end to end: the key must be a prop, the emitted value has the prop's type,
 * and `emit` must accept that exact event (a `defineEmits` result with the
 * matching overload qualifies). With `passive: true` a local ref mirrors the
 * prop so the component also works uncontrolled. Relies only on the props
 * and emit you pass (no component internals), so it is SSR- and
 * Vapor-friendly; the passive watchers follow the owning scope.
 *
 * @example
 * ```ts
 * const props = defineProps<{ modelValue: string }>();
 * const emit = defineEmits<{ "update:modelValue": [value: string] }>();
 * const model = useVModel(props, "modelValue", emit);
 * model.value = "next"; // emits update:modelValue
 * ```
 *
 * @param props Component props object.
 * @param key Prop to bind.
 * @param emit Component emit function.
 * @param options Passive mode, event name, default, clone, and emit guard.
 * @default options {}
 * @returns A writable ref over the prop.
 */
export function useVModel<
  Props extends object,
  Key extends keyof Props & string,
  Event extends string = `update:${Key}`,
  Default extends Exclude<Props[Key], undefined> | undefined = undefined,
>(
  props: Props,
  key: Key,
  emit: VModelEmit<NoInfer<Event>, NoInfer<Props[Key]>>,
  options?: UseVModelOptions<Props[Key], Event, Default>,
): Ref<VModelValue<Props[Key], Default>>;
export function useVModel(
  props: Readonly<Record<string, unknown>>,
  key: string,
  emit: VModelEmit<string, unknown>,
  options: UseVModelOptions<unknown, string, unknown> = {},
): Ref<unknown> | WritableComputedRef<unknown> {
  const event = options.eventName ?? `update:${key}`;
  const read = (): unknown => props[key] ?? options.defaultValue;
  const send = (value: unknown): void => {
    if (options.shouldEmit?.(value) ?? true) emit(event, value);
  };

  if (!(options.passive ?? false)) {
    return computed({ get: read, set: send });
  }

  const clone =
    options.clone === true
      ? (value: unknown): unknown => structuredClone(toRaw(value))
      : typeof options.clone === "function"
        ? options.clone
        : (value: unknown): unknown => value;
  const deep = options.deep ?? false;
  const local: Ref<unknown> = deep ? ref(clone(read())) : shallowRef(clone(read()));
  let fromProp = false;

  watch(read, (value) => {
    fromProp = true;
    local.value = clone(value);
    fromProp = false;
  });
  watch(
    local,
    (value) => {
      if (!fromProp && (deep || value !== props[key])) send(value);
    },
    { deep, flush: "sync" },
  );
  return local;
}

/** Options for {@link useVModels}. */
export interface UseVModelsOptions<Key extends string> extends Omit<
  UseVModelOptions<unknown>,
  "eventName" | "defaultValue" | "shouldEmit" | "clone"
> {
  /**
   * Props to bind. Defaults to every prop, which then requires an
   * `update:<prop>` emit for each of them.
   *
   * @default all prop keys
   */
  readonly keys?: readonly Key[];
}

/** Turn a union into the intersection of its members. */
type UnionToIntersection<Union> = (Union extends unknown ? (value: Union) => void : never) extends (
  value: infer Intersection,
) => void
  ? Intersection
  : never;

/** Emit overloads required by {@link useVModels} for the bound keys. */
export type VModelsEmit<Props, Key extends keyof Props & string> = UnionToIntersection<
  { [Name in Key]: VModelEmit<`update:${Name}`, Props[Name]> }[Key]
>;

/**
 * {@link useVModel} for several props at once.
 *
 * Returns one writable ref per bound prop, each emitting its own
 * `update:<prop>` event. `emit` must declare an update event for every bound
 * key; restrict the set with `keys`.
 *
 * @example
 * ```ts
 * const { open, value } = useVModels(props, emit, { keys: ["open", "value"] });
 * ```
 *
 * @param props Component props object.
 * @param emit Component emit function.
 * @param options Passive mode, depth, and the keys to bind.
 * @default options {}
 * @returns One writable ref per bound prop.
 */
export function useVModels<
  Props extends object,
  const Key extends keyof Props & string = keyof Props & string,
>(
  props: Props,
  emit: NoInfer<VModelsEmit<Props, Key>>,
  options?: UseVModelsOptions<Key>,
): { readonly [Name in Key]: Ref<Props[Name]> };
export function useVModels(
  props: Readonly<Record<string, unknown>>,
  emit: VModelEmit<`update:${string}`, unknown>,
  options: UseVModelsOptions<string> = {},
): { readonly [key: string]: Ref<unknown> } {
  const keys = options.keys ?? Object.keys(props);
  const models: Record<string, Ref<unknown> | WritableComputedRef<unknown>> = {};
  for (const key of keys) {
    models[key] = useVModel(props, key, emit, {
      ...(options.passive === undefined ? {} : { passive: options.passive }),
      ...(options.deep === undefined ? {} : { deep: options.deep }),
    });
  }
  return models;
}

import { computed, shallowRef } from "vue";
import type { ComponentPublicInstance, ComputedRef, ShallowRef } from "vue";

/** Union of the parameter tuples of every overload of `Emit` (up to eight). */
export type EmitOverloadArgs<Emit> = Emit extends {
  (...args: infer A1): unknown;
  (...args: infer A2): unknown;
  (...args: infer A3): unknown;
  (...args: infer A4): unknown;
  (...args: infer A5): unknown;
  (...args: infer A6): unknown;
  (...args: infer A7): unknown;
  (...args: infer A8): unknown;
}
  ? A1 | A2 | A3 | A4 | A5 | A6 | A7 | A8
  : Emit extends (...args: infer Args) => unknown
    ? Args
    : never;

/** Event names accepted by an `emit` function (from `defineEmits`). */
export type EmitEventName<Emit> =
  EmitOverloadArgs<Emit> extends infer Args
    ? Args extends readonly [infer Name extends string, ...unknown[]]
      ? Name
      : never
    : never;

/** Payload tuple of one event of an `emit` function. */
export type EmitEventArgs<Emit, Name extends string> =
  EmitOverloadArgs<Emit> extends infer Args
    ? Args extends readonly [Name, ...infer Payload]
      ? Payload
      : never
    : never;

type Camelize<Value extends string> = Value extends `${infer Head}-${infer Tail}`
  ? `${Head}${Capitalize<Camelize<Tail>>}`
  : Value;

/** Handler prop key Vue uses for an event (`update:modelValue` → `onUpdate:modelValue`). */
export type EmitHandlerKey<Name extends string> = `on${Capitalize<Camelize<Name>>}`;

/** `onX` handler props that re-emit the named events. */
export type EmitsAsProps<Emit, Names extends string> = {
  readonly [Name in Names as EmitHandlerKey<Name>]: (...payload: EmitEventArgs<Emit, Name>) => void;
};

/** Props with `undefined` values removed (so the child's defaults apply). */
export type ForwardedProps<Props extends object> = {
  readonly [Key in keyof Props]?: Exclude<Props[Key], undefined>;
};

/**
 * Convert an event name to the handler prop key Vue uses.
 *
 * @param name Event name, e.g. `update:modelValue` or `value-change`.
 * @returns Handler key, e.g. `onUpdate:modelValue` or `onValueChange`.
 */
export function toHandlerKey<const Name extends string>(name: Name): EmitHandlerKey<Name>;
export function toHandlerKey(name: string): string {
  const camelized = name.replace(/-(\w)/gu, (_match, letter: string) => letter.toUpperCase());
  return `on${camelized.charAt(0).toUpperCase()}${camelized.slice(1)}`;
}

/**
 * Turn selected events of the current component into `onX` props that
 * re-emit them, for forwarding to a wrapped child.
 *
 * Instance-free: the event list is explicit (no `getCurrentInstance()`), and
 * both names and payloads are inferred from the `emit` returned by
 * `defineEmits`, so forwarding a misspelled event is a compile error.
 *
 * @param emit Emit function of the wrapper.
 * @param events Event names to forward.
 * @returns Handler props to bind on the child (`v-bind`).
 */
export function useEmitAsProps<
  Emit extends (...args: never[]) => unknown,
  const Names extends readonly EmitEventName<Emit>[],
>(emit: Emit, events: Names): EmitsAsProps<Emit, Names[number]>;
export function useEmitAsProps(
  emit: (event: string, ...payload: unknown[]) => unknown,
  events: readonly string[],
): Readonly<Record<string, (...payload: unknown[]) => void>> {
  const handlers: Record<string, (...payload: unknown[]) => void> = {};
  for (const event of events) {
    handlers[toHandlerKey(event)] = (...payload) => {
      emit(event, ...payload);
    };
  }
  return Object.freeze(handlers);
}

/**
 * Forward a wrapper's props to a child, dropping `undefined` values so the
 * child's own defaults still apply.
 *
 * Note: Vue casts absent boolean props to `false`; declare such props with
 * `default: undefined` on the wrapper to keep them forwardable as "unset".
 *
 * @param props Reactive props object of the wrapper.
 * @returns Computed defined props.
 */
export function useForwardProps<const Props extends object>(
  props: Props,
): ComputedRef<ForwardedProps<Props>>;
export function useForwardProps(props: object): ComputedRef<Readonly<Record<string, unknown>>> {
  return computed(() => {
    const forwarded: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(props)) {
      if (value !== undefined) forwarded[key] = value;
    }
    return forwarded;
  });
}

/**
 * {@link useForwardProps} and {@link useEmitAsProps} combined into one object
 * to `v-bind` on the wrapped child.
 *
 * @param props Reactive props object of the wrapper.
 * @param emit Emit function of the wrapper.
 * @param events Event names to forward.
 * @returns Computed props plus handler props.
 */
export function useForwardPropsEmits<
  const Props extends object,
  Emit extends (...args: never[]) => unknown,
  const Names extends readonly EmitEventName<Emit>[],
>(
  props: Props,
  emit: Emit,
  events: Names,
): ComputedRef<ForwardedProps<Props> & EmitsAsProps<Emit, Names[number]>>;
export function useForwardPropsEmits(
  props: object,
  emit: (event: string, ...payload: unknown[]) => unknown,
  events: readonly string[],
): ComputedRef<Readonly<Record<string, unknown>>> {
  const forwarded = useForwardProps(props);
  const handlers = useEmitAsProps(emit, events);
  return computed(() => ({ ...forwarded.value, ...handlers }));
}

/** Exposed object of a wrapper that forwards its child's exposed API. */
export type ForwardedExpose<Exposed extends object> = Readonly<Partial<Exposed>> & {
  /** Root element of the forwarded child (or the element itself). */
  readonly $el: Element | null;
};

/** State returned by {@link useForwardExpose}. */
export interface ForwardExposeControls<Exposed extends object> {
  /** Bind with `:ref="forwardRef"` on the wrapped child or element. */
  readonly forwardRef: (value: Element | ComponentPublicInstance | null) => void;
  /** Wrapped child (component instance or element). */
  readonly current: Readonly<ShallowRef<Element | ComponentPublicInstance | null>>;
  /** Pass to `defineExpose(exposed)` to re-expose the child's API. */
  readonly exposed: ForwardedExpose<Exposed>;
}

/**
 * Re-expose a wrapped child's public API (and root element) from a wrapper.
 *
 * ```ts
 * const { forwardRef, exposed } = useForwardExpose<{ focus: () => void }>();
 * defineExpose(exposed);
 * ```
 *
 * Instance-free: the child is captured through a function ref and read
 * lazily, so the exposed object always reflects the currently mounted child.
 * Methods of a wrapped element are bound to it. Properties are `undefined`
 * while nothing is mounted (always during SSR).
 *
 * @returns Function ref, current child, and the exposed proxy.
 */
export function useForwardExpose<Exposed extends object = {}>(): ForwardExposeControls<Exposed> {
  const current = shallowRef<Element | ComponentPublicInstance | null>(null);
  const root = (): Element | null => {
    const value = current.value;
    if (value === null) return null;
    if ("$el" in value) return isElement(value.$el) ? value.$el : null;
    return value;
  };
  const exposed = new Proxy<Record<string | symbol, unknown>>(
    {},
    {
      get: (_target, key) => {
        if (key === "$el") return root();
        const value = current.value;
        if (value === null || !(key in value)) return undefined;
        const property: unknown = Reflect.get(value, key);
        return typeof property === "function" && !("$el" in value)
          ? (...args: unknown[]) => Reflect.apply(property, value, args)
          : property;
      },
      has: (_target, key) => key === "$el" || (current.value !== null && key in current.value),
    },
  );
  return {
    forwardRef: (value) => {
      current.value = value;
    },
    current,
    // The proxy implements exactly the ForwardedExpose contract at runtime.
    exposed: exposed as ForwardedExpose<Exposed>,
  };
}

function isElement(value: unknown): value is Element {
  return typeof value === "object" && value !== null && "nodeType" in value && value.nodeType === 1;
}

import { isRef, reactive, unref } from "vue";
import type { MaybeRef, Reactive } from "vue";

/**
 * Reactive object view over a ref that holds an object.
 *
 * Property reads and writes go to the ref's current object, so replacing the
 * ref's value swaps the whole object behind a stable reactive proxy (handy
 * for `v-bind="state"` or destructuring-free templates). Nested refs are
 * unwrapped on read and written through on assignment. A non-ref object is
 * simply passed to `reactive`. SSR-safe and nothing to dispose.
 *
 * @example
 * ```ts
 * const state = ref({ name: "a" });
 * const view = toReactive(state);
 * state.value = { name: "b" };
 * view.name; // "b"
 * ```
 *
 * @param objectRef Ref (or plain object) holding the object.
 * @returns A reactive proxy following the ref.
 */
export function toReactive<Value extends object>(objectRef: MaybeRef<Value>): Reactive<Value> {
  if (!isRef(objectRef)) return reactive(objectRef);

  const proxy = new Proxy<Value>(objectRef.value, {
    get: (_target, property, receiver) => unref(Reflect.get(objectRef.value, property, receiver)),
    set: (_target, property, value) => {
      const current: unknown = Reflect.get(objectRef.value, property);
      if (isRef(current) && !isRef(value)) current.value = value;
      else Reflect.set(objectRef.value, property, value);
      return true;
    },
    deleteProperty: (_target, property) => Reflect.deleteProperty(objectRef.value, property),
    has: (_target, property) => Reflect.has(objectRef.value, property),
    ownKeys: () => Reflect.ownKeys(objectRef.value),
    getOwnPropertyDescriptor: (_target, property) => {
      const descriptor = Reflect.getOwnPropertyDescriptor(objectRef.value, property);
      return descriptor === undefined ? undefined : { ...descriptor, configurable: true };
    },
  });
  return reactive(proxy);
}

import { Comment, Fragment, Text, isVNode, normalizeClass, normalizeStyle } from "vue";
import type { Slot, StyleValue } from "vue";

/** Prop names treated as event handlers (`onClick`, `onUpdate:modelValue`, ...). */
export type HandlerKey = `on${Capitalize<string>}`;

type Simplify<Value> = { [Key in keyof Value]: Value[Key] } & {};

type MergeValue<Key, Left, Right> = Key extends HandlerKey
  ? Left | Right
  : undefined extends Right
    ? Left | Exclude<Right, undefined>
    : Right;

type MergeTwo<Left, Right> = Simplify<{
  [Key in keyof Left | keyof Right]: Key extends "class"
    ? string
    : Key extends "style"
      ? StyleValue
      : Key extends keyof Right
        ? Key extends keyof Left
          ? MergeValue<Key, Left[Key], Right[Key]>
          : Right[Key]
        : Key extends keyof Left
          ? Left[Key]
          : never;
}>;

/**
 * Result type of {@link mergeProps}: handlers become unions of every source's
 * handler, `class` becomes a string, `style` a `StyleValue`, and other keys
 * take the last defined value.
 */
export type MergedProps<Sources extends readonly object[]> = Sources extends readonly [
  infer First extends object,
  ...infer Rest extends readonly object[],
]
  ? Rest extends readonly []
    ? MergeTwo<{}, First>
    : MergeTwo<First, MergedProps<Rest>> extends infer Merged
      ? Merged
      : never
  : {};

/**
 * Merge prop objects for forwarding onto one element or component.
 *
 * - `class` values are normalized and joined (left to right).
 * - `style` values are normalized and merged (later declarations win).
 * - Event handlers (`onX`) are chained: every handler runs, in source order.
 * - Other keys take the last value that is not `undefined`, so optional
 *   forwarded props never erase a defined value.
 *
 * Pure and SSR-safe.
 *
 * @param sources Prop objects, lowest priority first.
 * @returns A new merged object.
 */
export function mergeProps<const Sources extends readonly object[]>(
  ...sources: Sources
): MergedProps<Sources>;
export function mergeProps(...sources: readonly object[]): Record<string, unknown> {
  const merged: Record<string, unknown> = {};
  const classes: unknown[] = [];
  const styles: unknown[] = [];
  const handlers = new Map<string, unknown[]>();

  for (const source of sources) {
    for (const [key, value] of Object.entries(source)) {
      if (key === "class") {
        classes.push(value);
      } else if (key === "style") {
        styles.push(value);
      } else if (
        isHandlerKey(key) &&
        (typeof value === "function" ||
          (Array.isArray(value) && value.every((handler) => typeof handler === "function")))
      ) {
        const list = handlers.get(key) ?? [];
        if (Array.isArray(value)) list.push(...value);
        else list.push(value);
        handlers.set(key, list);
      } else if (value !== undefined || !Object.hasOwn(merged, key)) {
        merged[key] = value;
      }
    }
  }

  if (classes.length > 0) merged["class"] = normalizeClass(classes);
  if (styles.length > 0) merged["style"] = normalizeStyle(styles);
  for (const [key, list] of handlers) {
    merged[key] =
      list.length === 1
        ? list[0]
        : (...args: unknown[]) => {
            for (const handler of list) {
              if (typeof handler === "function") Reflect.apply(handler, undefined, args);
            }
          };
  }
  return merged;
}

/**
 * Whether a prop key names an event handler (`on` followed by an upper-case
 * letter or `:`-namespaced event).
 *
 * @param key Prop key.
 * @returns Whether the key is a handler key.
 */
export function isHandlerKey(key: string): key is HandlerKey {
  return key.length > 2 && key.startsWith("on") && !/^[a-z]$/u.test(key.charAt(2));
}

/**
 * Whether a slot renders meaningful content.
 *
 * Invokes the slot and ignores comments (from `v-if`), whitespace-only text,
 * and empty fragments, recursing into fragments. Use it to skip wrapper
 * elements for empty slots. Calling a slot during render is safe on the
 * server; call it from render (or a computed read during render) so the
 * component tracks the slot's dependencies.
 *
 * @param slot Slot function (possibly undefined).
 * @param props Slot props.
 * @default props undefined
 * @returns Whether the slot produced at least one non-empty node.
 */
export function hasSlotContent<Props>(slot: Slot<Props> | undefined, props?: Props): boolean {
  if (!slot) return false;
  const nodes: unknown = props === undefined ? Reflect.apply(slot, undefined, []) : slot(props);
  return containsContent(nodes);
}

/**
 * Names of the slots that were provided and render content.
 *
 * @param slots Slots object (e.g. from `useSlots()`).
 * @returns Slot names with content, in declaration order.
 */
export function presentSlotNames<Name extends string>(
  slots: Readonly<Partial<Record<Name, Slot | undefined>>>,
): Name[] {
  const names: Name[] = [];
  for (const name of Object.keys(slots)) {
    if (isName<Name>(slots, name) && hasSlotContent(slots[name])) names.push(name);
  }
  return names;
}

function isName<Name extends string>(
  slots: Readonly<Partial<Record<Name, unknown>>>,
  key: string,
): key is Name {
  return Object.hasOwn(slots, key);
}

function containsContent(value: unknown): boolean {
  if (value === null || value === undefined || typeof value === "boolean") return false;
  if (Array.isArray(value)) return value.some(containsContent);
  if (typeof value === "string") return value.trim().length > 0;
  if (typeof value === "number") return true;
  if (!isVNode(value)) return false;
  if (value.type === Comment) return false;
  if (value.type === Text)
    return typeof value.children === "string" && value.children.trim().length > 0;
  if (value.type === Fragment) return containsContent(value.children);
  return true;
}

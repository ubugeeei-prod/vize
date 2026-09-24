import { toValue } from "vue";
import type { ComponentPublicInstance, MaybeRefOrGetter } from "vue";

/**
 * A concrete value that can identify a DOM element.
 *
 * Template refs on native elements resolve to the element itself, while refs
 * on components resolve to the component's public instance whose root node
 * is exposed as `$el`. `null` and `undefined` represent a target that is not
 * mounted yet (including every server render).
 */
export type ElementTargetValue<TargetElement extends Element = Element> =
  | TargetElement
  | ComponentPublicInstance
  | null
  | undefined;

/**
 * Reactive element target accepted by every element-based composable.
 *
 * Accepts a plain value, a ref (such as `useTemplateRef()`), or a getter.
 */
export type MaybeElementTarget<TargetElement extends Element = Element> = MaybeRefOrGetter<
  ElementTargetValue<TargetElement>
>;

/**
 * Reactive list of element targets for observers that watch several nodes.
 *
 * Each item follows {@link MaybeElementTarget} semantics; unresolved items
 * are skipped.
 */
export type MaybeElementTargets<TargetElement extends Element = Element> =
  | MaybeElementTarget<TargetElement>
  | MaybeRefOrGetter<readonly ElementTargetValue<TargetElement>[]>;

/**
 * Whether a value is a DOM element node.
 *
 * Uses the `nodeType` contract instead of `instanceof Element`, so the check
 * reads no browser globals and works across realms (iframes, happy-dom,
 * jsdom).
 *
 * @param value Candidate value.
 * @returns Whether `value` is an element node.
 */
export function isElementNode(value: unknown): value is Element {
  return (
    typeof value === "object" &&
    value !== null &&
    "nodeType" in value &&
    value.nodeType === 1 &&
    "getBoundingClientRect" in value
  );
}

/**
 * Resolve a reactive element target to its concrete element.
 *
 * Reads the reactive source once (tracking it when called inside an effect).
 * Component instances resolve to their root element; fragment or text roots
 * resolve to `null`. Never throws and reads no browser globals, so it is safe
 * during server rendering, where it always returns `null`.
 *
 * @param target Reactive element target.
 * @returns The resolved element, or `null` while unresolved.
 */
export function resolveElement(target: MaybeElementTarget): Element | null {
  return elementFromValue(toValue(target));
}

function elementFromValue(value: unknown): Element | null {
  if (isElementNode(value)) return value;
  if (typeof value === "object" && value !== null && "$el" in value) {
    const root: unknown = value.$el;
    return isElementNode(root) ? root : null;
  }
  return null;
}

/**
 * Resolve a reactive list (or single) element target to its concrete elements.
 *
 * Unresolved entries are dropped and duplicates are removed while keeping the
 * first-seen order, so observers never register the same node twice.
 *
 * @param targets Reactive element target or target list.
 * @returns The resolved, de-duplicated elements.
 */
export function resolveElements(targets: MaybeElementTargets): Element[] {
  const value: unknown = toValue(targets);
  const items: readonly unknown[] = Array.isArray(value) ? value : [value];
  const elements: Element[] = [];
  for (const item of items) {
    const element = elementFromValue(item);
    if (element && !elements.includes(element)) elements.push(element);
  }
  return elements;
}

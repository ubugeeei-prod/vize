// Surface-oracle fixtures and observers beside the pinned runtime. The public
// JSX child-values source custody freezes runtime.ts, so additions live here.
import assert from "node:assert/strict";
import { Vue, componentFixtures } from "./runtime.ts";

const FixtureInput = Vue.defineComponent({
  props: {
    modelValue: { type: String, required: true },
  },
  emits: ["update:modelValue"],
  setup(props, { emit }) {
    return () =>
      Vue.h("input", {
        "data-fixture": "input",
        value: props.modelValue,
        onInput: (event: Event) =>
          emit("update:modelValue", (event.target as HTMLInputElement).value),
      });
  },
});

// Linked fixtures resolve free identifiers in the global scope, like the DOM
// globals runtime.ts installs; its pinned linker binds only `FixtureCard`.
Object.assign(globalThis, { FixtureInput });
const surfaceFixtures = { ...componentFixtures, FixtureInput };

interface ObservedNode {
  readonly nodeType: number;
  readonly data?: string;
  readonly localName?: string;
  readonly attributes?: Iterable<{ readonly name: string; readonly value: string }>;
  readonly childNodes: Iterable<ObservedNode>;
  readonly value?: string;
  readonly checked?: boolean;
}
type Observation =
  | string
  | {
      tag: string | undefined;
      attributes: Record<string, string>;
      children: Observation[];
      value?: string;
      checked?: boolean;
    };

// Normalized DOM: adjacent text merges, empty text and comments drop, attributes sort.
function observeChildren(parent: ObservedNode): Observation[] {
  const children: Observation[] = [];
  for (const node of parent.childNodes) {
    if (node.nodeType === 3) {
      if (!node.data) continue;
      const last = children.length - 1;
      if (typeof children[last] === "string") children[last] += node.data;
      else children.push(node.data);
      continue;
    }
    if (node.nodeType !== 1) continue;
    const attributes = Object.fromEntries(
      [...(node.attributes ?? [])]
        .map(({ name, value }) => [name, value])
        .sort(([left], [right]) => left!.localeCompare(right!)),
    );
    const element: Observation = {
      tag: node.localName,
      attributes,
      children: observeChildren(node),
    };
    if (node.localName === "input") {
      element.value = node.value;
      element.checked = node.checked;
    }
    children.push(element);
  }
  return children;
}

// Corpus values use `{ "$jsxValue": … }` descriptors for inputs JSON cannot spell.
function materialize(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(materialize);
  if (!value || typeof value !== "object") return value;
  if (Object.hasOwn(value, "$jsxValue")) {
    assert.deepEqual(Object.keys(value), ["$jsxValue"]);
    switch ((value as { $jsxValue: unknown }).$jsxValue) {
      case "zero":
        return 0;
      case "empty-string":
        return "";
      case "nan":
        return Number.NaN;
      case "null":
        return null;
      case "undefined":
        return undefined;
      case "false":
        return false;
      case "true":
        return true;
      case "array":
        return [0, "", Number.NaN, null, undefined, false];
      case "nested-vnode":
        return Vue.h("i", { "data-kind": "nested" }, [
          Vue.h("b", null, "inner"),
          Vue.h("u", null, "leaf"),
        ]);
      default:
        assert.fail(
          `unknown JSX value descriptor: ${String((value as { $jsxValue: unknown }).$jsxValue)}`,
        );
    }
  }
  return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, materialize(item)]));
}

export { surfaceFixtures, materialize, observeChildren };

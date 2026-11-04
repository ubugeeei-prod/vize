import assert from "node:assert/strict";
import {
  assertCompilerPatchSuccessor,
  compilerPatchSuccessor,
} from "./compiler-patch-successors.ts";

type RouterCase = "vue-router-clean" | "vue-router-broken";

const previousProps = "  props: { disabled: {\n    type: Boolean,\n    required: false\n  } },";

const previousReturnKeys = [
  "useAttrs",
  "currentRoute",
  "navigate",
  "START_LOCATION",
  "useLink",
  "useRoute",
  "isActive",
  "isExternalLink",
  "classes",
  "computed",
  "route",
  "isExactActive",
  "attrs",
  "href",
];
const importedReturnKeys = [
  "href",
  "currentRoute",
  "isExactActive",
  "START_LOCATION",
  "useLink",
  "navigate",
  "useRoute",
  "isActive",
  "isExternalLink",
  "classes",
  "computed",
  "route",
  "attrs",
  "useAttrs",
];
const returnBlock = (keys: string[]) => `    return {\n      ${keys.join(",\n      ")}\n    };`;

// Authored from the pinned RouterLinkOptions and RouterLinkProps declarations.
// RouteLocationRaw contains generic branches without a runtime constructor.
export const importedRouterProps = `  props: {
    to: {
      type: null,
      required: true
    },
    replace: {
      type: Boolean,
      required: false
    },
    custom: {
      type: Boolean,
      required: false
    },
    activeClass: {
      type: String,
      required: false
    },
    exactActiveClass: {
      type: String,
      required: false
    },
    ariaCurrentValue: {
      type: String,
      required: false
    },
    viewTransition: {
      type: Boolean,
      required: false
    },
    disabled: {
      type: Boolean,
      required: false
    }
  },`;

export function routerImportedPropsSuccessor(id: RouterCase) {
  const original = compilerPatchSuccessor(id);
  assert.equal(original.current.code.split(previousProps).length, 2);
  let code = original.current.code.replace(previousProps, importedRouterProps);
  if (id === "vue-router-broken") {
    // collect_setup_bindings walks the fixed FxHashMap after excluding Props.
    // Adding the real imported props changes its buckets. The complete d204
    // producer packet fixes this one permutation, without admitting other orders.
    assert.deepEqual([...importedReturnKeys].sort(), [...previousReturnKeys].sort());
    const previous = returnBlock(previousReturnKeys);
    assert.equal(code.split(previous).length, 2);
    code = code.replace(previous, returnBlock(importedReturnKeys));
  }
  const output = {
    ...original.current,
    code,
  };
  return { original, output, raw: JSON.stringify(output, null, 2) };
}

export function assertRouterImportedPropsSuccessor(
  id: RouterCase,
  actualRaw: string,
  pins: { packetSha256: string; codeSha256: string },
): void {
  const expected = routerImportedPropsSuccessor(id);
  assertCompilerPatchSuccessor(id, expected.original.currentRaw, pins);
  assert.equal(
    actualRaw,
    expected.raw,
    "the complete packet changes only props and the exact broken-template return permutation",
  );
}

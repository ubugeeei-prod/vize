import assert from "node:assert/strict";
import {
  assertCompilerPatchSuccessor,
  compilerPatchSuccessor,
} from "./compiler-patch-successors.ts";

type RouterCase = "vue-router-clean" | "vue-router-broken";

const previousProps = "  props: { disabled: {\n    type: Boolean,\n    required: false\n  } },";

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
  const output = {
    ...original.current,
    code: original.current.code.replace(previousProps, importedRouterProps),
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
    "the complete packet changes only the authored props block",
  );
}

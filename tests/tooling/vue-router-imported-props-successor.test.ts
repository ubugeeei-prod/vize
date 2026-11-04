import assert from "node:assert/strict";
import { test } from "node:test";
import {
  assertRouterImportedPropsSuccessor,
  importedRouterProps,
  routerImportedPropsSuccessor,
} from "../_helpers/vue-router-imported-props-successor.ts";

for (const id of ["vue-router-clean", "vue-router-broken"] as const) {
  test(`${id} preserves every original packet field and code byte outside runtime props`, () => {
    const { original, output, raw } = routerImportedPropsSuccessor(id);
    const pins = {
      packetSha256: original.review.previousSha256,
      codeSha256: original.review.previousCodeSha256,
    };
    assert.equal(output.code.split(importedRouterProps).length, 2);
    assert.deepEqual({ ...output, code: original.current.code }, original.current);
    assertRouterImportedPropsSuccessor(id, raw, pins);
    for (const changed of [
      { ...output, code: output.code.replace("required: true", "required: false") },
      { ...output, code: output.code + "\n" },
      { ...output, errors: ["different error"] },
      { ...output, unknownFacet: true },
    ]) {
      assert.throws(() =>
        assertRouterImportedPropsSuccessor(id, JSON.stringify(changed, null, 2), pins),
      );
    }
    assert.throws(() => assertRouterImportedPropsSuccessor(id, original.currentRaw, pins));
    assert.throws(() =>
      assertRouterImportedPropsSuccessor(id, raw, { ...pins, packetSha256: "0".repeat(64) }),
    );
  });
}

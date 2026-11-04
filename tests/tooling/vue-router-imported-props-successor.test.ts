import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import {
  assertRouterImportedPropsSuccessor,
  importedRouterProps,
  routerImportedPropsSuccessor,
} from "../_helpers/vue-router-imported-props-successor.ts";

for (const id of ["vue-router-clean", "vue-router-broken"] as const) {
  test(`${id} preserves the whole packet outside the finite props and return changes`, () => {
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
    if (id === "vue-router-broken") {
      const reordered = {
        ...output,
        code: output.code.replace(
          "      href,\n      currentRoute,",
          "      currentRoute,\n      href,",
        ),
      };
      assert.notEqual(reordered.code, output.code);
      assert.throws(() =>
        assertRouterImportedPropsSuccessor(id, JSON.stringify(reordered, null, 2), pins),
      );
    }
    assert.throws(() =>
      assertRouterImportedPropsSuccessor(id, raw, { ...pins, packetSha256: "0".repeat(64) }),
    );
  });
}

test("the broken successor equals the complete source producer packet", () => {
  const directory = new URL("../_fixtures/vue-router-imported-props-11794/", import.meta.url);
  const raw = fs.readFileSync(new URL("broken.current.json", directory), "utf8");
  assert.equal(new TextEncoder().encode(raw).byteLength, 2217);
  assert.equal(
    createHash("sha256").update(raw).digest("hex"),
    "9938ebe8a1628dee67ae443d4ebba6c921d459284f88458b3bb66900f792f279",
  );
  assert.equal(routerImportedPropsSuccessor("vue-router-broken").raw, raw);
});

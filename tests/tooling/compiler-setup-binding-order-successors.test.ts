import assert from "node:assert/strict";
import { test } from "node:test";
import { compilerPatchSuccessor } from "../_helpers/compiler-patch-successors.ts";
import {
  assertSetupBindingOrderSuccessor,
  setupBindingOrderSuccessor,
  verifySetupBindingOrderPair,
} from "../_helpers/compiler-setup-binding-order-successors.ts";

const id = "vue-router-broken";
const pins = () => {
  const { review } = compilerPatchSuccessor(id);
  return { packetSha256: review.previousSha256, codeSha256: review.previousCodeSha256 };
};

test("the #12639 successor moves only the setup return bindings of the authored-piece packet", () => {
  const { previous, output, raw } = setupBindingOrderSuccessor(id);
  verifySetupBindingOrderPair(id, previous, output);
  assertSetupBindingOrderSuccessor(id, raw, pins());
  assert.equal(output.code.split("return {").length, 2);
});

test("the #12639 successor rejects other code, packet fields and raw bytes", () => {
  const { previous, output, raw } = setupBindingOrderSuccessor(id);
  const line = output.code.split("\n").find((entry) => entry.includes("return {"));
  assert.ok(line);
  for (const code of [
    output.code.replace(line, line.replace("useAttrs, ", "useAttrs, useAttrs, ")),
    output.code.replace(line, line.replace("href", "hrefs")),
    output.code.replace(line, line.replace(", href }", " }")),
    output.code.replace("__name: 'AppLink'", "__name: 'AppLinks'"),
    previous.code,
  ]) {
    assert.throws(() => verifySetupBindingOrderPair(id, previous, { ...output, code }));
  }
  for (const field of Object.keys(output)) {
    const missing = { ...output } as Partial<typeof output>;
    delete missing[field as keyof typeof output];
    for (const mutation of [
      { ...output, [field]: "unrelated" },
      missing,
      { ...output, extra: 1 },
    ]) {
      assert.throws(() => verifySetupBindingOrderPair(id, previous, mutation as typeof output));
    }
  }
  for (const altered of [raw + "\n", JSON.stringify(output), raw.replace("__name", "__NAME")]) {
    assert.throws(() => assertSetupBindingOrderSuccessor(id, altered, pins()));
  }
  assert.throws(() => assertSetupBindingOrderSuccessor(id, raw, { codeSha256: "0".repeat(64) }));
});

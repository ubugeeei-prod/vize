import assert from "node:assert/strict";
import { test } from "node:test";
import { canonicalMappedGenerated } from "../../tools/benchmarks/scripts/canon-slot-constructor-audit.ts";

test("slot constructor audit canonicalizes only the certified required-slot helper rename", () => {
  assert.equal(
    canonicalMappedGenerated(
      '  const __vize_required_slots_0: $RequiredSlots<$SlotContract_0, { readonly "default": true; }> = {};\n',
    ),
    '  const __vize_required_slots_0: __VizeRequiredSlots<__VizeSlotContract_0, { readonly "default": true; }> = {};\n',
  );
  assert.equal(
    canonicalMappedGenerated(
      '      const __vize_required_slots_17: $RequiredSlots<$SlotContract_17, { readonly "before": true; }> = {};\n',
    ),
    '      const __vize_required_slots_17: __VizeRequiredSlots<__VizeSlotContract_17, { readonly "before": true; }> = {};\n',
  );
});

test("slot constructor audit retains unrelated generated text exactly", () => {
  for (const source of [
    "$RequiredSlots<$SlotContract_0, {}>",
    "const user_slots_0: $RequiredSlots<$SlotContract_0, {}> = {};",
    "const __vize_required_slots_0: $Other<$SlotContract_0, {}> = {};",
    "const __vize_required_slots_0: $RequiredSlots<$Other_0, {}> = {};",
  ])
    assert.equal(canonicalMappedGenerated(source), source);
});

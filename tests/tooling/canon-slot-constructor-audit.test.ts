import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  canonicalMappedGenerated,
  selectSdk,
} from "../../tools/benchmarks/scripts/canon-slot-constructor-audit.ts";

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

test("slot constructor audit selects the binary and declarations from one SDK image", (context) => {
  const root = fs.mkdtempSync(path.join(process.env.TMPDIR ?? "/tmp", "vize-slot-sdk-"));
  context.after(() => fs.rmSync(root, { recursive: true }));
  const project = path.join(root, "project"),
    installed = path.join(project, "node_modules/vize"),
    head = path.join(root, "head");
  fs.mkdirSync(installed, { recursive: true });
  fs.mkdirSync(head);
  fs.writeFileSync(
    path.join(installed, "package.json"),
    JSON.stringify({ typescript: { contentMapper: { exec: ["baseline", "content-mapper"] } } }),
  );
  for (const extension of ["ts", "mts", "cts"])
    fs.writeFileSync(path.join(head, `content-mapper-helpers.d.${extension}`), `head-${extension}`);

  selectSdk(project, "/head/vize", head);

  assert.deepEqual(
    JSON.parse(fs.readFileSync(path.join(installed, "package.json"), "utf8")).typescript
      .contentMapper.exec,
    ["/head/vize", "content-mapper"],
  );
  for (const extension of ["ts", "mts", "cts"])
    assert.equal(
      fs.readFileSync(path.join(installed, `content-mapper-helpers.d.${extension}`), "utf8"),
      `head-${extension}`,
    );
});

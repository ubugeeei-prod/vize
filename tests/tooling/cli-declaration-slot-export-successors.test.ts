import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/tool-commands.ts";

const originals = [
  [
    "integration__build_declaration_emit_cli__build_dts_emits_declarations_alongside_compiled_js",
    "b3700c0bf6219fdfa83fd4c84daed401d150f0116975cd77b41b8d0e0ca55290",
  ],
  [
    "integration__check_cli__check_can_emit_declarations",
    "5705ceab9598e11053117bbd6c06b2f7d332ba8e1e5b3617ff03983fb0ff0d20",
  ],
] as const;

for (const [name, digest] of originals) {
  test(`${name} preserves its original packet and admits only the slot export successor`, () => {
    const snapshotRoot = path.join(repoRoot, "crates/vize/tests/snapshots");
    const original = fs.readFileSync(path.join(snapshotRoot, `${name}.snap`));
    const successor = fs.readFileSync(path.join(snapshotRoot, `${name}_slot_reexport.snap`));
    assert.equal(createHash("sha256").update(original).digest("hex"), digest);
    const before = "export type Slots = $Slots;";
    const after = "export type { $Slots as Slots };";
    const text = original.toString("utf8");
    assert.equal(text.split(before).length - 1, 1);
    assert.equal(text.split(after).length - 1, 0);
    assert.deepEqual(successor, Buffer.from(text.replace(before, after)));
  });
}

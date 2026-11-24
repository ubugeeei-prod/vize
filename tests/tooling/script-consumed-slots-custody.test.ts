import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { beforeScriptConsumedSlots } from "../../tools/support/compat/content-mapper/authored-component-infer-transition.ts";

const root = new URL("../../", import.meta.url);
const identity = (text: string) => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});
const raw = readFileSync(new URL("tests/_fixtures/tooling/script-consumed-slots-13578.json", root));
assert.equal(
  createHash("sha256").update(raw).digest("hex"),
  "87473c12da88cdefdadb4d3fac8e2ac97c61495e45896502783c2298e7a15314",
);
const record = JSON.parse(raw.toString("utf8"));

void test("script-consumed slot producers restore exact predecessors without historical credit", () => {
  assert.deepEqual(record.credit, { compiled: false, native: false, performance: false });
  assert.deepEqual(
    record.files.map((file: { path: string }) => file.path),
    [
      "crates/vize_canon/src/virtual_ts/generator.rs",
      "crates/vize_canon/src/virtual_ts/generator/setup_imports.rs",
      "crates/vize_canon/src/virtual_ts/generator/component_public_types.rs",
    ],
  );
  for (const file of record.files) {
    const current = readFileSync(new URL(file.path, root), "utf8");
    assert.deepEqual(identity(current), file.after);
    const previous = beforeScriptConsumedSlots(file.path, current);
    assert.deepEqual(identity(previous), file.before);
    assert.equal(beforeScriptConsumedSlots(file.path, previous), previous);
    assert.equal(beforeScriptConsumedSlots("other.rs", current), current);
    for (const mutated of [
      current + current,
      current + "\n",
      ...file.hunks.map((hunk: { after: string }) => current.replace(hunk.after, "")),
    ]) {
      assert.equal(beforeScriptConsumedSlots(file.path, mutated), mutated);
      assert.notDeepEqual(identity(mutated), file.before);
    }
  }
});

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import path from "node:path";

const root = fileURLToPath(new URL("../../", import.meta.url));
const read = (file: string) => readFileSync(path.join(root, file), "utf8");
const hash = (text: string) => createHash("sha256").update(text).digest("hex");
const corpus =
  "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/";
type Identity = { bytes: number; sha256: string };

void test("the ABI hook restores the exact previously authenticated generator", () => {
  const bytes = read(corpus + "helper-abi-generator-update-11809.json");
  const bridge = read(
    "crates/vize_canon/tests/empty_vue_star_exports/membership/constructor_without_args/payload_update/helper_abi_sdk.rs",
  );
  assert.ok(bridge.includes(`const RECORD_SHA: &str = "${hash(bytes)}";`));
  const record = JSON.parse(bytes) as {
    nativeAcceptanceCredit: boolean;
    canonicalAssetsChangedByThisFix: boolean;
    files: {
      path: string;
      before: Identity;
      after: Identity;
      edits: { before: string; after: string }[];
    }[];
  };
  assert.equal(record.nativeAcceptanceCredit, false);
  assert.equal(record.canonicalAssetsChangedByThisFix, false);
  assert.equal(record.files.length, 1);
  const entry = record.files[0]!;
  const identity = (text: string) => ({ bytes: Buffer.byteLength(text), sha256: hash(text) });
  const inverse = (current: string) => {
    assert.deepEqual(identity(current), entry.after);
    for (const edit of entry.edits) {
      assert.ok(edit.after);
      assert.equal(current.split(edit.after).length, 2);
      current = current.replace(edit.after, edit.before);
    }
    assert.deepEqual(identity(current), entry.before);
    return current;
  };
  const oldBytes = read(corpus + "canonical-loose-emit-sdk-update-11815.json");
  assert.equal(hash(oldBytes), "b39a4a47a11d2e343c2cec25a899830c0ac13403b9445d2bc73715f533e22247");
  const old = JSON.parse(oldBytes) as { files: { path: string; after: Identity }[] };
  assert.deepEqual(entry.before, old.files.find((file) => file.path === entry.path)!.after);
  const current = read(entry.path);
  inverse(current);
  for (const changed of [
    current + "\n",
    current + current,
    current.replace(entry.edits[0]!.after, ""),
  ])
    assert.throws(() => inverse(changed));
});

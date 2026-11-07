import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  beforeEmitMemberInfer,
  emitMemberInferEditsBefore,
  emitMemberInferRecordPath,
} from "./support/emit-member-infer-source.ts";

const root = new URL("../../", import.meta.url);
const read = (path: string) => fs.readFileSync(new URL(path, root), "utf8");
interface Entry {
  path: string;
  before: { bytes: number; sha256: string };
  edits: { before: string; after: string }[];
}
void test("member-inferred emit factories restore every exact previous source and reject foreign bytes", () => {
  const record = JSON.parse(read(emitMemberInferRecordPath)) as { files: Entry[] };
  assert.equal(record.files.length, 4);
  for (const entry of record.files) {
    const current = read(entry.path);
    const previous = beforeEmitMemberInfer(read, entry.path, current);
    assert.equal(Buffer.byteLength(previous), entry.before.bytes);
    assert.equal(emitMemberInferEditsBefore(read, entry.path, current), previous);
    assert.equal(emitMemberInferEditsBefore(read, entry.path, previous), previous);
    for (const edit of entry.edits) {
      for (const mutation of [
        current + "\n",
        current + current,
        current.replace(edit.after, ""),
        current.replace(edit.after, edit.before),
      ]) {
        assert.notEqual(mutation, current);
        assert.throws(() => beforeEmitMemberInfer(read, entry.path, mutation));
      }
    }
  }
  const other = "crates/vize_canon/src/virtual_ts/helpers/declarations/vue3-types.d.ts.txt";
  assert.equal(emitMemberInferEditsBefore(read, other, read(other)), read(other));
  assert.throws(() =>
    beforeEmitMemberInfer(
      (path) => read(path) + (path === emitMemberInferRecordPath ? " " : ""),
      record.files[0]!.path,
      read(record.files[0]!.path),
    ),
  );
});

import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  beforeGenericEventSpecialization,
  genericEventRecordPath,
} from "./support/generic-event-specialization-source.ts";

const root = new URL("../../", import.meta.url);
const read = (path: string) => fs.readFileSync(new URL(path, root), "utf8");
interface Entry {
  path: string;
  before: { bytes: number; sha256: string };
  edits: { before: string; after: string }[];
}
void test("generic specialization restores every exact previous source and rejects foreign bytes", () => {
  const record = JSON.parse(read(genericEventRecordPath)) as { files: Entry[] };
  assert.equal(record.files.length, 5);
  for (const entry of record.files) {
    const current = read(entry.path);
    const previous = beforeGenericEventSpecialization(read, entry.path, current);
    assert.equal(Buffer.byteLength(previous), entry.before.bytes);
    for (const edit of entry.edits) {
      for (const mutation of [
        current + "\n",
        current + current,
        current.replace(edit.after, ""),
        current.replace(edit.after, edit.before),
      ]) {
        assert.notEqual(mutation, current);
        assert.throws(() => beforeGenericEventSpecialization(read, entry.path, mutation));
      }
    }
  }
  assert.throws(() =>
    beforeGenericEventSpecialization(
      (path) => read(path) + (path === genericEventRecordPath ? " " : ""),
      record.files[0]!.path,
      read(record.files[0]!.path),
    ),
  );
});

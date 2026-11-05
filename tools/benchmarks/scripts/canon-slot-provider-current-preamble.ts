/** Source-certified predictions; fresh native output must still match every byte. */
import assert from "node:assert/strict";
import fs from "node:fs";
import {
  beforeGenericEventPreamble,
  beforeGenericEventSpecialization,
  genericEventRecordPath,
} from "../../../tests/tooling/support/generic-event-specialization-source.ts";
import { applyCurrentEmitPreamble } from "./canon-slot-provider-preamble.ts";

type Read = (path: string) => string;
const source = (path: string) => fs.readFileSync(path, "utf8");

/** Preserve the b39 producer law through its exact, authenticated successor inverse. */
export function readPreviousEmitSource(path: string, read: Read = source): string {
  return beforeGenericEventSpecialization(read, path, read(path));
}

export function applyCurrentSlotPreamble(previous: string, read: Read = source): string {
  const predecessor = applyCurrentEmitPreamble(previous, (path) =>
    readPreviousEmitSource(path, read),
  );
  const certificate = JSON.parse(read(genericEventRecordPath)) as {
    nativeAcceptanceCredit: boolean;
    preamble: { edits: { before: string; after: string }[] };
  };
  assert.equal(certificate.nativeAcceptanceCredit, false);
  let current = predecessor;
  for (const edit of certificate.preamble.edits) {
    assert.ok(edit.before.length > 0);
    assert.equal(current.split(edit.before).length, 2, "one certified successor preamble edit");
    current = current.replace(edit.before, edit.after);
  }
  assert.equal(
    beforeGenericEventPreamble(read, current),
    predecessor,
    "complete predicted preamble must invert to its certified predecessor",
  );
  return current;
}

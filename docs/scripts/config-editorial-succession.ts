import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const fixture = JSON.parse(
  readFileSync(
    resolve(import.meta.dirname, "../../tests/_fixtures/docs-config-editorial-succession.json"),
    "utf8",
  ),
) as {
  witnesses: { case: string; previous: string; after: string }[];
};

/** Map a recorded mistranslation witness to its reviewed configuration successor. */
export function currentEditorialWitness<T extends { case: string; after: string }>(witness: T): T {
  const successor = fixture.witnesses.find((entry) => entry.case === witness.case);
  if (!successor) return witness;
  assert.equal(witness.after, successor.previous, `${witness.case}: original whole witness`);
  return { ...witness, after: successor.after };
}

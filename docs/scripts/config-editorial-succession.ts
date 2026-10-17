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

const current = JSON.parse(
  readFileSync(
    resolve(import.meta.dirname, "../../tests/_fixtures/docs-vue-config-editorial-succession.json"),
    "utf8",
  ),
) as typeof fixture;

/** Map a recorded mistranslation witness to its reviewed configuration successor. */
export function currentEditorialWitness<T extends { case: string; after: string }>(witness: T): T {
  let result = witness;
  for (const generation of [fixture, current]) {
    const successor = generation.witnesses.find((entry) => entry.case === result.case);
    if (!successor) continue;
    assert.equal(result.after, successor.previous, `${result.case}: original whole witness`);
    result = { ...result, after: successor.after };
  }
  return result;
}

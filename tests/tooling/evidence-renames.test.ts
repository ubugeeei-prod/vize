// Every registered rename spells back to its pinned evidence bytes.
import assert from "node:assert/strict";
import { test } from "node:test";

import {
  evidenceRenames,
  parseEvidenceRenames,
  spellBack,
  spellBackText,
  validateEvidenceRenames,
} from "./support/evidence-renames.ts";

test("every registered rename spells back to its pinned bytes", () => {
  assert(evidenceRenames().length > 0);
  validateEvidenceRenames();
});

const synthetic = parseEvidenceRenames(
  JSON.stringify({
    schema: "vize.evidence-renames",
    version: 1,
    renames: [
      { id: "one", names: [["A", "B"]], evidence: ["e.json"] },
      { id: "two", names: [["B", "C"]], evidence: ["e.json", "f.json"] },
    ],
  }),
);

test("spelling back undoes newer renames first, only for listed files, word for word", () => {
  assert.equal(spellBackText("C { B, CX, _C }", "e.json", synthetic), "A { A, CX, _C }");
  assert.equal(spellBackText("C", "f.json", synthetic), "B");
  assert.equal(spellBackText("C", "g.json", synthetic), "C");
  assert.deepEqual(spellBack({ C: ["C", 1, null] }, "f.json", synthetic), { B: ["B", 1, null] });
});

test("evidence that already spells a new name, or never the old one, is refused", () => {
  const files: Record<string, string> = { "e.json": '{"debug":"Old { x: New }"}' };
  const rename = (old: string, renamed: string) =>
    parseEvidenceRenames(
      JSON.stringify({
        schema: "vize.evidence-renames",
        version: 1,
        renames: [{ id: "r", names: [[old, renamed]], evidence: ["e.json"] }],
      }),
    );
  validateEvidenceRenames(rename("Old", "Fresh"), (file) => files[file]!);
  assert.throws(
    () => validateEvidenceRenames(rename("Old", "New"), (file) => files[file]!),
    /already spells New/,
  );
  assert.throws(
    () => validateEvidenceRenames(rename("Gone", "Fresh"), (file) => files[file]!),
    /never spells Gone/,
  );
});

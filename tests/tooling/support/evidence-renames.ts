// Pinned evidence read through the current names (the TS side of
// `tests/davinci_test_support/src/evidence_renames.rs`).
//
// `docs/davinci/plan/evidence-renames.json` registers each type rename, oldest
// first, with the pinned evidence files that spell the old names. A custody
// gate compares `spellBack(current, file)` with the pinned bytes, which stay
// untouched; spelling back undoes, newest first, every registered rename that
// lists the file, word for word.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
export const evidenceRenamesPath = "docs/davinci/plan/evidence-renames.json";

export interface EvidenceRename {
  readonly id: string;
  readonly names: readonly (readonly [old: string, renamed: string])[];
  readonly evidence: readonly string[];
}

export function parseEvidenceRenames(text: string): EvidenceRename[] {
  const map = JSON.parse(text);
  assert.deepEqual(Object.keys(map).sort(), ["renames", "schema", "version"]);
  assert.equal(map.schema, "vize.evidence-renames");
  assert.equal(map.version, 1);
  assert(Array.isArray(map.renames), "`renames` array");
  return map.renames.map((entry: any) => {
    assert.deepEqual(Object.keys(entry).sort(), ["evidence", "id", "names"]);
    assert.equal(typeof entry.id, "string");
    for (const pair of entry.names)
      assert(
        Array.isArray(pair) && pair.length === 2 && pair.every((name) => typeof name === "string"),
        `${entry.id}: a name pair is [old, new]`,
      );
    assert(entry.evidence.every((file: unknown) => typeof file === "string"));
    return { id: entry.id, names: entry.names, evidence: entry.evidence };
  });
}

let cached: EvidenceRename[] | undefined;
export function evidenceRenames(): EvidenceRename[] {
  cached ??= parseEvidenceRenames(readFileSync(path.join(repoRoot, evidenceRenamesPath), "utf8"));
  return cached;
}

const word = (name: string) => new RegExp(`(?<![A-Za-z0-9_])${name}(?![A-Za-z0-9_])`, "gu");
export const mentions = (text: string, name: string) => word(name).test(text);

function listing(renames: readonly EvidenceRename[], evidence: string) {
  return renames.filter((rename) => rename.evidence.includes(evidence));
}

export function spellBackText(
  text: string,
  evidence: string,
  renames: readonly EvidenceRename[] = evidenceRenames(),
): string {
  return listing(renames, evidence)
    .toReversed()
    .reduce(
      (out, rename) =>
        rename.names.reduce((spelled, [old, renamed]) => spelled.replace(word(renamed), old), out),
      text,
    );
}

export function spellForwardText(
  text: string,
  evidence: string,
  renames: readonly EvidenceRename[] = evidenceRenames(),
): string {
  return listing(renames, evidence).reduce(
    (out, rename) =>
      rename.names.reduce((spelled, [old, renamed]) => spelled.replace(word(old), renamed), out),
    text,
  );
}

/** A current JSON value spelled with the names the pinned file `evidence` recorded. */
export function spellBack(
  value: any,
  evidence: string,
  renames: readonly EvidenceRename[] = evidenceRenames(),
): any {
  if (typeof value === "string") return spellBackText(value, evidence, renames);
  if (Array.isArray(value)) return value.map((item) => spellBack(item, evidence, renames));
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [
        spellBackText(key, evidence, renames),
        spellBack(item, evidence, renames),
      ]),
    );
  return value;
}

/**
 * The checks `evidence_renames::validate` makes: unique ids; identifier names
 * renamed to new spellings; every listed file spells each old name of its
 * entry and no new one (in the spelling that entry renames from), and reads
 * back unchanged after spelling forward.
 */
export function validateEvidenceRenames(
  renames: readonly EvidenceRename[] = evidenceRenames(),
  read: (file: string) => string = (file) => readFileSync(path.join(repoRoot, file), "utf8"),
) {
  const identifier = /^[A-Za-z_][A-Za-z0-9_]*$/u;
  renames.forEach((rename, index) => {
    assert(!renames.slice(0, index).some((earlier) => earlier.id === rename.id), rename.id);
    assert(rename.names.length && rename.evidence.length, `${rename.id} needs names and evidence`);
    for (const [old, renamed] of rename.names) {
      assert(identifier.test(old) && identifier.test(renamed) && old !== renamed, rename.id);
      assert(!rename.names.some(([other]) => other === renamed), `${rename.id} renames ${renamed}`);
    }
    for (const file of rename.evidence) {
      const atEntry = spellBackText(read(file), file, renames.slice(index + 1));
      for (const [old, renamed] of rename.names) {
        assert(mentions(atEntry, old), `${file} never spells ${old} (${rename.id})`);
        assert(!mentions(atEntry, renamed), `${file} already spells ${renamed} (${rename.id})`);
      }
    }
  });
  for (const file of new Set(renames.flatMap((rename) => rename.evidence))) {
    const text = read(file);
    assert.equal(
      spellBackText(spellForwardText(text, file, renames), file, renames),
      text,
      `${file} reads back after spelling forward`,
    );
  }
}

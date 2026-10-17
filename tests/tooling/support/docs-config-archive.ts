import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

const archive = JSON.parse(
  readFileSync(new URL("../fixtures/docs-config-archived-inputs.json", import.meta.url), "utf8"),
) as {
  documents: { path: string; base: string; sha256: string; source: string }[];
};

/** Whole historical documents remain available even in shallow Actions checkouts. */
export function originalConfigDocument(path: string): string {
  const document = archive.documents.find((entry) => entry.path === path);
  assert.ok(document, `${path}: named pre-migration document`);
  assert.match(document.base, /^[0-9a-f]{40}$/);
  assert.equal(
    createHash("sha256").update(document.source).digest("hex"),
    document.sha256,
    `${path}: complete original bytes`,
  );
  return document.source;
}

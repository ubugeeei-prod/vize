import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import type { FixtureProject } from "./real-project-lsp-report.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const corpus = path.join(root, "tests/_fixtures/differential/lsp/airi-authored-events");
type RankedItem = { label: string; rank: number };
type Authority = {
  projectId: string;
  repository: string;
  revision: string;
  componentFile: string;
  sourceUrl: string;
  sourceBlob: string;
  sourceSha256: string;
  originalEventCorpusSha256: string;
  predecessorSourceCommit: string;
  predecessorCompletionCount: number;
  predecessorCompletionItems: RankedItem[];
  declaration: string;
  eventLabels: string[];
};

/** A finite seventh declaration owner; the original six-case corpus is frozen. */
export function appendAiriAuthoredEventSuccessor(
  project: FixtureProject & { repository: string },
  predecessor: RankedItem[],
): RankedItem[] {
  if (project.id !== "airi") return predecessor;
  const authority = JSON.parse(
    fs.readFileSync(path.join(corpus, "case.json"), "utf8"),
  ) as Authority;
  const source = fs.readFileSync(path.join(corpus, "CharacterItem.vue.txt"));
  const originalEvents = fs.readFileSync(
    path.join(root, "tests/_fixtures/lsp-authored-component-event-contracts.json"),
  );
  assert.equal(authority.projectId, "airi");
  assert.equal(authority.repository, "https://github.com/moeru-ai/airi");
  assert.equal(authority.repository, project.repository);
  assert.equal(authority.revision, "b6011381bc34a6b85ad669363513cb1a2eea6438");
  assert.equal(authority.revision, project.revision);
  assert.equal(
    authority.componentFile,
    "apps/stage-web/src/pages/settings/characters/components/CharacterItem.vue",
  );
  assert.equal(authority.componentFile, project.lspAuthoredOracle?.componentBoundary.componentFile);
  assert.equal(
    authority.sourceUrl,
    `${authority.repository}/blob/${authority.revision}/${authority.componentFile}`,
  );
  assert.equal(authority.sourceBlob, "15f1567cf138b2f85243739263e8071a6745a9dc");
  assert.equal(
    createHash("sha1").update(`blob ${source.byteLength}\0`).update(source).digest("hex"),
    authority.sourceBlob,
  );
  assert.equal(
    authority.sourceSha256,
    "e1ad0bb3f1c20e510071a420bc3eb49ccc13ec9646f8796070f9b11fac7e85cc",
  );
  assert.equal(createHash("sha256").update(source).digest("hex"), authority.sourceSha256);
  assert.equal(
    authority.originalEventCorpusSha256,
    "3e99c6a976972fdd15dabe1c08b1b2748b08cdb744123813f7d13ab992c06042",
  );
  assert.equal(
    createHash("sha256").update(originalEvents).digest("hex"),
    authority.originalEventCorpusSha256,
  );
  assert.equal(authority.predecessorSourceCommit, "1d988f6bcac8afc6a0fcad9b76b0d5f2da27be71");
  assert.equal(authority.predecessorCompletionCount, 40);
  assert.equal(predecessor.length, 40);
  assert.deepEqual(authority.predecessorCompletionItems, predecessor);
  assert.deepEqual(
    predecessor.map((item) => item.rank),
    [...predecessor.keys()],
  );
  assert.equal(
    authority.declaration,
    "const emit = defineEmits<{\n  (e: 'select'): void\n  (e: 'activate'): void\n  (e: 'delete'): void\n}>()",
  );
  assert.equal(source.toString().split(authority.declaration).length, 2);
  const declared = [...authority.declaration.matchAll(/\(e: '([a-z]+)'\): void/g)]
    .map((match) => `@${match[1]}`)
    .sort();
  assert.deepEqual(declared, ["@activate", "@delete", "@select"]);
  assert.deepEqual(authority.eventLabels, declared);
  const successor = [
    ...predecessor,
    ...declared.map((label, index) => ({ label, rank: 40 + index })),
  ];
  assert.equal(successor.length, 43);
  assert.equal(new Set(successor.map((item) => item.label)).size, 43);
  return successor;
}

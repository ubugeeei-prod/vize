import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export type CompilerSuccessorReference = {
  path: string;
  sha256: string;
  supersedesSha256: string;
  issue: string;
};

type CompilerSuccessorOwner = {
  id: string;
  reference: { path: string; sha256: string };
  referenceHistory?: CompilerSuccessorReference[];
  currentReference?: CompilerSuccessorReference;
};

const ISSUE_URL = /^https:\/\/github\.com\/ubugeeei-prod\/vize\/issues\/[1-9]\d*$/u;
const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");

export function assertCompilerSuccessors(root: string, fixture: CompilerSuccessorOwner) {
  const history = fixture.referenceHistory ?? [];
  assert(Array.isArray(history), `${fixture.id}: referenceHistory must be an array`);
  assert(
    history.length === 0 || fixture.currentReference,
    `${fixture.id}: referenceHistory requires currentReference`,
  );
  const successors = fixture.currentReference ? [...history, fixture.currentReference] : [];
  const paths = new Set([fixture.reference.path]);
  const hashes = new Set([fixture.reference.sha256]);
  let predecessorSha256 = fixture.reference.sha256;
  for (const [index, successor] of successors.entries()) {
    assert.deepEqual(
      Object.keys(successor).sort(),
      ["issue", "path", "sha256", "supersedesSha256"],
      `${fixture.id}: successor ${index} schema is closed`,
    );
    assert(!paths.has(successor.path), `${fixture.id}: duplicate successor path ${successor.path}`);
    assert(
      !hashes.has(successor.sha256),
      `${fixture.id}: duplicate successor SHA256 ${successor.sha256}`,
    );
    paths.add(successor.path);
    hashes.add(successor.sha256);
    assert.equal(
      successor.supersedesSha256,
      predecessorSha256,
      `${fixture.id}: successor ${index} does not extend its authenticated predecessor`,
    );
    assert.match(
      successor.issue,
      ISSUE_URL,
      `${fixture.id}: successor ${index} needs an authenticated issue`,
    );
    assert.match(successor.path, /^[a-zA-Z0-9_./-]+$/u);
    assert.match(successor.sha256, /^[a-f0-9]{64}$/u);
    const bytes = fs.readFileSync(path.join(root, successor.path));
    assert.equal(
      digest(bytes),
      successor.sha256,
      `immutable artifact SHA256 mismatch: ${successor.path}`,
    );
    predecessorSha256 = successor.sha256;
  }
  return successors;
}

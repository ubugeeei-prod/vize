import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { gunzipSync } from "node:zlib";
import { generatedBody } from "./generated-header.ts";
import transitions from "./workflow-program-history.json" with { type: "json" };

type WorkflowName = keyof typeof transitions;
const digest = (bytes: Buffer) => createHash("sha256").update(generatedBody(bytes)).digest("hex");

/** Follow one reviewed format transition before applying an older whole-workflow byte contract. */
export function workflowPredecessor(
  owner: string,
  current: Buffer,
  read: (path: string) => Buffer | null,
): Buffer {
  assert.ok(Object.hasOwn(transitions, owner), "workflow has no registered byte contract");
  const transition = transitions[owner as WorkflowName];
  const currentDigest = digest(current);
  if (currentDigest === transition.before) return generatedBody(current);
  assert.equal(currentDigest, transition.workflow, owner + ": unreviewed workflow revision");

  const closure = createHash("sha256");
  for (const file of transition.programs) {
    const bytes = read(file);
    assert.ok(bytes, "missing executable source: " + file);
    closure.update(file + "\0" + digest(bytes) + "\n");
  }
  assert.equal(closure.digest("hex"), transition.closure, owner + ": executable source changed");
  const archive = read("tests/_fixtures/tooling/named-workflow-programs/" + owner + ".yml.gz");
  assert.ok(archive, "missing historical workflow: " + owner);
  const original = gunzipSync(archive);
  assert.equal(digest(original), transition.before, owner + ": historical workflow changed");
  return original;
}

/** Let older validators inspect unknown revisions and independently enforce their original identities. */
export function priorRegisteredWorkflow(
  owner: string,
  current: Buffer,
  read: (path: string) => Buffer | null,
): Buffer {
  if (
    !Object.hasOwn(transitions, owner) ||
    digest(current) !== transitions[owner as WorkflowName].workflow
  )
    return current;
  return workflowPredecessor(owner, current, read);
}

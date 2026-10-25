import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const baseSource = "49a98d1e72707083354b8ff11e1cec800ded28b3";
const sha = /^[0-9a-f]{40}$/;

/** The recipe actor and PR trigger are distinct; products keep their real source. */
export function sourceProvenance(observerActor, env = process.env) {
  assert.match(observerActor, sha, "The actual observer git HEAD must be a full SHA");
  assert.match(env.GITHUB_SHA || "", sha, "The workflow actor must be a full SHA");
  assert.equal(observerActor, env.GITHUB_SHA, "Observer HEAD must equal the workflow actor");
  assert(["pull_request", "workflow_dispatch"].includes(env.GITHUB_EVENT_NAME));
  let triggerHead = null;
  if (env.GITHUB_EVENT_NAME === "pull_request") {
    triggerHead = env.VIZE_SFC_ATTRS_TRIGGER_HEAD;
    assert.match(triggerHead || "", sha, "The PR trigger head must be recorded separately");
  } else {
    assert.equal(env.VIZE_SFC_ATTRS_TRIGGER_HEAD || "", "", "Manual runs have no PR head");
  }
  return { event: env.GITHUB_EVENT_NAME, observerActor, githubSha: env.GITHUB_SHA, triggerHead };
}

export function validateSourceProvenance(value, observerActor, env = process.env) {
  const expected = sourceProvenance(observerActor, env);
  assert.deepEqual(value, expected, "All source roles must bind the same actual observer");
  return expected;
}

export function validateProductSource(source, role, observerActor) {
  assert(["base", "head"].includes(role));
  assert.match(observerActor, sha);
  assert.equal(source, role === "base" ? baseSource : observerActor);
}

export function validateProductPacket(packet, image, role, observerActor, env = process.env) {
  assert.equal(packet.role, role, "Packet role must match its expected artifact container");
  assert.equal(
    packet.receipt.role,
    role,
    "Receipt role must match its expected artifact container",
  );
  assert.equal(image.role, role, "Image role must match its expected artifact container");
  validateProductSource(packet.source, role, observerActor);
  assert.equal(packet.receipt.source, packet.source);
  validateSourceProvenance(packet.receipt.provenance, observerActor, env);
  assert.deepEqual(image, packet.receipt, "The compared packets must retain their actual image");
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const observerActor = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  process.stdout.write(JSON.stringify(sourceProvenance(observerActor)) + "\n");
}

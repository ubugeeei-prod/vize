/** Original casing consumer custody and its reviewed finite successors. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import {
  asArray,
  asRecord,
  asString,
  parseJson,
  type JsonRecord,
} from "../../../../tools/support/json-value.ts";

export const componentNameSuccessorPath =
  "npm/linter/oxlint/src/component-name-source-successor.json";
const successorSha256 = "384e1699ba8f83233fc410ad6c52255d8394cf01ab5d658fe51c29a6a4c2bc79";
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

export function componentNameSource(
  root: string,
  read: (path: string) => Buffer = (path) => readFileSync(join(root, path)),
): { historical: Buffer; current: Buffer; successor: JsonRecord } {
  const metadata = read(componentNameSuccessorPath);
  assert.equal(hash(metadata), successorSha256);
  const successor = asRecord(parseJson(metadata.toString("utf8")));
  assert.equal(successor.schema, "vize.component-name-eslint.source-successor.v1");
  assert.equal(successor.path, "crates/vize/tests/lint_rule_options_casing_cli.rs");
  const descriptor = asRecord(successor.originalDescriptor);
  const descriptorBytes = read(asString(descriptor.path));
  assert.equal(descriptorBytes.length, descriptor.bytes);
  assert.equal(hash(descriptorBytes), descriptor.sha256);
  const historicalIdentity = asRecord(successor.historical);
  const historical = read(asString(historicalIdentity.snapshot));
  const latest = read(asString(successor.path));
  assert.equal(
    asRecord(asRecord(parseJson(descriptorBytes.toString("utf8"))).source).originalConsumerSha256,
    hash(historical),
  );
  for (const [kind, bytes] of [["historical", historical]] as const) {
    const identity = asRecord(successor[kind]);
    assert.equal(bytes.length, identity.bytes, kind);
    assert.equal(hash(bytes), identity.sha256, kind);
    assert.equal(
      createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex"),
      identity.gitBlob,
      kind,
    );
  }
  let projected = historical.toString("utf8");
  for (const raw of asArray(successor.changes)) {
    const change = asRecord(raw);
    assert.equal(projected.split(asString(change.before)).length - 1, change.occurrences);
    projected = projected.replace(asString(change.before), asString(change.after));
  }
  const current = Buffer.from(projected);
  for (const [kind, bytes] of [
    ["current", current],
    ["latest", latest],
  ] as const) {
    const identity = asRecord(successor[kind]);
    assert.equal(bytes.length, identity.bytes, kind);
    assert.equal(hash(bytes), identity.sha256, kind);
    assert.equal(
      createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex"),
      identity.gitBlob,
      kind,
    );
  }
  for (const raw of asArray(successor.latestChanges)) {
    const change = asRecord(raw);
    assert.equal(projected.split(asString(change.before)).length - 1, change.occurrences);
    projected = projected.replace(asString(change.before), asString(change.after));
  }
  assert.equal(
    latest.toString("utf8"),
    projected,
    "only the reviewed finite successor literals may change",
  );
  return { historical, current: latest, successor };
}

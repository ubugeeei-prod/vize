/** Invert the projection text-buffer refactor (#12341) before older emitter custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { beforeClosedEmitFactory } from "./closed-emit-factory-transition.ts";
import { beforeConstructorNoInferReturn } from "./constructor-noinfer-return-transition.ts";
import { beforeListenerContract } from "./listener-contract-transition.ts";
import { beforeStaticEmitGate } from "./static-emit-gate-transition.ts";

export const projectionTextBufferRecord =
  "tests/_fixtures/tooling/projection-text-buffer-12341.json";
const recordSha256 = "0a6f7d277e335d9815d109126e2f116598c5b10f3971ab97d66f786ee198177a";

type Identity = { bytes: number; sha256: string };
interface Transition {
  schema: string;
  pr: number;
  credit: { compiled: boolean; native: boolean; performance: boolean };
  files: Array<{
    path: string;
    before: Identity;
    after: Identity;
    hunks: Array<{ before: string; after: string }>;
  }>;
}

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

let record: Transition | undefined;
function load(): Transition {
  if (record) return record;
  const raw = readFileSync(
    new URL(`../../../../${projectionTextBufferRecord}`, import.meta.url),
    "utf8",
  );
  assert.equal(identity(raw).sha256, recordSha256, "immutable projection text-buffer transition");
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.projectionTextBufferSource.v1");
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/**
 * The exact pre-refactor bytes of a recorded producer's current image. Older
 * images pass through; the layers below authenticate them.
 */
export function beforeProjectionTextBuffer(path: string, current: string): string {
  current = beforeStaticEmitGate(
    path,
    beforeListenerContract(
      path,
      beforeConstructorNoInferReturn(path, beforeClosedEmitFactory(path, current)),
    ),
  );
  const file = load().files.find((entry) => entry.path === path);
  if (!file || identity(current).sha256 !== file.after.sha256) return current;
  assert.equal(
    Buffer.byteLength(current),
    file.after.bytes,
    `projection text-buffer producer: ${path}`,
  );
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `projection text-buffer hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `projection text-buffer predecessor: ${path}`);
  return current;
}

/** `read` with every recorded producer restored to its pre-refactor bytes. */
export function readBeforeProjectionTextBuffer(
  read: (path: string) => string,
): (path: string) => string {
  return (path) => beforeProjectionTextBuffer(path, read(path));
}

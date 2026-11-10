/** Invert the shared listener contract before older emitter and SDK custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { beforeListenerLibTypeQualification } from "./lib-type-qualification-listeners-transition.ts";

export const listenerContractRecord = "tests/_fixtures/tooling/listener-contract-source.json";
const recordSha256 = "cafa40d3371b70f521b218afd628431e671625266aca9d310d02013c4b765be5";

type Identity = { bytes: number; sha256: string };
interface Transition {
  schema: string;
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
    new URL(`../../../../${listenerContractRecord}`, import.meta.url),
    "utf8",
  );
  assert.equal(identity(raw).sha256, recordSha256, "immutable listener contract transition");
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.listenerContractSource.v1");
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/** The exact pre-contract bytes of a recorded source's current image; older images pass through. */
export function beforeListenerContract(path: string, current: string): string {
  current = beforeListenerLibTypeQualification(path, current);
  const file = load().files.find((entry) => entry.path === path);
  if (!file || identity(current).sha256 !== file.after.sha256) return current;
  assert.equal(Buffer.byteLength(current), file.after.bytes, `listener contract source: ${path}`);
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `listener contract hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `listener contract predecessor: ${path}`);
  return current;
}

/** `read` with every recorded source restored to its pre-contract bytes. */
export function readBeforeListenerContract(
  read: (path: string) => string,
): (path: string) => string {
  return (path) => beforeListenerContract(path, read(path));
}

const asset = "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-listener-args.d.ts.txt";

/** A compiled preamble with the asset's contract lines added, as it embeds them. */
export function applyListenerContractIn(previous: string): string {
  const file = load().files.find((entry) => entry.path === asset);
  assert.ok(file, "listener contract asset");
  for (const hunk of file.hunks) {
    assert.equal(previous.split(hunk.before).length, 2, "listener contract predecessor lines");
    previous = previous.replace(hunk.before, () => hunk.after);
  }
  return previous;
}

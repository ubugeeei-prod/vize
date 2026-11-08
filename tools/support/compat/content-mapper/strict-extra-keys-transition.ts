/** Authenticate #11497's removal of the one-prop boolean-attribute exception. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const strictExtraKeysPath = "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt";
export const strictExtraKeysBeforeSha256 =
  "3a3c9fb284feba5cdd7fc639ae1b99d19c31e5cb344339968acd553734adff53";
export const strictExtraKeysAfterSha256 =
  "3eca9ffe508f656bf244960c7c9a2da73993dc073e6a5262778fbc4e99c10c6d";

const recordUrl = new URL(
  "../../../../crates/vize_canon/tests/empty_vue_star_exports/membership/constructor_without_args/payload_update/strict_extra_keys.rs",
  import.meta.url,
);
const recordSha256 = "8a30e43dbde811c229a4bc92cc53863fdaa5f8918eb8daa4caeabb6578869057";
const canonLeftoversUrl = new URL(
  "../../../../crates/vize_canon/tests/empty_vue_star_exports/membership/constructor_without_args/payload_update/canon_leftovers_11499.json",
  import.meta.url,
);
export const canonLeftoversSha256 =
  "8f87abcf7dfdd6af9dfef500ff3a01616d2756d1439df71f413151b21fb78e36";
const before =
  "type $ExtraInputKeys<T, Declared, Authored, Allowed, K extends keyof Authored = Exclude<keyof Authored, keyof Allowed>> = K extends unknown ? $RelaxedDynamicUnionExtra<T, Declared, Authored, K> extends true ? never : K : never;";
const after =
  "type $ExtraInputKeys<_T, _Declared, Authored, Allowed> = Exclude<keyof Authored, keyof Allowed>;";
const sha256 = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
const identity = (value: string | Buffer) => ({
  bytes: Buffer.byteLength(value),
  sha256: sha256(value),
});

function canonLeftoversTransition(path: string) {
  const receiptBytes = readFileSync(canonLeftoversUrl);
  assert.equal(sha256(receiptBytes), canonLeftoversSha256, "canon leftovers source record");
  const receipt = JSON.parse(receiptBytes.toString("utf8")) as {
    version: number;
    kind: string;
    issue: number;
    files: Array<{
      path: string;
      before: { bytes: number; sha256: string };
      after: { bytes: number; sha256: string };
      edits: Array<{ before: string; after: string }>;
    }>;
  };
  assert.equal(receipt.version, 1);
  assert.equal(receipt.kind, "exact-canon-leftovers-source-inverse");
  assert.equal(receipt.issue, 11499);
  return receipt.files.find((transition) => transition.path === path);
}

export function beforeCanonLeftovers(path: string, current: string): string {
  const transition = canonLeftoversTransition(path);
  if (!transition) return current;
  const currentIdentity = identity(current);
  if (
    currentIdentity.bytes === transition.before.bytes &&
    currentIdentity.sha256 === transition.before.sha256
  )
    return current;
  assert.deepEqual(currentIdentity, transition.after, "canon leftovers source identity");
  let previous = current;
  for (const edit of transition.edits) {
    assert.equal(previous.split(edit.after).length - 1, 1);
    previous = previous.replace(edit.after, () => edit.before);
  }
  assert.deepEqual(identity(previous), transition.before);
  return previous;
}

export function applyCanonLeftovers(path: string, previous: string): string {
  const transition = canonLeftoversTransition(path);
  if (!transition) return previous;
  const previousIdentity = identity(previous);
  if (
    previousIdentity.bytes === transition.after.bytes &&
    previousIdentity.sha256 === transition.after.sha256
  )
    return previous;
  assert.deepEqual(previousIdentity, transition.before, "canon leftovers predecessor identity");
  let current = previous;
  for (const edit of [...transition.edits].reverse()) {
    assert.equal(current.split(edit.before).length - 1, 1);
    current = current.replace(edit.before, () => edit.after);
  }
  assert.deepEqual(identity(current), transition.after);
  return current;
}

export function beforeCanonLeftoversIn(path: string, current: string): string {
  const transition = canonLeftoversTransition(path);
  if (!transition) return current;
  let previous = current;
  for (const edit of transition.edits) {
    assert.equal(previous.split(edit.after).length - 1, 1);
    previous = previous.replace(edit.after, () => edit.before);
  }
  return previous;
}

export function applyCanonLeftoversIn(path: string, previous: string): string {
  const transition = canonLeftoversTransition(path);
  if (!transition) return previous;
  let current = previous;
  for (const edit of [...transition.edits].reverse()) {
    assert.equal(current.split(edit.before).length - 1, 1);
    current = current.replace(edit.before, () => edit.after);
  }
  return current;
}

function authenticateRecord() {
  assert.equal(sha256(readFileSync(recordUrl)), recordSha256, "strict extra-key source record");
}

export function beforeStrictExtraKeysIn(current: string): string {
  assert.equal(current.split(after).length, 2, "current strict extra-key helper is unique");
  assert.equal(current.split(before).length, 1, "previous relaxed helper is absent");
  return current.replace(after, before);
}

export function applyStrictExtraKeysIn(previous: string): string {
  assert.equal(previous.split(before).length, 2, "previous relaxed helper is unique");
  assert.equal(previous.split(after).length, 1, "current strict extra-key helper is absent");
  return previous.replace(before, after);
}

export function strictOutputTransition(current: Buffer, direction: "forward" | "inverse"): Buffer {
  const discriminated = {
    bytes: 15109,
    sha256: "4403b85ed61e9aec229ddea2b4a891950ddd2746bca69be0db4ff670679f0608",
  };
  const strict = {
    bytes: 14978,
    sha256: "703a9301c927db24522d86bb1acf6f80a103281661768791310ffd5361a12b25",
  };
  const [fromIdentity, toIdentity] =
    direction === "forward" ? [discriminated, strict] : [strict, discriminated];
  assert.deepEqual(identity(current), fromIdentity);
  const text = current.toString("utf8");
  const changed = Buffer.from(
    direction === "forward" ? applyStrictExtraKeysIn(text) : beforeStrictExtraKeysIn(text),
  );
  assert.deepEqual(identity(changed), toIdentity);
  return changed;
}

export function beforeStrictExtraKeys(current: string): string {
  authenticateRecord();
  const canonPredecessor = beforeCanonLeftovers(strictExtraKeysPath, current);
  assert.deepEqual(identity(canonPredecessor), {
    bytes: 5739,
    sha256: strictExtraKeysAfterSha256,
  });
  const previous = beforeStrictExtraKeysIn(canonPredecessor);
  assert.deepEqual(identity(previous), { bytes: 5870, sha256: strictExtraKeysBeforeSha256 });
  return previous;
}

export function certifyStrictExtraKeysTransition(file: string, previous: Buffer, current: Buffer) {
  if (
    file !== strictExtraKeysPath ||
    previous.length !== 5870 ||
    current.length !== 5739 ||
    sha256(previous) !== strictExtraKeysBeforeSha256 ||
    sha256(current) !== strictExtraKeysAfterSha256
  )
    return;
  authenticateRecord();
  assert.deepEqual(Buffer.from(beforeStrictExtraKeys(current.toString("utf8"))), previous);
  return {
    file,
    beforeSha256: strictExtraKeysBeforeSha256,
    afterSha256: strictExtraKeysAfterSha256,
    recordSha256,
    nativeAcceptanceCredit: false as const,
  };
}

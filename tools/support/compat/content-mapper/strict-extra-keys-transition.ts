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
const recordSha256 = "bda3fceba73cb304d2cb1b46ad086b5aa8f92e1fa40665fa50bf6db45ab23157";
const before =
  "type $ExtraInputKeys<T, Declared, Authored, Allowed, K extends keyof Authored = Exclude<keyof Authored, keyof Allowed>> = K extends unknown ? $RelaxedDynamicUnionExtra<T, Declared, Authored, K> extends true ? never : K : never;";
const after =
  "type $ExtraInputKeys<_T, _Declared, Authored, Allowed> = Exclude<keyof Authored, keyof Allowed>;";
const sha256 = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
const identity = (value: string | Buffer) => ({
  bytes: Buffer.byteLength(value),
  sha256: sha256(value),
});

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

export function beforeStrictExtraKeys(current: string): string {
  authenticateRecord();
  assert.deepEqual(identity(current), { bytes: 5739, sha256: strictExtraKeysAfterSha256 });
  const previous = beforeStrictExtraKeysIn(current);
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

/** Authenticate the exact integrated JSX-union helper successor. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const discriminatedJsxUnionPath =
  "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt";
export const discriminatedJsxUnionBeforeSha256 =
  "989c87b3f04ffab6a0b8be1c4249bb38d98dd383a6199a90522c18cf7cb13f0d";
export const discriminatedJsxUnionAfterSha256 =
  "3a3c9fb284feba5cdd7fc639ae1b99d19c31e5cb344339968acd553734adff53";

const beforeInstance =
  "$ReplaceComponentProps<I, T extends unknown ? Pick<$AuthoredKeyWitness<Jsx>, keyof Jsx & keyof $ComponentInputProps<T, Declared>> & $ComponentInputProps<T, Declared> & $ComponentMissingInputGuard<T, Jsx> : never>";
const afterInstance =
  "$ReplaceComponentProps<I, Pick<$AuthoredKeyWitness<Jsx>, keyof Jsx & $ComponentMetadataKeys<$ComponentInputProps<T, Declared>>> & (T extends unknown ? $ComponentInputProps<T, Declared> & $ComponentInputGuard<T, Declared, Jsx> : never)>";
const sha256 = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
const identity = (value: string | Buffer) => ({
  bytes: Buffer.byteLength(value),
  sha256: sha256(value),
});

export function beforeDiscriminatedJsxUnionIn(current: string): string {
  assert.equal(current.split(afterInstance).length, 2, "current JSX-union helper is unique");
  assert.equal(current.split(beforeInstance).length, 1, "previous JSX-union helper is absent");
  return current.replace(afterInstance, beforeInstance);
}

export function applyDiscriminatedJsxUnionIn(previous: string): string {
  assert.equal(previous.split(beforeInstance).length, 2, "previous JSX-union helper is unique");
  assert.equal(previous.split(afterInstance).length, 1, "current JSX-union helper is absent");
  return previous.replace(beforeInstance, afterInstance);
}

export function beforeDiscriminatedJsxUnion(current: string): string {
  assert.deepEqual(identity(current), {
    bytes: 5870,
    sha256: discriminatedJsxUnionAfterSha256,
  });
  const previous = beforeDiscriminatedJsxUnionIn(current);
  assert.deepEqual(identity(previous), {
    bytes: 5847,
    sha256: discriminatedJsxUnionBeforeSha256,
  });
  return previous;
}

export function certifyDiscriminatedJsxUnionTransition(
  file: string,
  before: Buffer,
  after: Buffer,
) {
  if (
    file !== discriminatedJsxUnionPath ||
    before.length !== 5847 ||
    after.length !== 5870 ||
    sha256(before) !== discriminatedJsxUnionBeforeSha256 ||
    sha256(after) !== discriminatedJsxUnionAfterSha256
  )
    return;
  assert.deepEqual(Buffer.from(beforeDiscriminatedJsxUnion(after.toString("utf8"))), before);
  return {
    file,
    beforeSha256: discriminatedJsxUnionBeforeSha256,
    afterSha256: discriminatedJsxUnionAfterSha256,
    recordSha256: sha256(readFileSync(new URL(import.meta.url))),
    nativeAcceptanceCredit: false,
  };
}

/** Authenticate the only changed canonical body before packet relocation. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

const runtimePropPath = "crates/vize_canon/src/virtual_ts/helpers/declarations/core-types.d.ts.txt";
export const previousRuntimeProp =
  "type __RuntimePropValue<T> = T extends abstract new (...args: any[]) => infer V ? V : T extends (...args: any[]) => infer V ? V : never;";
export const currentRuntimeProp =
  "type __RuntimePropValue<T> = T extends (...args: any[]) => infer V ? V : T extends abstract new (...args: any[]) => infer V ? V : never;";
const previousSha256 = "1cb95f44b238332378bab53af58bbe0b95ad428d2a846f3136b81bfc3590cb98";
const currentSha256 = "3767bffe638c34126238eb7122ed70be0c5d1cbc8a79349aa10bdbc08f0820b5";
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

export function certifyCanonicalAssets(
  paths: readonly string[],
  base: readonly Buffer[],
  head: readonly Buffer[],
): void {
  assert.equal(base.length, paths.length);
  assert.equal(head.length, paths.length);
  for (const [index, file] of paths.entries()) {
    const before = base[index],
      after = head[index];
    assert.ok(before && after);
    if (before.equals(after)) continue;
    const label = `canonical helper body changed: ${file}`;
    assert.equal(file, runtimePropPath, label);
    assert.equal(before.length, 2004, label);
    assert.equal(after.length, 2004, label);
    assert.equal(digest(before), previousSha256, label);
    assert.equal(digest(after), currentSha256, label);
    const text = before.toString("utf8");
    assert.equal(text.split(previousRuntimeProp).length, 2, label);
    assert.equal(text.split(currentRuntimeProp).length, 1, label);
    assert.deepEqual(
      after,
      Buffer.from(text.replace(previousRuntimeProp, currentRuntimeProp)),
      label,
    );
  }
}

export function uniqueOffset(text: Buffer, needle: Buffer) {
  const at = text.indexOf(needle);
  assert.ok(at >= 0 && text.indexOf(needle, at + 1) < 0, "exact helper block must occur once");
  return at;
}
function occurrences(text: string, needle: string) {
  return text.split(needle).length - 1;
}
export function certifiedTransition(
  before: Buffer,
  after: Buffer,
  previous: string,
  current: string,
  label: string,
) {
  const counts = {
    beforePrevious: occurrences(before.toString(), previous),
    beforeCurrent: occurrences(before.toString(), current),
    afterPrevious: occurrences(after.toString(), previous),
    afterCurrent: occurrences(after.toString(), current),
  };
  if (
    (counts.beforePrevious === 0 &&
      counts.beforeCurrent === 0 &&
      counts.afterPrevious === 0 &&
      counts.afterCurrent === 0) ||
    (counts.beforePrevious === 1 &&
      counts.beforeCurrent === 0 &&
      counts.afterPrevious === 1 &&
      counts.afterCurrent === 0) ||
    (counts.beforePrevious === 0 &&
      counts.beforeCurrent === 1 &&
      counts.afterPrevious === 0 &&
      counts.afterCurrent === 1)
  )
    return;
  assert.deepEqual(
    counts,
    {
      beforePrevious: 1,
      beforeCurrent: 0,
      afterPrevious: 0,
      afterCurrent: 1,
    },
    `only the certified ${label} transition may change`,
  );
  return {
    start: uniqueOffset(before, Buffer.from(previous)),
    end: uniqueOffset(before, Buffer.from(previous)) + Buffer.byteLength(previous),
    replacement: Buffer.from(current),
    label,
  };
}

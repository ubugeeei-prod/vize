/** Invert the shared JSX helper includes before older source-custody checks. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

type Identity = { bytes: number; sha256: string };

const preamblePath = "crates/vize_canon/src/virtual_ts/helpers/preamble.rs";
const before: Identity = {
  bytes: 6_130,
  sha256: "6594462c0c1e163fde882056a484c0d5edc9e65d45d5f65494c2319975e2e62d",
};
const after: Identity = {
  bytes: 6_259,
  sha256: "8b6e4b50a2e8b0294d6ef1e5d7699daef0dcdd9915df72f6632fdb54e90af4a6",
};
const includes =
  '            include_str!("declarations/jsx-context.d.ts.txt"),\n' +
  '            include_str!("declarations/jsx-components.d.ts.txt"),\n';

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

/** Return the exact pre-publication preamble; unrelated source images pass through. */
export function beforeJsxSharedHelpersSource(file: string, current: string): string {
  const now = identity(current);
  if (file !== preamblePath || now.bytes !== after.bytes || now.sha256 !== after.sha256)
    return current;
  assert.equal(current.split(includes).length, 2, "shared JSX helper include block");
  const previous = current.replace(includes, "");
  assert.deepEqual(identity(previous), before, "shared JSX helper predecessor");
  return previous;
}

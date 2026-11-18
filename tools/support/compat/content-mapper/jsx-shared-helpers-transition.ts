/** Invert the published JSX helper ABI before older source-custody checks. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

type Identity = { bytes: number; sha256: string };

const globalsPath = "npm/cli/virtual-ts-globals.d.ts";
const globalsBefore: Identity = {
  bytes: 18_023,
  sha256: "3603093403379904949ed9c7c7ffd221595b2e1d8b3f7f10820ca748d7ed533d",
};
const globalsAfter: Identity = {
  bytes: 21_217,
  sha256: "d2cc4d1e40b40dac7e6362a3532ba92e9cefc4d7272a345f73f096919aba390e",
};
const globalsStart =
  "  // Keep this ABI block line-aligned with the canonical Rust declaration assets.\n";
const globalsEnd = "  type __RuntimePropValue";

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

/** Return the exact pre-publication SDK file; unrelated source images pass through. */
export function beforeJsxSharedHelpersSource(file: string, current: string): string {
  const now = identity(current);
  if (
    file === globalsPath &&
    now.bytes === globalsAfter.bytes &&
    now.sha256 === globalsAfter.sha256
  ) {
    assert.equal(current.split(globalsStart).length, 2, "shared JSX SDK block start");
    assert.equal(current.split(globalsEnd).length, 2, "shared JSX SDK block end");
    const start = current.indexOf(globalsStart);
    const end = current.indexOf(globalsEnd);
    const previous = current.slice(0, start) + current.slice(end);
    assert.deepEqual(identity(previous), globalsBefore, "shared JSX SDK predecessor");
    return previous;
  }
  return current;
}

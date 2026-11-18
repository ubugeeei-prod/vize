/** Invert the shared JSX helper includes before older source-custody checks. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

type Identity = { bytes: number; sha256: string };

const preamblePath = "crates/vize_canon/src/virtual_ts/helpers/preamble.rs";
const globalsPath = "npm/cli/virtual-ts-globals.d.ts";
const before: Identity = {
  bytes: 6_130,
  sha256: "6594462c0c1e163fde882056a484c0d5edc9e65d45d5f65494c2319975e2e62d",
};
const after: Identity = {
  bytes: 6_259,
  sha256: "8b6e4b50a2e8b0294d6ef1e5d7699daef0dcdd9915df72f6632fdb54e90af4a6",
};
const globalsBefore: Identity = {
  bytes: 18_023,
  sha256: "3603093403379904949ed9c7c7ffd221595b2e1d8b3f7f10820ca748d7ed533d",
};
const globalsAfter: Identity = {
  bytes: 21_217,
  sha256: "d2cc4d1e40b40dac7e6362a3532ba92e9cefc4d7272a345f73f096919aba390e",
};
const includes =
  '            include_str!("declarations/jsx-context.d.ts.txt"),\n' +
  '            include_str!("declarations/jsx-components.d.ts.txt"),\n';
const globalsStart =
  "  // Keep this ABI block line-aligned with the canonical Rust declaration assets.\n";
const globalsEnd = "  type __RuntimePropValue";
const contextPath = "crates/vize_canon/src/virtual_ts/helpers/declarations/jsx-context.d.ts.txt";
const componentsPath =
  "crates/vize_canon/src/virtual_ts/helpers/declarations/jsx-components.d.ts.txt";
const macrosPath = "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-macros.d.ts.txt";

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

function asset(file: string, sha256: string): string {
  const text = readFileSync(new URL(`../../../../${file}`, import.meta.url), "utf8");
  assert.equal(identity(text).sha256, sha256, `shared JSX helper asset: ${file}`);
  return text;
}

/** Return the exact pre-publication preamble; unrelated source images pass through. */
export function beforeJsxSharedHelpersSource(file: string, current: string): string {
  const now = identity(current);
  if (file === preamblePath && now.bytes === after.bytes && now.sha256 === after.sha256) {
    assert.equal(current.split(includes).length, 2, "shared JSX helper include block");
    const previous = current.replace(includes, "");
    assert.deepEqual(identity(previous), before, "shared JSX helper preamble predecessor");
    return previous;
  }
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

/** Add the authenticated JSX declarations to a composed historical preamble. */
export function applyJsxSharedHelpersIn(previous: string): string {
  const context = asset(
    contextPath,
    "00788518d757f4d15b51b06f774f89be8a5a9e615c9f9a750da4343ccf42e29f",
  );
  const components = asset(
    componentsPath,
    "c671c3ecbf63a37a726ec99a42f9259863be53f85e322f2db821529a874e0ba3",
  );
  const macros = asset(
    macrosPath,
    "1171318f948e0d14c77a0524abbad9eeab77c905cbdf58fb97c48afefb1cefcd",
  );
  assert.ok(!previous.includes(context), "JSX context is not yet shared");
  assert.ok(!previous.includes(components), "JSX component helpers are not yet shared");
  assert.equal(previous.split(macros).length, 2, "shared macro preamble anchor");
  return previous.replace(macros, context + components + macros);
}

/** Inverse of {@link applyJsxSharedHelpersIn}. */
export function beforeJsxSharedHelpersIn(current: string): string {
  const context = asset(
    contextPath,
    "00788518d757f4d15b51b06f774f89be8a5a9e615c9f9a750da4343ccf42e29f",
  );
  const components = asset(
    componentsPath,
    "c671c3ecbf63a37a726ec99a42f9259863be53f85e322f2db821529a874e0ba3",
  );
  const block = context + components;
  assert.equal(current.split(block).length, 2, "shared JSX helper block");
  return current.replace(block, "");
}

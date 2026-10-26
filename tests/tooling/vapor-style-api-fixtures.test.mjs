import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import test from "node:test";
import {
  newVaporStyleApiCases,
  VAPOR_STYLE_API_CASE_ID,
} from "../../tools/support/compat/github/vapor-style-api-fixtures.ts";

const root = "tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/";
const manifest = JSON.parse(
  readFileSync("tests/_fixtures/differential/compiler/manifest.json", "utf8"),
);
const row = manifest.vaporStyleApiCases;
const authority = JSON.parse(readFileSync(root + "authority.json", "utf8"));
// The refined producer closure; `authority.json` keeps the original campaign.
const admission = JSON.parse(readFileSync(root + "admission-source-pairs.json", "utf8"));
const files = new Map();
for (const source of admission.sourceFiles) {
  const old = spawnSync("git", ["show", `${authority.beforeRevision}:${source.path}`], {
    maxBuffer: 8 * 1024 * 1024,
  });
  files.set(`base:${source.path}`, old.status === 0 ? old.stdout : null);
  files.set(`head:${source.path}`, readFileSync(source.path));
}
for (const pin of [...row[0].inputs, ...row[0].witnesses])
  files.set(`head:${pin.path}`, readFileSync(pin.path));
const read = (side, path) => files.get(`${side}:${path}`) ?? null;

await test("actual finite Vapor CSR source witness grants exactly the real source closure", () => {
  const result = newVaporStyleApiCases(undefined, row, read);
  assert.deepEqual(result.ids, [VAPOR_STYLE_API_CASE_ID]);
  assert.deepEqual(
    [...result.sources],
    admission.sourceFiles.map((source) => source.path),
  );
  assert.equal(result.sources.size, 20);
});

await test("missing registration cannot be appeased by a generic SSR filler", () => {
  for (const current of [undefined, []])
    assert.throws(() => newVaporStyleApiCases(undefined, current, read));
});

await test("every actual producer byte and its absence observation is pinned", () => {
  for (const source of admission.sourceFiles) {
    for (const side of ["base", "head"]) {
      const changed = (which, path) =>
        which === side && path === source.path
          ? Buffer.from("changed actual source")
          : read(which, path);
      assert.throws(() => newVaporStyleApiCases(undefined, row, changed), `${side}:${source.path}`);
    }
  }
});

await test("every input, raw packet and executable witness rejects missing or mutated bytes", () => {
  for (const pin of [...row[0].inputs, ...row[0].witnesses]) {
    for (const mutation of [null, Buffer.from("changed bytes")]) {
      const changed = (side, path) =>
        side === "head" && path === pin.path ? mutation : read(side, path);
      assert.throws(() => newVaporStyleApiCases(undefined, row, changed), pin.path);
    }
  }
});

await test("schema broadening, native credit, fake selectors and future APIs are denied", () => {
  for (const mutate of [
    (value) => {
      value[0].id += "-future";
    },
    (value) => {
      value[0].targets = ["ssr"];
    },
    (value) => {
      value[0].targets.push("vapor-ssr");
    },
    (value) => {
      value[0].nativeParity = true;
    },
    (value) => {
      value[0].scope = "complete native browser support";
    },
    (value) => {
      value[0].sources.pop();
    },
    (value) => {
      value[0].sources.push({
        path: "crates/vize_atelier_sfc/src/compile.rs",
        beforeSha256: null,
        afterSha256: "0".repeat(64),
      });
    },
    (value) => {
      value[0].witnesses[0].symbol = "marker_only";
    },
    (value) => {
      value[0].witnesses = [];
    },
    (value) => {
      value[0].inputs[0].sha256 = "0".repeat(64);
    },
    (value) => {
      value.push(structuredClone(value[0]));
    },
  ]) {
    const changed = structuredClone(row);
    mutate(changed);
    assert.throws(() => newVaporStyleApiCases(undefined, changed, read));
  }
});

await test("historical rows provide no source privilege to a future source pair", () => {
  const future = (side, path) => (side === "base" ? read("head", path) : read(side, path));
  const result = newVaporStyleApiCases(row, row, future);
  assert.deepEqual(result.ids, []);
  assert.equal(result.sources.size, 0);
  assert.throws(() => newVaporStyleApiCases(undefined, row, future));
});

await test("the frozen180 source inputs are identical to all original actual packets", () => {
  const inputs = JSON.parse(readFileSync(root + "inputs.json", "utf8"));
  const original = JSON.parse(readFileSync(root + "observed-head.json", "utf8"));
  assert.equal(inputs.length, 180);
  assert.equal(original.length, 180);
  inputs.forEach((input, index) => {
    for (const key of [
      "id",
      "family",
      "source",
      "retained",
      "mapped",
      "mode",
      "ssr",
      "standalone",
      "fold",
    ])
      assert.deepEqual(input[key], original[index][key]);
  });
});

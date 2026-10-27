import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import test from "node:test";
import {
  newVaporStyleApiCases,
  VAPOR_STYLE_API_CASE_ID,
} from "../../tools/support/compat/github/vapor-style-api-fixtures.ts";

import type compilerManifest from "../_fixtures/differential/compiler/manifest.json";
import type styleAuthority from "../_fixtures/differential/compiler/vapor-literal-object-styles-10749/authority.json";
import type admissionSourcePairs from "../_fixtures/differential/compiler/vapor-literal-object-styles-10749/admission-source-pairs.json";

type StyleRows = typeof compilerManifest.vaporStyleApiCases;
type MutableStyleRows = Array<StyleRows[number] & { nativeParity?: boolean }>;
type ReadFile = Parameters<typeof newVaporStyleApiCases>[2];

const root = "tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/";
const manifest: typeof compilerManifest = JSON.parse(
  readFileSync("tests/_fixtures/differential/compiler/manifest.json", "utf8"),
);
const row = manifest.vaporStyleApiCases;
const authority: typeof styleAuthority = JSON.parse(readFileSync(root + "authority.json", "utf8"));
// The refined producer closure; `authority.json` keeps the original campaign.
const admission: typeof admissionSourcePairs = JSON.parse(
  readFileSync(root + "admission-source-pairs.json", "utf8"),
);
// This is the immutable source pair admitted by #10751. Later compiler
// changes execute the full packet/runtime workflows; they cannot inherit this
// closed historical source permission by replacing the approved hashes.
const admittedRevision = "b09ac660cd0684396c21d24249c4b1e2193e965a";
const files = new Map<string, Buffer | null>();
for (const source of admission.sourceFiles) {
  const old = spawnSync("git", ["show", `${authority.beforeRevision}:${source.path}`], {
    maxBuffer: 8 * 1024 * 1024,
  });
  files.set(`base:${source.path}`, old.status === 0 ? old.stdout : null);
  const admitted = spawnSync("git", ["show", `${admittedRevision}:${source.path}`], {
    maxBuffer: 8 * 1024 * 1024,
  });
  assert.equal(admitted.status, 0, `missing admitted source ${source.path}`);
  files.set(`head:${source.path}`, admitted.stdout);
}
for (const pin of [...row[0].inputs, ...row[0].witnesses])
  files.set(`head:${pin.path}`, readFileSync(pin.path));
const read: ReadFile = (side, path) => files.get(`${side}:${path}`) ?? null;

await test("the admitted finite Vapor CSR source witness grants exactly its historical source closure", () => {
  const result = newVaporStyleApiCases(undefined, row, read);
  assert.deepEqual(result.ids, [VAPOR_STYLE_API_CASE_ID]);
  assert.deepEqual(
    [...result.sources],
    admission.sourceFiles.map((source) => source.path),
  );
  assert.equal(result.sources.size, 20);
});

await test("current producer edits receive no privilege from the historical admission", () => {
  const changedSource = admission.sourceFiles[0].path;
  const current: ReadFile = (side, file) => {
    if (side !== "head" || !admission.sourceFiles.some((source) => source.path === file))
      return read(side, file);
    const bytes = readFileSync(file);
    // Exercise a real edit even when main still equals the historical pair.
    // A later producer must never inherit a previous row's source privilege.
    return file === changedSource ? Buffer.concat([bytes, Buffer.from("\n")]) : bytes;
  };
  const result = newVaporStyleApiCases(row, row, current);
  assert.deepEqual(result.ids, []);
  assert.equal(result.sources.size, 0);
});

await test("missing registration cannot be appeased by a generic SSR filler", () => {
  for (const current of [undefined, []])
    assert.throws(() => newVaporStyleApiCases(undefined, current, read));
});

await test("every actual producer byte and its absence observation is pinned", () => {
  for (const source of admission.sourceFiles) {
    for (const side of ["base", "head"]) {
      const changed: ReadFile = (which, path) =>
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
      const changed: ReadFile = (side, path) =>
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
  ] satisfies Array<(value: MutableStyleRows) => void>) {
    const changed = structuredClone(row);
    mutate(changed);
    assert.throws(() => newVaporStyleApiCases(undefined, changed, read));
  }
});

await test("historical rows provide no source privilege to a future source pair", () => {
  const future: ReadFile = (side, path) =>
    side === "base" ? read("head", path) : read(side, path);
  const result = newVaporStyleApiCases(row, row, future);
  assert.deepEqual(result.ids, []);
  assert.equal(result.sources.size, 0);
  assert.throws(() => newVaporStyleApiCases(undefined, row, future));
});

await test("the frozen180 source inputs are identical to all original actual packets", () => {
  const inputs: Array<Record<string, unknown>> = JSON.parse(
    readFileSync(root + "inputs.json", "utf8"),
  );
  const original: Array<Record<string, unknown>> = JSON.parse(
    readFileSync(root + "observed-head.json", "utf8"),
  );
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

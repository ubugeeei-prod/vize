import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  PATINA_NATIVE_IR_API_ID,
  patinaNativeIrApiRegistration,
  patinaNativeIrApiRoot,
} from "../../tools/support/compat/github/patina-native-ir-api-fixtures.ts";
import {
  analyzeLegacyFix,
  failingProducts,
} from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";
import { rustTestMatrix } from "../../tools/support/compat/github/rust-test-cohorts.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const manifest = "tests/_fixtures/differential/linter/manifest.json";
// Immutable admission-time pairs must not permanently pin later source bytes.
// Required Rust consumers exercise the current API independently of this record.
const blob = (oid: string) => execFileSync("git", ["cat-file", "blob", oid], { cwd: root });
const currentManifest = JSON.parse(
  readFileSync(new URL(`../../${manifest}`, import.meta.url), "utf8"),
);
const beforeManifest = structuredClone(currentManifest);
delete beforeManifest.rustApiCases;
const original = new Map<string, Buffer>([
  [`base:${manifest}`, Buffer.from(JSON.stringify(beforeManifest))],
  [`head:${manifest}`, Buffer.from(JSON.stringify(currentManifest))],
]);
for (const source of patinaNativeIrApiRegistration.sources) {
  original.set(`base:${source.path}`, blob(source.beforeBlob));
  original.set(`head:${source.path}`, blob(source.afterBlob));
}
for (const input of patinaNativeIrApiRegistration.inputs.files) {
  const path = `${patinaNativeIrApiRoot}/${input.path}`;
  original.set(`head:${path}`, readFileSync(new URL(`../../${path}`, import.meta.url)));
}
const paths = [...patinaNativeIrApiRegistration.sources.map(({ path }) => path), manifest];

function report(tree = original, changed = paths) {
  return analyzeLegacyFix({
    title: "fix(patina): preserve deprecated native IR accessors",
    paths: changed,
    readFile: (side, path) => tree.get(`${side}:${path}`) ?? null,
  });
}

function editedCase(edit: (row: typeof patinaNativeIrApiRegistration) => void) {
  const tree = new Map(original);
  const current = JSON.parse(tree.get(`head:${manifest}`)!.toString());
  edit(current.rustApiCases[0]);
  tree.set(`head:${manifest}`, Buffer.from(JSON.stringify(current)));
  return tree;
}

test("the actual Rust API inputs and exact producer/support pairs satisfy the fix gate", () => {
  assert.deepEqual(report().products, [
    {
      product: "linter",
      state: "new-inputs-registered",
      cases: [PATINA_NATIVE_IR_API_ID],
    },
  ]);
  assert.deepEqual(failingProducts(report()), []);
});

test("closed IDs, adapters, getter data, paths and test selectors reject substitutions", () => {
  const edits: Array<[string, (row: typeof patinaNativeIrApiRegistration) => void]> = [
    [
      "ID",
      (row) => {
        row.id = "linter/rust-api/unreviewed";
      },
    ],
    [
      "adapter",
      (row) => {
        row.adapter = "unreviewed";
      },
    ],
    [
      "target",
      (row) => {
        row.targets = ["lint"];
      },
    ],
    [
      "missing producer",
      (row) => {
        row.sources.splice(1, 1);
      },
    ],
    [
      "unknown source",
      (row) => {
        row.sources[1].path = "crates/vize_patina/src/rules.rs";
      },
    ],
    [
      "missing selector",
      (row) => {
        row.tests.pop();
      },
    ],
    [
      "unknown selector",
      (row) => {
        row.tests[0].selector = "unrelated::test";
      },
    ],
    [
      "fixture path",
      (row) => {
        row.inputs.files[0].path = "unrelated.json";
      },
    ],
  ];
  for (const [label, edit] of edits) {
    const result = report(editedCase(edit));
    assert.equal(result.products[0].state, "invalid-manifest-or-input", label);
    assert.deepEqual(failingProducts(result), ["linter"], label);
  }
  for (const edit of [
    (data: { getters: Array<{ legacy: string }> }) => {
      data.getters[0].legacy = "unreviewed_getter";
    },
    (data: { getters: Array<{ legacy: string }> }) => {
      data.getters.pop();
    },
  ]) {
    const tree = new Map(original);
    const path = `head:${patinaNativeIrApiRoot}/cases.json`;
    const data = JSON.parse(tree.get(path)!.toString());
    edit(data);
    tree.set(path, Buffer.from(JSON.stringify(data)));
    assert.match(report(tree).products[0].reason ?? "", /head hash mismatch/);
  }
});

test("missing or altered actual fixtures and every predecessor/postimage fail closed", () => {
  for (const key of [...original.keys()].filter((key) => !key.endsWith(manifest))) {
    const missing = new Map(original);
    missing.delete(key);
    assert.equal(report(missing).products[0].state, "invalid-manifest-or-input", key);
    const changed = new Map(original);
    changed.set(key, Buffer.concat([changed.get(key)!, Buffer.from("\nchanged\n")]));
    assert.equal(report(changed).products[0].state, "invalid-manifest-or-input", key);
  }
});

test("an unrelated legacy edit still needs its own language differential input", () => {
  const result = report(original, [...paths, "crates/vize_patina/src/rules/vue/no_v_html.rs"]);
  assert.equal(result.products[0].state, "new-input-missing");
  assert.deepEqual(failingProducts(result), ["linter"]);
  const tree = new Map(original);
  const current = JSON.parse(tree.get(`head:${manifest}`)!.toString());
  delete current.rustApiCases;
  tree.set(`head:${manifest}`, Buffer.from(JSON.stringify(current)));
  assert.equal(report(tree).products[0].state, "new-input-missing");
});

test("a historical API registration cannot be reused for a subsequent fix", () => {
  const tree = new Map(original);
  tree.set(`base:${manifest}`, tree.get(`head:${manifest}`)!);
  assert.equal(report(tree).products[0].state, "new-input-missing");
});

test("historical registrations still authenticate their current Rust consumer and getter inputs", () => {
  const tree = new Map(original);
  tree.set(`base:${manifest}`, tree.get(`head:${manifest}`)!);
  for (const input of patinaNativeIrApiRegistration.inputs.files) {
    const key = `head:${patinaNativeIrApiRoot}/${input.path}`;
    const missing = new Map(tree);
    missing.delete(key);
    assert.equal(report(missing).products[0].state, "invalid-manifest-or-input", key);
    const changed = new Map(tree);
    changed.set(key, Buffer.concat([changed.get(key)!, Buffer.from("\nchanged\n")]));
    assert.equal(report(changed).products[0].state, "invalid-manifest-or-input", key);
  }
});

test("historical admission records do not read or freeze later live implementations", () => {
  const tree = new Map(original);
  tree.set(`base:${manifest}`, tree.get(`head:${manifest}`)!);
  let liveSourceReads = 0;
  const result = analyzeLegacyFix({
    title: "fix(patina): a later implementation change",
    paths,
    readFile: (side, path) => {
      if (patinaNativeIrApiRegistration.sources.some((source) => source.path === path)) {
        liveSourceReads++;
        throw new Error("a historical case must not read this changed live source");
      }
      return tree.get(`${side}:${path}`) ?? null;
    },
  });
  assert.equal(liveSourceReads, 0);
  assert.equal(result.products[0].state, "new-input-missing");
  assert.deepEqual(failingProducts(result), ["linter"]);
});

test("the real Rust cohort selects the entire package, including lib and integration consumers", () => {
  const matrix = rustTestMatrix(
    {
      workspace_root: root,
      workspace_members: ["vize_patina"],
      packages: [
        {
          id: "vize_patina",
          name: "vize_patina",
          dependencies: [],
          manifest_path: `${root}/crates/vize_patina/Cargo.toml`,
        },
      ],
    },
    ["vize_patina"],
  );
  assert.equal(matrix.include.length, 1);
  assert.equal(matrix.include[0].cohort, "lint-and-format");
  assert.ok(matrix.include[0].filter.split(" | ").includes("package(=vize_patina)"));
});

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { analyzeLegacyFix } from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";
import { loadTypecheckerManifest } from "../differential/typechecker.ts";
import { assertSortedCases, summarizeDialectCoverage } from "../differential/harness.mjs";
import { sourceInspectedDialectCoverage } from "../differential/source-dialect-coverage.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const manifestPath = "tests/_fixtures/differential/typechecker/manifest.json";
const inputPath =
  "tests/_fixtures/differential/typechecker/windows-verbatim-overlay-11145/input.json";
const id = "typechecker/regression/windows-verbatim-overlay-11145";
const sha = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const raw = fs.readFileSync(path.join(root, inputPath));
const input = JSON.parse(raw.toString("utf8"));
const manifest = JSON.parse(fs.readFileSync(path.join(root, manifestPath), "utf8"));
const row = manifest.cases.find((value: { id: string }) => value.id === id);
const withoutInput = {
  ...manifest,
  cases: manifest.cases.filter((value: { id: string }) => value.id !== id),
};

test("Windows overlay input pins the genuine main/imported-child/support/config and all revision bytes", () => {
  assert.equal(raw.length, 1404);
  assert.equal(sha(raw), "09e9e861a991d5ba37cd88c87a3a70a86ab2443cbf1b4443243515bb7c8af4f4");
  assert.deepEqual(
    Object.keys(input).sort(),
    ["schema", "id", "main", "scriptMain", "child", "support", "configText", "phases"].sort(),
  );
  assert.equal(input.schema, 1);
  assert.equal(input.id, id);
  for (const source of [input.main, input.scriptMain]) {
    assert(source.includes("import Child from './Child.vue'"));
    assert(source.includes("import { value } from '@/support'"));
    assert(source.includes("const checked: number = value"));
  }
  const config = JSON.parse(input.configText);
  assert.deepEqual(config, {
    compilerOptions: { baseUrl: ".", paths: { "@/*": ["src/*"] } },
    include: ["src/**/*.vue"],
  });
  assert.deepEqual(
    input.phases.map((value: { id: string; support: string | null; child: string | null }) => {
      assert.deepEqual(Object.keys(value).sort(), ["child", "id", "support"]);
      return [value.id, value.support, value.child];
    }),
    [
      ["valid", input.support, input.child],
      [
        "dependency-error",
        "export const value = 'bad';\n",
        input.child.replace("saved-child", "dirty-child"),
      ],
      [
        "dependency-repair",
        "export const value = 2;\n",
        input.child.replace("saved-child", "repaired-child"),
      ],
      ["dependency-close", null, null],
    ],
  );
});

test("one input-only row adds source inspection without a diagnostic pack or native admission", () => {
  assert(row);
  assert.equal(row.state, "active");
  assert.deepEqual(row.targets, ["batch"]);
  assert.deepEqual(row.inputs, [
    {
      path: "windows-verbatim-overlay-11145/input.json",
      sha256: sha(raw),
      role: "regression-reproduction",
    },
  ]);
  for (const field of [
    "pack",
    "test",
    "packReference",
    "expectations",
    "adapters",
    "nativeAdmission",
  ])
    assert.equal(Object.hasOwn(row, field), false);
  const coverage = sourceInspectedDialectCoverage(
    [
      { filename: "src/App.vue", source: input.main },
      { filename: "src/main.ts", source: input.scriptMain },
      { filename: "src/Child.vue", source: input.child },
      { filename: "src/support.ts", source: input.support },
    ],
    row.dialectCoverage.evidence[0].selector,
  );
  assert.deepEqual(row.dialectCoverage, coverage);
  assert.deepEqual(coverage.evidence[0]!.dialects, ["vue3-sfc", "html-template", "ts"]);
  assert.equal(coverage.state, "source-inspected");
  assert(
    !loadTypecheckerManifest(path.join(root, manifestPath)).cases.some(
      (value: { id: string }) => value.id === id,
    ),
  );
});

test("the genuine pinned input satisfies source registration and refuses missing or modified bytes", () => {
  const paths = ["crates/vize_canon/src/corsa_bridge/vue_document/build/tests/windows_overlays.rs"];
  const read = (side: "base" | "head", file: string): Buffer | null => {
    if (file === manifestPath)
      return Buffer.from(JSON.stringify(side === "base" ? withoutInput : manifest));
    try {
      return fs.readFileSync(path.join(root, file));
    } catch {
      return null;
    }
  };
  const report = (replacement?: Buffer | null) =>
    analyzeLegacyFix({
      title: "fix(canon): preserve Windows overlay revision inputs",
      paths,
      readFile: (side, file) =>
        side === "head" && file === inputPath && replacement !== undefined
          ? replacement
          : read(side, file),
    });
  assert.deepEqual(report().products, [
    { product: "typechecker", state: "new-inputs-registered", cases: [id] },
  ]);
  assert.equal(report(null).products[0]!.state, "invalid-manifest-or-input");
  assert.equal(
    report(Buffer.concat([raw, Buffer.from(" ")])).products[0]!.state,
    "invalid-manifest-or-input",
  );
});

test("one source-inspected input adds no verified readiness and retains the historical wire source", () => {
  assertSortedCases(manifest.cases);
  assert.equal(manifest.cases.filter((value: { id: string }) => value.id === id).length, 1);
  const oldPairs =
    "tests/_fixtures/differential/typechecker/native-beta-diagnostic-dto-10731/source-pairs.json";
  assert.equal(
    sha(fs.readFileSync(path.join(root, oldPairs))),
    "512e95f921e6ec064f281ec00d777cab0df66964d69087384cbb8615c22e21b1",
  );
  const before = summarizeDialectCoverage(withoutInput.cases),
    after = summarizeDialectCoverage(manifest.cases);
  assert.equal(after.plannedCases, before.plannedCases + 1);
  assert.equal(after.unknownReadinessCases, before.unknownReadinessCases + 1);
  assert.equal(after.verifiedCases, before.verifiedCases);
  assert.deepEqual(after.verified, before.verified);
});

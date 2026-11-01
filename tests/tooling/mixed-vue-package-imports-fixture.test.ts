import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { sourceInspectedDialectCoverage } from "../differential/source-dialect-coverage.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const inputPath =
  "tests/_fixtures/differential/typechecker/mixed-vue-package-imports-11253/input.json";
const raw = fs.readFileSync(path.join(root, inputPath));
const sha = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const input = JSON.parse(raw.toString()) as {
  schema: string;
  id: string;
  files: { path: string; source: string }[];
};
const manifest = JSON.parse(
  fs.readFileSync(
    path.join(root, "tests/_fixtures/differential/typechecker/manifest.json"),
    "utf8",
  ),
) as {
  cases: {
    id: string;
    inputs: { path: string; sha256: string; role: string }[];
    dialectCoverage: unknown;
  }[];
};
const row = manifest.cases.find((entry) => entry.id === input.id);

test("the consumed reproduction retains all five original file bodies and the reported imports", () => {
  assert.equal(sha(raw), "41f272617bd60acc5c737f4d72327edcc0e85e53ca161ef076ca8509072c1ed6");
  assert.equal(input.schema, "vize.mixed-vue-package-imports");
  assert.equal(input.id, "typechecker/regression/mixed-vue-package-imports-11253");
  assert.deepEqual(
    input.files.map((file) => [
      file.path,
      Buffer.byteLength(file.source),
      sha(Buffer.from(file.source)),
    ]),
    [
      ["package.json", 260, "c5bc1e91d32304525fc093dbe8d1b828479382d5d22051f8b3769822c99f7461"],
      ["tsconfig.json", 225, "ee5fca5e89d5c8f4d98aeb6175b41eb702baec8e52c2d4e1b4725d8b35959c0d"],
      ["src/App.vue", 107, "46c19e03d37447d379f56d4d43460ffd33de737a678d279182d53385590dd7a5"],
      ["src/main.ts", 135, "7e0b11adbb8653c81e6a7f6c951987a5593ae7a82d4783465c7f97bc49a97a35"],
      ["src/plain.ts", 81, "f122f10ec44e72f111262f782d61de2d37b4033cc2d3783137ab9ee98d60defc"],
    ],
  );
  assert.deepEqual(JSON.parse(input.files[0].source).imports, { "#src/*": "./src/*.ts" });
});

test("the one input-only row authenticates real authored dialects without inventing native admission", () => {
  assert(row);
  assert.deepEqual(row.inputs, [
    {
      path: "mixed-vue-package-imports-11253/input.json",
      sha256: sha(raw),
      role: "regression-reproduction",
    },
  ]);
  for (const field of [
    "pack",
    "test",
    "packReference",
    "adapters",
    "expectations",
    "nativeAdmission",
  ])
    assert.equal(Object.hasOwn(row, field), false);
  const coverage = row.dialectCoverage as { evidence: { selector: string }[] };
  assert.deepEqual(
    row.dialectCoverage,
    sourceInspectedDialectCoverage(
      input.files
        .filter((file) => /\.(?:vue|ts)$/u.test(file.path))
        .map((file) => ({ filename: file.path, source: file.source })),
      coverage.evidence[0].selector,
    ),
  );
});

test("both Git autocrlf modes retain the pinned original input bytes", () => {
  for (const autocrlf of ["false", "true"])
    assert.deepEqual(
      execFileSync(
        "git",
        ["-c", `core.autocrlf=${autocrlf}`, "cat-file", "--filters", `HEAD:${inputPath}`],
        { cwd: root },
      ),
      raw,
    );
});

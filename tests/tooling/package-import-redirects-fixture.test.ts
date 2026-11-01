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
  "tests/_fixtures/differential/typechecker/package-import-redirects-11317/input.json";
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
  assert.equal(sha(raw), "13e0288f0d69e7be48366148023c8ec2324fd650e8396966ce46fabb1449d0b4");
  assert.equal(input.schema, "vize.package-import-redirects");
  assert.equal(input.id, "typechecker/regression/package-import-redirects-11317");
  assert.deepEqual(
    input.files.map((file) => [
      file.path,
      Buffer.byteLength(file.source),
      sha(Buffer.from(file.source)),
    ]),
    [
      ["package.json", 255, "08f4cfe109af9537ffdef766d938740269e964844097670dff0d4854ece09332"],
      ["tsconfig.json", 225, "ee5fca5e89d5c8f4d98aeb6175b41eb702baec8e52c2d4e1b4725d8b35959c0d"],
      ["src/main.ts", 61, "69b4fbb7f9c187371ef0eb580641636e9dcb47881d67de24de9527440a6183eb"],
      ["src/plain.ts", 122, "268fe558ea66b8f47689eb3688cf0dda6c96366705b018e6256da5957f8e64d3"],
      ["src/Other.vue", 38, "cdba2734bf2fe1c7db516518142c8e6085fa68cbf5519360641963e3d21e7fb9"],
    ],
  );
  assert.deepEqual(JSON.parse(input.files[0].source).imports, { "#src/*": "./src/*.ts" });
});

test("the one input-only row authenticates real authored dialects without inventing native admission", () => {
  assert(row);
  assert.deepEqual(row.inputs, [
    {
      path: "package-import-redirects-11317/input.json",
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

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { sourceInspectedDialectCoverage } from "../differential/source-dialect-coverage.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const relative = "tests/_fixtures/differential/linter/symbol-owner-probes-11267";
const pinned = [
  ["Required.ts.txt", "a16feef8a905ea3918e924764f56d4ba3f07be149254f52cd80c08453d5cfa28"],
  ["Optional.ts.txt", "b194898883ef4107518f6d242296d794f5c4a95e79c46b25ab7ee70dc4850bea"],
  ["Rest.ts.txt", "81cebcbf1ff41c3d1bac1fe2349857d74e4c4dd944b33f493e9b3363ab9eb634"],
  ["This.ts.txt", "40b96eff2b668bff75199a9c19e15115e9f2396c8ec48c3bb8539119538cb7c5"],
  ["Overloads.ts.txt", "8f3cfe2f3b7fb8f7021a58c206e155a2278e09d31e6ab5727f2aed29a31d8cc1"],
  ["Generic.ts.txt", "26826ea287e287279f72a009af9279b301dfea8a4d7433214921cd865a471b08"],
  ["Zero.ts.txt", "ff4f1805a4fd39bdf48be10878b102265a9b3b3bd31d3182c88364443b3e9495"],
  ["Probe.ts.txt", "e0a254faa8138b02e3a7a571f338a2ac2e58e983be461891d57a954eb0e46a14"],
  ["package.json.txt", "a90fc33418afa2b9cbf4768524b021135b965ecdf993457505bfae3ac44a8428"],
  ["tsconfig.json.txt", "4d2cc180f026c57b51bfc101123ca7e19e8d1be54755e2f9e9f826868fb2c642"],
  ["App.vue.txt", "9534d47afd07e350aff5521da92f3ad7b960788ab8a66ce061437ccfc9a943ef"],
  ["Untyped.ts.txt", "e0bd6c85fa4bf04f3aea48d8444a73d8ffd32ed9ec9bceaa47d01d0b1f5d68d1"],
] as const;
const files = pinned.map(([filename]) => ({
  filename,
  source: fs.readFileSync(path.join(root, relative, filename), "utf8"),
}));
const sha = (raw: Buffer) => createHash("sha256").update(raw).digest("hex");

test("all seven signature forms and the project inputs are pinned and consumed by the real owner test", () => {
  const consumer = [
    "crates/vize_patina/src/linter/corsa_session/probe/signature_owner_tests.rs",
    "crates/vize/tests/lint_strict_boolean_cli/signature_owner.rs",
  ]
    .map((file) => fs.readFileSync(path.join(root, file), "utf8"))
    .join("\n");
  assert.deepEqual(
    files.map(({ filename, source }) => [filename, sha(Buffer.from(source))]),
    pinned,
  );
  for (const { filename } of files) assert(consumer.includes(`fixture!("${filename}")`));
  assert.equal(
    files[7].source,
    'import { check } from "./dependency";\nexport const probe = check;\n',
  );
  assert.deepEqual(JSON.parse(files[8].source), { private: true, type: "module" });
  assert.deepEqual(JSON.parse(files[9].source), {
    compilerOptions: {
      target: "ESNext",
      module: "ESNext",
      moduleResolution: "bundler",
      strict: true,
      noEmit: true,
    },
    include: ["*.ts"],
  });
});

test("the real authored input-only row carries no invented native acceptance", () => {
  const manifest = JSON.parse(
    fs.readFileSync(path.join(root, "tests/_fixtures/differential/linter/manifest.json"), "utf8"),
  ) as {
    cases: { id: string; inputs: unknown; dialectCoverage: unknown }[];
  };
  const matches = manifest.cases.filter((row) => row.id === "linter/symbol-owner-probes-11267");
  assert.equal(matches.length, 1);
  const row = matches[0];
  assert.deepEqual(row.inputs, {
    root: "symbol-owner-probes-11267",
    files: pinned.map(([filename, sha256]) => ({
      path: filename,
      role: filename.endsWith(".json.txt") ? "project-configuration" : "causal-regression",
      sha256,
    })),
  });
  assert.deepEqual(
    row.dialectCoverage,
    sourceInspectedDialectCoverage(files, "linter manifest pinned authored sources"),
  );
  for (const field of ["pack", "expectations", "nativeAdmission", "adapters", "oracle"])
    assert.equal(Object.hasOwn(row, field), false);
});

test("both Git checkout modes retain every pinned input byte", () => {
  for (const [filename] of pinned)
    for (const autocrlf of ["false", "true"])
      assert.deepEqual(
        execFileSync(
          "git",
          [
            "-c",
            `core.autocrlf=${autocrlf}`,
            "cat-file",
            "--filters",
            `HEAD:${relative}/${filename}`,
          ],
          { cwd: root },
        ),
        fs.readFileSync(path.join(root, relative, filename)),
      );
});

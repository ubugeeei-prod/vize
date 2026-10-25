import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
const git = (...args) => execFileSync("git", args, { encoding: "utf8" }).trim();
const hash = (data) => createHash("sha256").update(data).digest("hex");
const base = "b3ef5520644577a1a636e1d33026991bb29b0c9c";
const shutdown = "6b8e0de49c088e1011141178b36e720579800f54";
const harness = "91cfa49f7a36a309493d20ec2168296502a78aa3";
const borrowed = JSON.parse(
  fs.readFileSync("tests/tooling/support/native-beta-dto/validation-harness.json", "utf8"),
);
const streaming = borrowed.streamingReference;
const prefix = "tests/tooling/support/native-runtime-selection/";
const names = borrowed.files
  .filter((row) => row.revision === harness && row.path.startsWith(prefix))
  .map((row) => row.path);
assert.equal(names.length, 13);
const paths = [
  ...new Set([
    ...names,
    prefix + "lsp-strace-lines.mjs",
    prefix + "lsp-native-beta-dto.mjs",
    ".github/workflows/native-beta-diagnostic-dto.yml",
    ...borrowed.shutdownSourceFiles.map((row) => row.path),
  ]),
];
const files = paths.map((file) => {
  const current = fs.readFileSync(file),
    pinned = execFileSync("git", ["show", `${git("rev-parse", "HEAD")}:${file}`]);
  assert.deepEqual(current, pinned, "actual actor source/module bytes must be committed");
  const original = borrowed.files.find((row) => row.path === file && row.revision === harness);
  const closed = borrowed.shutdownSourceFiles.find((row) => row.path === file);
  if (closed)
    assert.equal(hash(current), closed.sha256, "exact qualified shutdown source leaf bytes");
  return {
    path: file,
    sha256: hash(current),
    originalHarnessSha256: original?.sha256 ?? null,
    observerDelta: original ? hash(current) !== original.sha256 : null,
  };
});
for (const file of [
  "crates/vize_canon/src/lsp_client/initial_diagnostics.rs",
  "crates/vize_canon/src/lsp_client/editor_lsp/client.rs",
])
  assert.deepEqual(
    fs.readFileSync(file),
    execFileSync("git", ["show", `${base}:${file}`]),
    "no 10733 production injection",
  );
const out = "target/vize-tests/native-beta-dto-composition";
fs.mkdirSync(out, { recursive: true });
fs.writeFileSync(
  path.join(out, "composition.json"),
  JSON.stringify(
    {
      role: "validation-only combined source; not a publication or source-session credit",
      source: git("rev-parse", "HEAD"),
      tree: git("rev-parse", "HEAD^{tree}"),
      dtoProductionBase: base,
      shutdownSourceLeaf: shutdown,
      harnessReference: harness,
      streamingReference: streaming,
      immutableBefore: "13fa41356d075047a229f580014b637d3bbeb57f",
      files,
    },
    null,
    2,
  ) + "\n",
);

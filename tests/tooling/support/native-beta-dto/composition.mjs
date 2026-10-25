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
    prefix + "lsp-native-beta-workspace.mjs",
    prefix + "lsp-api-snapshot-drain.mjs",
    "crates/vize_canon/src/lsp_client/editor_lsp.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/project_configuration.rs",
    ".cargo/config.toml",
    "Cargo.lock",
    "crates/vize_canon/src/lsp_client/editor_lsp/client.rs",
    "crates/vize_canon/src/corsa_bridge/bridge/documents.rs",
    "tests/tooling/support/native-beta-dto/queued-retirement/composition.json",
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
assert.deepEqual(
  fs.readFileSync("crates/vize_canon/src/lsp_client/initial_diagnostics.rs"),
  execFileSync("git", ["show", `${base}:crates/vize_canon/src/lsp_client/initial_diagnostics.rs`]),
  "no 10733 initial diagnostics injection",
);
const queued = JSON.parse(
  fs.readFileSync(
    "tests/tooling/support/native-beta-dto/queued-retirement/composition.json",
    "utf8",
  ),
);
const queuedMethod = fs.readFileSync(
  "tests/tooling/support/native-beta-dto/queued-retirement/method.rs.txt",
  "utf8",
);
assert.equal(hash(Buffer.from(queuedMethod)), queued.methodSha256);
const clientPath = "crates/vize_canon/src/lsp_client/editor_lsp/client.rs";
const clientBefore = execFileSync("git", ["show", `${base}:${clientPath}`], { encoding: "utf8" });
assert.equal(
  fs.readFileSync(clientPath, "utf8"),
  clientBefore.replace(
    "    /// Drop the editor session immediately after an external project-shape\n",
    queuedMethod + "    /// Drop the editor session immediately after an external project-shape\n",
  ),
  "only the exact approved private queued retirement method is added",
);
for (const row of queued.files) assert.equal(hash(fs.readFileSync(row.path)), row.afterSha256);

const out = "target/vize-tests/native-beta-dto-composition";
fs.mkdirSync(out, { recursive: true });
fs.writeFileSync(
  path.join(out, "composition.json"),
  JSON.stringify(
    {
      role: "LOCAL-PATCHED-SDK validation-only combined source; not stock SDK, publication or source-session credit",
      source: git("rev-parse", "HEAD"),
      tree: git("rev-parse", "HEAD^{tree}"),
      dtoProductionBase: base,
      shutdownSourceLeaf: shutdown,
      harnessReference: harness,
      streamingReference: streaming,
      queuedNormalRetirement: queued,
      immutableBefore: "13fa41356d075047a229f580014b637d3bbeb57f",
      files,
    },
    null,
    2,
  ) + "\n",
);

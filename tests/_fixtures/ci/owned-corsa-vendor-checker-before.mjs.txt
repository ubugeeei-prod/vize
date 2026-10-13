import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const custody = JSON.parse(readFileSync(join(root, "vendor/corsa-published-custody.json")));
const modified = {
  corsa_core: ["src/lib.rs", "src/process.rs"],
  corsa_lsp: ["src/lib.rs", "src/lsp/client.rs"],
};
const additions = { corsa_core: ["LICENSE", "src/process_receipt.rs"], corsa_lsp: ["LICENSE"] };
const report = {};
const sha = (path) => createHash("sha256").update(readFileSync(path)).digest("hex");
for (const crate of ["corsa_core", "corsa_lsp"]) {
  const entry = custody[crate];
  assert.equal(entry.version, "1.14.0");
  assert.deepEqual(Object.keys(entry.modifiedPublishedFiles).sort(), modified[crate].sort());
  assert.deepEqual(Object.keys(entry.addedFiles).sort(), additions[crate].sort());
  let unchanged = 0;
  for (const [path, original] of Object.entries(entry.files)) {
    const patch = entry.modifiedPublishedFiles[path];
    if (patch) assert.equal(patch.originalSha256, original);
    else unchanged++;
    assert.equal(sha(join(root, "vendor", crate, path)), patch?.patchedSha256 ?? original, path);
  }
  for (const [path, expected] of Object.entries(entry.addedFiles)) {
    assert.equal(sha(join(root, "vendor", crate, path)), expected, path);
  }
  assert.equal(sha(join(root, "vendor", crate, "LICENSE")), custody.upstreamLicense.sha256);
  const vcs = JSON.parse(readFileSync(join(root, "vendor", crate, ".cargo_vcs_info.json")));
  assert.equal(vcs.git.sha1, "34bb731bdcaf5d4c154596b0c32912cbffd60b18");
  report[crate] = {
    archiveSha256: entry.archiveSha256,
    originalFiles: Object.keys(entry.files).length,
    unchanged,
    modified: modified[crate],
    added: additions[crate],
    revision: vcs.git.sha1,
  };
}
console.log(JSON.stringify(report, null, 2));

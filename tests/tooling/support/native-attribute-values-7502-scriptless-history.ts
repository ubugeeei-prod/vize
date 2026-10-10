// The original V1 gate is independently run on its pinned source, not recreated.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { baseline7502, exactKeys7502, hash7502 } from "./native-attribute-values-7502-inputs.ts";

const archive = new URL(
  "../../../crates/vize_atelier_sfc/tests/fixtures/native-scriptless-ssr-output.json",
  import.meta.url,
);
const archiveSha = "42d07bc4f7c6d19c67d83e4f3d13221e68700aa8c01a8cf7ccad4847088b4773";
export function scriptlessHistory7502(history: string, worktree: string) {
  const original = readFileSync(archive);
  assert.equal(hash7502(original), archiveSha);
  assert(
    original.equals(
      readFileSync(
        path.join(
          worktree,
          "crates/vize_atelier_sfc/tests/fixtures/native-scriptless-ssr-output.json",
        ),
      ),
    ),
  );
  const receipt = {
    sourceRevision: baseline7502.revision,
    sourceTree: baseline7502.tree,
    archiveSha256: archiveSha,
    captureSha256: hash7502(readFileSync(path.join(history, "scriptless.capture.json"))),
    runtimeSha256: hash7502(readFileSync(path.join(history, "scriptless.runtime.json"))),
  };
  validateScriptlessHistory7502(receipt, history);
  return receipt;
}
export function validateScriptlessHistory7502(receipt: any, history: string) {
  exactKeys7502(receipt, [
    "sourceRevision",
    "sourceTree",
    "archiveSha256",
    "captureSha256",
    "runtimeSha256",
  ]);
  assert.equal(receipt.sourceRevision, baseline7502.revision);
  assert.equal(receipt.sourceTree, baseline7502.tree);
  assert.equal(receipt.archiveSha256, archiveSha);
  const original = readFileSync(archive);
  assert.equal(hash7502(original), archiveSha);
  const frozen = JSON.parse(original.toString("utf8"));
  for (const [key, file, expected] of [
    ["captureSha256", "scriptless.capture.json", frozen.capture],
    ["runtimeSha256", "scriptless.runtime.json", frozen.runtime],
  ]) {
    const bytes = readFileSync(path.join(history, file));
    assert.equal(receipt[key], hash7502(bytes));
    assert.deepEqual(
      JSON.parse(bytes.toString("utf8")),
      expected,
      "whole untouched historical scriptless gate remains exact",
    );
  }
}

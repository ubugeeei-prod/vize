import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { root } from "./support/lsp/paths.ts";
import { matrix, rowsRequiringExecutedEvidence } from "./support/typed-editor-oracle-matrix.ts";

test("typed editor oracle matrix covers or explicitly tracks every P0 slice", () => {
  assert.deepEqual(
    matrix.map((row) => row.id),
    [
      "cli-authored-script-diagnostics",
      "lsp-authored-script-diagnostics",
      "cli-fallthrough-template-range",
      "lsp-fallthrough-template-range",
      "lsp-hover-definition-template-anchors",
      "cli-jsx-intrinsic-globals",
      "lsp-jsx-intrinsic-globals",
      "vscode-host-reactive-hover-surface",
      "vscode-host-component-contract-hover",
      "lsp-imported-component-contract-hover",
      "lsp-component-event-contract-navigation",
      "lsp-component-prop-reexport-hover",
      "lsp-component-v-model-navigation",
      "lsp-static-template-ref-navigation",
      "non-vscode-host-reactive-hover-surface",
      "non-vscode-host-component-contract-hover",
    ],
  );

  for (const row of matrix) {
    assert.match(row.followUp, /^#\d+(?:[,\s]+#\d+)*$/, `${row.id} must point at tracking issues`);
    if (row.status === "covered") {
      assert.ok(
        row.evidence.some((entry) => entry.kind === "file"),
        `${row.id} needs file proof`,
      );
      if (rowsRequiringExecutedEvidence.has(row.id)) {
        assert.ok(
          row.evidence.some((entry) => entry.kind === "ci"),
          `${row.id} needs mandatory CI execution proof`,
        );
      }
    } else {
      assert.ok(
        row.evidence.every((entry) => entry.kind === "pending-pr" && entry.reason.length > 0),
        `${row.id} known gaps need explicit pending PR rationale`,
      );
    }
  }
});

test("covered typed editor oracle matrix rows point at live gate files and CI gates", () => {
  for (const row of matrix) {
    for (const evidence of row.evidence) {
      if (evidence.kind !== "file" && evidence.kind !== "ci") continue;
      const source = fs.readFileSync(path.join(root, evidence.path), "utf8");
      for (const text of evidence.requiredText) {
        assert.match(source, new RegExp(escapeRegExp(text)), `${row.id} missing ${text}`);
      }
    }
  }
});

test("covered LSP tooling rows require executed CI evidence", () => {
  const missing = matrix
    .filter(
      (row) =>
        row.status === "covered" &&
        row.id.startsWith("lsp-") &&
        row.evidence.some(
          (evidence) => evidence.kind === "file" && evidence.path.startsWith("tests/tooling/"),
        ) &&
        !rowsRequiringExecutedEvidence.has(row.id),
    )
    .map((row) => row.id);

  assert.deepEqual(missing, []);
});

test("typed editor oracle matrix document mirrors the executable ledger", () => {
  const doc = fs.readFileSync(
    path.join(root, "docs/release/typed-editor-oracle-matrix.md"),
    "utf8",
  );
  for (const row of matrix) {
    for (const issue of row.followUp.match(/#\d+/g) ?? []) {
      assert.match(doc, new RegExp(escapeRegExp(issue)), `${row.id} missing ${issue}`);
    }
    if (row.status === "known-gap") {
      for (const evidence of row.evidence) {
        if (evidence.kind === "pending-pr") {
          assert.match(doc, new RegExp(`pending PR #${evidence.pr}`), `${row.id} missing PR`);
        }
      }
    } else {
      for (const evidence of row.evidence) {
        if (evidence.kind === "file" || evidence.kind === "ci") {
          assert.match(doc, new RegExp(escapeRegExp(evidence.path)), `${row.id} missing file`);
          if (evidence.kind === "ci") {
            assert.match(
              doc,
              new RegExp(escapeRegExp(evidence.label)),
              `${row.id} missing CI evidence label`,
            );
          }
        }
      }
    }
  }
});

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

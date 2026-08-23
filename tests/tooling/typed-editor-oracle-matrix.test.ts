import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { root } from "./support/lsp/paths.ts";

type MatrixRow = {
  evidence: Evidence[];
  followUp: string;
  id: string;
  status: "covered" | "known-gap";
};

type Evidence =
  | { kind: "file"; path: string; requiredText: string[] }
  | { kind: "pending-pr"; pr: number; reason: string };

const matrix: MatrixRow[] = [
  {
    evidence: [
      {
        kind: "file",
        path: "crates/vize/tests/check_text_diagnostics_cli.rs",
        requiredText: [
          "const count: string = 0;",
          "Type 'number' is not assignable to type 'string'.",
        ],
      },
    ],
    followUp: "#4587",
    id: "cli-authored-script-diagnostics",
    status: "covered",
  },
  {
    evidence: [
      {
        kind: "file",
        path: "tests/tooling/lsp-authored-script-diagnostics.test.ts",
        requiredText: ["const a: string = 1", "params.version === 2"],
      },
    ],
    followUp: "#4587",
    id: "lsp-authored-script-diagnostics",
    status: "covered",
  },
  {
    evidence: [
      {
        kind: "file",
        path: "crates/vize_canon/src/sfc_typecheck/tests/fallthrough_ranges.rs",
        requiredText: [
          "fallthrough_diagnostic_range_uses_authored_template_offsets",
          "diagnostic.start > script_start",
        ],
      },
    ],
    followUp: "#4586",
    id: "cli-fallthrough-template-range",
    status: "covered",
  },
  {
    evidence: [
      {
        kind: "file",
        path: "tests/tooling/lsp-fallthrough-attrs.test.ts",
        requiredText: ["publishes and clears fallthrough attribute diagnostics", 'id="outer"'],
      },
    ],
    followUp: "#4586",
    id: "lsp-fallthrough-template-range",
    status: "covered",
  },
  {
    evidence: [
      {
        kind: "file",
        path: "tests/tooling/lsp-hover-type-backed.test.ts",
        requiredText: [
          "hover and definition answer authored template anchors with backend type text",
          "component slot hover must select the authored slot name",
        ],
      },
    ],
    followUp: "#4588 #4592",
    id: "lsp-hover-definition-template-anchors",
    status: "covered",
  },
  {
    evidence: [
      {
        kind: "file",
        path: "tests/snapshots/check/jsx-intrinsic-globals-oracle.ts",
        requiredText: ["JSX.IntrinsicElements", "[TS7026]"],
      },
    ],
    followUp: "#4590",
    id: "cli-jsx-intrinsic-globals",
    status: "covered",
  },
  {
    evidence: [
      {
        kind: "file",
        path: "editors/vscode/test/suite/real-scenario.cjs",
        requiredText: ["refSurfaceHovers", "Ref<unknown>|ComputedRef<unknown>|MaybeRef<unknown>"],
      },
    ],
    followUp: "#4589",
    id: "vscode-host-reactive-hover-surface",
    status: "covered",
  },
  {
    evidence: [
      {
        kind: "pending-pr",
        pr: 4608,
        reason: "LSP imported component hovers need marker-free component contracts.",
      },
    ],
    followUp: "#4591",
    id: "lsp-imported-component-contract-hover",
    status: "known-gap",
  },
  {
    evidence: [
      {
        kind: "pending-pr",
        pr: 4607,
        reason: "Component v-model navigation needs authored defineModel targets.",
      },
    ],
    followUp: "#4592",
    id: "lsp-component-v-model-navigation",
    status: "known-gap",
  },
  {
    evidence: [
      {
        kind: "pending-pr",
        pr: 4609,
        reason: "Non-VSCode host reactive hover coverage is pending merge.",
      },
    ],
    followUp: "#4589",
    id: "non-vscode-host-reactive-hover-surface",
    status: "known-gap",
  },
];

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
      "vscode-host-reactive-hover-surface",
      "lsp-imported-component-contract-hover",
      "lsp-component-v-model-navigation",
      "non-vscode-host-reactive-hover-surface",
    ],
  );

  for (const row of matrix) {
    assert.match(row.followUp, /^#\d+(?:[,\s]+#\d+)*$/, `${row.id} must point at tracking issues`);
    if (row.status === "covered") {
      assert.ok(
        row.evidence.some((entry) => entry.kind === "file"),
        `${row.id} needs file proof`,
      );
    } else {
      assert.ok(
        row.evidence.every((entry) => entry.kind === "pending-pr" && entry.reason.length > 0),
        `${row.id} known gaps need explicit pending PR rationale`,
      );
    }
  }
});

test("covered typed editor oracle matrix rows point at live gate files", () => {
  for (const row of matrix) {
    for (const evidence of row.evidence) {
      if (evidence.kind !== "file") continue;
      const source = fs.readFileSync(path.join(root, evidence.path), "utf8");
      for (const text of evidence.requiredText) {
        assert.match(source, new RegExp(escapeRegExp(text)), `${row.id} missing ${text}`);
      }
    }
  }
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
        if (evidence.kind === "file") {
          assert.match(doc, new RegExp(escapeRegExp(evidence.path)), `${row.id} missing file`);
        }
      }
    }
  }
});

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

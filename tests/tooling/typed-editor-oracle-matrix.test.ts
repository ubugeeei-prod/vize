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
  | { kind: "ci"; label: string; path: string; requiredText: string[] }
  | { kind: "file"; path: string; requiredText: string[] }
  | { kind: "pending-pr"; pr: number; reason: string };

const rowsRequiringExecutedEvidence = new Set([
  "lsp-imported-component-contract-hover",
  "lsp-component-v-model-navigation",
  "non-vscode-host-reactive-hover-surface",
]);

const toolingTestCiEvidence: Evidence = {
  kind: "ci",
  label: "test-scripts runs tests/tooling with VIZE_TEST_REQUIRE_TSGO=1",
  path: "tools/vite-plus/tasks/test-benchmark.ts",
  requiredText: [
    "VIZE_TEST_REQUIRE_TSGO=1 node --test --test-concurrency=1 tests/tooling/*.test.ts",
  ],
};

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
        kind: "file",
        path: "tests/tooling/lsp-imported-component-hover-type-backed.test.ts",
        requiredText: [
          "script hover describes imported SFC contracts instead of generated markers",
          "const Child: VueComponent",
          "__vizeComponentMarker|__vizeRawProps|__VizeComponentConstructor",
        ],
      },
      {
        kind: "file",
        path: "tests/tooling/lsp-imported-component-reexport-hover-type-backed.test.ts",
        requiredText: [
          "script hover describes re-exported and package SFC component contracts",
          "Vue component: PackageChild.vue",
          "__vizeComponentMarker|__vizeRawProps|__VizeComponentConstructor",
        ],
      },
      toolingTestCiEvidence,
    ],
    followUp: "#4591",
    id: "lsp-imported-component-contract-hover",
    status: "covered",
  },
  {
    evidence: [
      {
        kind: "file",
        path: "tests/tooling/lsp-component-v-model-type-backed.test.ts",
        requiredText: [
          "component v-model hover and definition use the child model contract",
          "definition must jump to the child defineModel declaration",
        ],
      },
      toolingTestCiEvidence,
    ],
    followUp: "#4592",
    id: "lsp-component-v-model-navigation",
    status: "covered",
  },
  {
    evidence: [
      {
        kind: "file",
        path: "editors/nvim/test/ref_surface_hover.lua",
        requiredText: [
          "degraded to an unknown reactive type",
          "script ref hover",
          "template template-ref hover",
        ],
      },
      {
        kind: "file",
        path: "editors/nvim/test/vize_e2e_expected.lua",
        requiredText: [
          "ref_surface_hovers",
          "const count: Ref<number, number>",
          "const doubled: ComputedRef<number>",
          "const button: HTMLButtonElement | null",
        ],
      },
      {
        kind: "ci",
        label: "editor-host-smoke runs the packaged Neovim real-server scenario",
        path: ".github/actions/vscode-host-smoke/action.yml",
        requiredText: ["vp run --workspace-root test:nvim-extension:real-server"],
      },
    ],
    followUp: "#4589",
    id: "non-vscode-host-reactive-hover-surface",
    status: "covered",
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

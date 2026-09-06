export type MatrixRow = {
  evidence: Evidence[];
  followUp: string;
  id: string;
  status: "covered" | "known-gap";
};
export type Evidence =
  | { kind: "ci"; label: string; path: string; requiredText: string[] }
  | { kind: "file"; path: string; requiredText: string[] }
  | { kind: "pending-pr"; pr: number; reason: string };
export const rowsRequiringExecutedEvidence = new Set([
  "lsp-authored-script-diagnostics",
  "lsp-fallthrough-template-range",
  "lsp-hover-definition-template-anchors",
  "vscode-host-reactive-hover-surface",
  "vscode-host-component-contract-hover",
  "lsp-imported-component-contract-hover",
  "lsp-jsx-intrinsic-globals",
  "lsp-component-prop-reexport-hover",
  "lsp-component-event-contract-navigation",
  "lsp-component-v-model-navigation",
  "lsp-static-template-ref-navigation",
  "non-vscode-host-reactive-hover-surface",
  "non-vscode-host-component-contract-hover",
]);

const toolingTestCiEvidence: Evidence = {
  kind: "ci",
  label: "test-scripts runs tests/tooling with VIZE_TEST_REQUIRE_TSGO=1",
  path: "tools/config/vite-plus/tasks/test-benchmark.ts",
  requiredText: [
    "VIZE_TEST_REQUIRE_TSGO=1 node --test --test-concurrency=1 tests/tooling/*.test.ts tests/tooling/*.test.mjs",
  ],
};

const vscodeHostCiEvidence: Evidence = {
  kind: "ci",
  label: "editor-host-smoke runs the packaged VS Code host-real scenario",
  path: ".github/actions/vscode-host-smoke/action.yml",
  requiredText: ["vp run --workspace-root test:vscode-extension:host-real"],
};

export const matrix: MatrixRow[] = [
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
      toolingTestCiEvidence,
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
        requiredText: [
          "publishes and clears fallthrough attribute diagnostics",
          'depressed="x"',
          'diagnostic.code === "fallthrough-attrs"',
          "$attrs.class",
          "PlainFragment.vue",
          "SingleRoot.vue",
          "assert.deepEqual(singleRootPublish.diagnostics, [])",
          "line: 5, character: 2",
        ],
      },
      toolingTestCiEvidence,
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
      toolingTestCiEvidence,
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
        path: "tests/tooling/lsp-typecheck-jsx-component.test.ts",
        requiredText: [
          "standalone TSX intrinsic elements stay diagnostic-free in LSP",
          "JSX.IntrinsicElements",
          "[TS7026]",
        ],
      },
      toolingTestCiEvidence,
    ],
    followUp: "#4590",
    id: "lsp-jsx-intrinsic-globals",
    status: "covered",
  },
  {
    evidence: [
      {
        kind: "file",
        path: "editors/vscode/test/suite/real-scenario.cjs",
        requiredText: ["refSurfaceHovers", "Ref<unknown>|ComputedRef<unknown>|MaybeRef<unknown>"],
      },
      vscodeHostCiEvidence,
    ],
    followUp: "#4589",
    id: "vscode-host-reactive-hover-surface",
    status: "covered",
  },
  {
    evidence: [
      {
        kind: "file",
        path: "editors/vscode/test/suite/real-scenario.cjs",
        requiredText: [
          "componentContractHovers",
          "__vizeComponentMarker|__vizeRawProps|__VizeComponentConstructor",
        ],
      },
      {
        kind: "file",
        path: "editors/vscode/test/suite/real-scenario-expected.cjs",
        requiredText: [
          "const ContractChild: VueComponent",
          "props: { label: string; count?: number };",
          "Vue component: ContractChild.vue",
        ],
      },
      vscodeHostCiEvidence,
    ],
    followUp: "#4591",
    id: "vscode-host-component-contract-hover",
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
        path: "tests/tooling/lsp-component-event-hover-type-backed.test.ts",
        requiredText: [
          "component event hovers and definitions use child emit contracts",
          "Component event",
          "[value: string]",
        ],
      },
      toolingTestCiEvidence,
    ],
    followUp: "#4592",
    id: "lsp-component-event-contract-navigation",
    status: "covered",
  },
  {
    evidence: [
      {
        kind: "file",
        path: "tests/tooling/lsp-component-reexport-navigation.test.ts",
        requiredText: [
          "component definition and prop hovers survive re-exported barrel and package boundaries",
          "assertComponentPropHover",
          "Component prop",
          "must not use v-bind fallback",
        ],
      },
      toolingTestCiEvidence,
    ],
    followUp: "#4592",
    id: "lsp-component-prop-reexport-hover",
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
        path: "tests/tooling/lsp-reactive-hover-type-backed.test.ts",
        requiredText: [
          "static template ref value",
          'ref="button"',
          "useTemplateRef<HTMLButtonElement>",
          "definition must jump to the authored useTemplateRef binding",
        ],
      },
      toolingTestCiEvidence,
    ],
    followUp: "#4592",
    id: "lsp-static-template-ref-navigation",
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
  {
    evidence: [
      {
        kind: "file",
        path: "editors/nvim/test/component_contract_hover.lua",
        requiredText: [
          "leaked generated component carrier types",
          "component contract import hover",
          "component contract script usage hover",
        ],
      },
      {
        kind: "file",
        path: "editors/nvim/test/vize_e2e_expected.lua",
        requiredText: [
          "component_contract_hovers",
          "const ContractChild: VueComponent",
          "Vue component: ContractChild.vue",
        ],
      },
      {
        kind: "ci",
        label: "editor-host-smoke runs the packaged Neovim real-server scenario",
        path: ".github/actions/vscode-host-smoke/action.yml",
        requiredText: ["vp run --workspace-root test:nvim-extension:real-server"],
      },
    ],
    followUp: "#4591",
    id: "non-vscode-host-component-contract-hover",
    status: "covered",
  },
];

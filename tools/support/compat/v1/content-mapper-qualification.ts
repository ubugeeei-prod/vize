import { readFileSync } from "node:fs";
import path from "node:path";

export const CONTENT_MAPPER_PROTOCOL_FEATURES = [
  "Hover",
  "SignatureHelp",
  "Completion",
  "Definition",
  "TypeDefinition",
  "Implementation",
  "References",
  "DocumentHighlights",
  "Rename",
  "CallHierarchy",
  "CodeActions",
  "Formatting",
  "InlayHints",
  "SemanticTokens",
  "FoldingRanges",
  "SelectionRanges",
  "LinkedEditing",
  "AutoInsert",
  "DocumentSymbols",
  "CodeLens",
] as const;

const semanticObligations = [
  {
    id: "vue-lines",
    requirement: "Vue 2.7 host and current Vue 3 SFC diagnostics remain exact.",
    issues: [13018],
    evidenceCandidates: [
      ".github/workflows/content-mapper-conformance.yml",
      "crates/vize/tests/check_vue2_vuetify_props_cli.rs",
    ],
  },
  {
    id: "script-setup",
    requirement:
      "Script setup bindings, macros and public local types retain authored diagnostics.",
    issues: [13018],
    evidenceCandidates: [
      "crates/vize/tests/content_mapper_tsgo_cli/props_binding.rs",
      "crates/vize/tests/content_mapper_tsgo_cli/setup_local_public_types.rs",
    ],
  },
  {
    id: "options-api",
    requirement: "Options API instance bindings and diagnostics retain source ownership.",
    issues: [11655, 13018],
    evidenceCandidates: [
      "crates/vize/tests/content_mapper_tsgo_cli/options_api_diagnostics.rs",
      "crates/vize/src/commands/content_mapper/tests.rs",
    ],
  },
  {
    id: "generics",
    requirement: "Generic components preserve declared inference, fallthrough and authored errors.",
    issues: [11655, 13021, 13057],
    evidenceCandidates: [
      "crates/vize/tests/content_mapper_tsgo_cli/reported_generic_components.rs",
      "crates/vize/tests/content_mapper_tsgo_cli/generic_constructor_slots.rs",
    ],
  },
  {
    id: "slots",
    requirement:
      "Slot declarations, destructuring and payload diagnostics are neither hidden nor repeated.",
    issues: [11655, 13018],
    evidenceCandidates: [
      "crates/vize/tests/content_mapper_tsgo_cli/generic_constructor_slots.rs",
      "crates/vize/tests/content_mapper_tsgo_cli/replayed_pattern_duplicates.rs",
    ],
  },
  {
    id: "emits",
    requirement: "Declared, open and dynamic emit payloads retain their exact contracts.",
    issues: [11655, 13018],
    evidenceCandidates: [
      "crates/vize/tests/content_mapper_tsgo_cli/setup_local_public_types.rs",
      "crates/vize/tests/content_mapper_tsgo_cli/reported_model_events_11390_11391.rs",
    ],
  },
  {
    id: "v-model",
    requirement: "Model props and update events retain value and listener diagnostics.",
    issues: [13018],
    evidenceCandidates: [
      "crates/vize/tests/content_mapper_tsgo_cli/template_model_props.rs",
      "crates/vize/tests/content_mapper_tsgo_cli/reported_model_props_11359.rs",
    ],
  },
  {
    id: "diagnostic-directives",
    requirement:
      "Ignore, skip and expect directives suppress only their authored diagnostic owners.",
    issues: [6258, 13057],
    evidenceCandidates: [
      "crates/vize/tests/content_mapper_tsgo_cli/directive_tables.rs",
      "crates/vize/tests/content_mapper_tsgo_directives.rs",
    ],
  },
  {
    id: "css-modules",
    requirement: "CSS module options and template access retain exact diagnostics.",
    issues: [13018],
    evidenceCandidates: [
      "tools/support/compat/content-mapper/official-css-options.ts",
      "tests/tooling/canon-css-module-contracts.test.ts",
    ],
  },
  {
    id: "native-elements-events",
    requirement: "Native attributes and events remain distinct from component props and emits.",
    issues: [11655, 13018],
    evidenceCandidates: [
      ".github/workflows/public-native-events.yml",
      "crates/vize/tests/content_mapper_tsgo_cli/shared_helpers_performance/open_component_event_contracts.rs",
    ],
  },
  {
    id: "aliases",
    requirement: "Path, import and template aliases resolve through the opened project.",
    issues: [12102, 13018],
    evidenceCandidates: [
      ".github/workflows/template-alias-paired-11822.yml",
      "crates/vize/tests/check_default_imports_cli/aliased_diagnostics.rs",
    ],
  },
  {
    id: "package-exports",
    requirement: "Imported SFC packages and declarations preserve importer-scoped resolution.",
    issues: [12102, 13018],
    evidenceCandidates: [
      "crates/vize/tests/content_mapper_tsgo_cli/shipped_declarations_12968.rs",
      ".github/workflows/content-mapper-conformance.yml",
    ],
  },
  {
    id: "project-references",
    requirement: "Referenced projects are checked once without dropping cross-project diagnostics.",
    issues: [12102, 12109],
    evidenceCandidates: [
      "crates/vize/tests/content_mapper_tsgo_package_references.rs",
      "crates/vize/src/commands/check/tsconfig_inputs/tests/reference_programs.rs",
    ],
  },
  {
    id: "nuxt-generated-types",
    requirement: "Nuxt-generated aliases and ambient types participate in the mapper project.",
    issues: [11104, 13018],
    evidenceCandidates: [
      ".github/workflows/nuxt-generated-membership.yml",
      "crates/vize/tests/check_canon_nuxt_parity_cli.rs",
    ],
  },
  {
    id: "independent-project-diagnostics",
    requirement: "An SFC transform failure never hides diagnostics from another project file.",
    issues: [13018],
    evidenceCandidates: [
      "crates/vize/tests/content_mapper_tsgo_cli/semantic_compile_errors_12522.rs",
      "crates/vize/tests/content_mapper_tsgo_cli/incomplete_tags.rs",
    ],
  },
] as const;

const diagnosticObligations = [
  {
    id: "authored-packets-and-spans",
    requirement:
      "Codes, categories, messages, files, authored spans, ordering and exit status are exact.",
    issues: [11655, 13037],
    evidenceCandidates: [
      "tests/tooling/content-mapper-replay.test.ts",
      "crates/vize/tests/content_mapper_tsgo_cli/unicode_prop_bindings.rs",
    ],
  },
  {
    id: "related-information",
    requirement:
      "Related information retains its authored file and range without generated leakage.",
    issues: [13018, 13037],
    evidenceCandidates: [
      ".github/workflows/content-mapper-conformance.yml",
      "tests/tooling/content-mapper-admission-hosts.test.ts",
    ],
  },
  {
    id: "suppression-ownership",
    requirement:
      "Suppression is selected by authored ownership and never widens to a generated range.",
    issues: [6258, 13021, 13057],
    evidenceCandidates: [
      "crates/vize_canon/src/batch/virtual_project/content_mapper/directives.rs",
      "crates/vize/src/commands/content_mapper/dispatch/wire.rs",
    ],
  },
  {
    id: "generated-file-filtering",
    requirement: "Generated helpers and virtual files never replace source diagnostics.",
    issues: [13018, 13037],
    evidenceCandidates: [
      "tests/tooling/content-mapper-instance-globals-packet-authority.test.ts",
      "crates/vize/tests/content_mapper_tsgo_cli/semantic_compile_errors_12522.rs",
    ],
  },
] as const;

const boundaryExamples = [
  {
    id: "vue-sfc-jsx-tsx",
    disposition: "required-v1",
    reason: "JSX and TSX embedded in a Vue SFC are part of the v1 Vue typechecker surface.",
    evidenceCandidates: ["crates/vize/tests/content_mapper_tsgo_cli/tsx_script_syntax.rs"],
  },
  {
    id: "tsrx",
    disposition: "outside-v1",
    reason: "TSRX is a pinned differential oracle, not a Content Mapper input claim.",
    evidenceCandidates: ["tests/tooling/tsrx-oracle.test.ts"],
  },
  {
    id: "svelte",
    disposition: "outside-v1",
    reason:
      "The Svelte proof of concept is compiler boundary evidence and remains unsupported input.",
    evidenceCandidates: ["examples/poc-svelte/README.md"],
  },
  {
    id: "astro",
    disposition: "outside-v1",
    reason:
      "The Astro proof of concept is compiler boundary evidence and remains unsupported input.",
    evidenceCandidates: ["examples/poc-astro/README.md"],
  },
] as const;

function pinnedTypeScriptRevision(root: string): string {
  const workflow = readFileSync(
    path.join(root, ".github/workflows/content-mapper-conformance.yml"),
    "utf8",
  );
  const revision = /CONTENT_MAPPER_TYPESCRIPT_SHA:\s*"([0-9a-f]{40})"/u.exec(workflow)?.[1];
  if (!revision) throw new Error("content-mapper conformance must pin one TypeScript revision");
  return revision;
}

function unbound<T extends object>(obligation: T, requiredCases: readonly string[]) {
  return {
    ...obligation,
    requiredCases: [...requiredCases],
    candidateSource: null,
    candidateRun: null,
    receipts: [] as string[],
    qualification: "unbound",
  };
}

export function buildContentMapperV1Qualification(root: string, requiredCases: readonly string[]) {
  const protocolFeatures = CONTENT_MAPPER_PROTOCOL_FEATURES.map((feature) =>
    unbound(
      {
        id: feature,
        issues: [13018],
        evidenceCandidates: [
          "crates/vize_canon/src/batch/virtual_project/content_mapper/span_features.rs",
          "crates/vize_canon/src/batch/virtual_project/content_mapper/tests/protocol.rs",
        ],
      },
      requiredCases,
    ),
  );
  const semantic = semanticObligations.map((obligation) => unbound(obligation, requiredCases));
  const diagnostics = diagnosticObligations.map((obligation) => unbound(obligation, requiredCases));
  const lifecycle = [
    unbound(
      {
        id: "monotonic-project-revision",
        operations: ["open", "edit", "save", "close", "create", "rename", "delete"],
        requirement: "Every file and overlay transition belongs to one monotonic project revision.",
        issues: [12102, 13018],
        evidenceCandidates: [
          "crates/vize/src/commands/content_mapper/project.rs",
          "tests/tooling/real-project-lsp-file-lifecycle.test.ts",
        ],
      },
      requiredCases,
    ),
  ];
  const performance = [
    {
      id: "cold-warm-edit",
      measurements: ["cold", "warm", "no-op", "leaf-edit", "dependency-edit", "p95", "rss"],
      scales: [1_000, 5_000, 20_000],
      issues: [10620, 12102, 12109],
      evidenceCandidates: [
        ".github/workflows/content-mapper-replay.yml",
        ".github/workflows/content-mapper-cold-start.yml",
      ],
      packetIdentity: "required",
      baseHeadComparison: "required",
      candidateSource: null,
      candidateRun: null,
      receipts: [] as string[],
      qualification: "unbound",
    },
  ];
  const runtimeAndCorpus = [
    unbound(
      {
        id: "node-bun-hosts",
        issues: [13038],
        evidenceCandidates: [
          ".github/workflows/pr-js-checks.yml",
          "npm/cli/tests/content-mapper-bun.test.ts",
        ],
      },
      requiredCases,
    ),
    unbound(
      {
        id: "real-projects",
        issues: [13044],
        evidenceCandidates: [
          ".github/workflows/real-project-matrix.yml",
          ".github/workflows/real-project-matrix-differential.yml",
        ],
      },
      requiredCases,
    ),
  ];
  const obligations = [
    ...semantic,
    ...protocolFeatures,
    ...diagnostics,
    ...lifecycle,
    ...performance,
    ...runtimeAndCorpus,
  ];

  return {
    schema: "vize.content-mapper-v1-qualification.v1",
    parentIssue: 13018,
    qualificationIssue: 13071,
    qualification: obligations.every(({ qualification }) => qualification === "complete")
      ? "complete"
      : "gap",
    pinnedTypeScriptRevision: pinnedTypeScriptRevision(root),
    producerProtocol: {
      diagnosticCodeSelectors: "negotiated",
      legacyPayload: "byte-identical-without-capability",
      evidenceCandidates: [
        "crates/vize_canon/src/batch/virtual_project/content_mapper/protocol.rs",
        "crates/vize/src/commands/content_mapper/dispatch/wire.rs",
      ],
    },
    upstreamBlockers: [
      {
        id: "grouped-disjoint-expect-usage",
        issue: 6258,
        state: "blocked-upstream",
      },
      {
        id: "diagnostic-code-selector-consumer",
        issue: 13057,
        external: "microsoft/TypeScript#65517",
        state: "blocked-upstream",
      },
    ],
    semantic,
    protocolFeatures,
    diagnostics,
    lifecycle,
    performance,
    runtimeAndCorpus,
    boundaryExamples,
    counts: {
      semantic: semantic.length,
      protocolFeatures: protocolFeatures.length,
      diagnostics: diagnostics.length,
      lifecycle: lifecycle.length,
      performance: performance.length,
      runtimeAndCorpus: runtimeAndCorpus.length,
      unbound: obligations.filter(({ qualification }) => qualification !== "complete").length,
    },
  };
}

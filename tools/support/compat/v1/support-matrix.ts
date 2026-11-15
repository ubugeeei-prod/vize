import { readFileSync } from "node:fs";
import path from "node:path";

import { PACKAGE_MANAGERS } from "../npm/smoke-release-init-managers.ts";
import { FRESH_INIT_MATRIX } from "../npm/smoke-release-init-shapes.ts";
import { editorReleasePlatforms } from "../../release/editor-platforms.ts";
import { cliReleasePlatforms, nativeReleasePlatforms } from "../../github/release-platforms.ts";
import { buildContentMapperV1Qualification } from "./content-mapper-qualification.ts";

export const V1_CASES = ["positive", "negative", "boundary", "malformed-edit", "repair"] as const;
export const V1_TOPOLOGIES = [
  "monorepo",
  "project-reference",
  "alias-package-export",
  "unicode-space",
  "crlf",
  "symlink",
] as const;

type V1Case = (typeof V1_CASES)[number];
type Topology = (typeof V1_TOPOLOGIES)[number];
type Profile = {
  id: string;
  owner: string;
  surface: "compiler" | "integration" | "linter" | "typechecker" | "lsp-editor";
  product: string;
  vue: "2.7" | "3.x" | "3.6+-vapor";
  backend: "host-vdom" | "vdom" | "vapor" | "analysis";
  sourceKinds: readonly string[];
  topologies: readonly Topology[];
  requiredQualificationSets: readonly string[];
  evidence: readonly string[];
  condition?: string;
};

const lintSourceKinds = [
  "vue",
  "html",
  "htm",
  "js",
  "mjs",
  "cjs",
  "ts",
  "mts",
  "cts",
  "jsx",
  "tsx",
] as const;
const compiledSourceKinds = ["sfc-js", "sfc-ts", "sfc-jsx", "sfc-tsx", "jsx", "tsx"] as const;
const sfcSourceKinds = ["sfc-js", "sfc-ts"] as const;
const typecheckSourceKinds = ["vue", "ts", "tsx", "d-ts"] as const;
const conditionalTypecheckSourceKinds = ["js-allow-js", "jsx-allow-js-jsx-typecheck"] as const;
const lspSourceKinds = ["vue", "art-vue", "js", "ts", "jsx", "tsx"] as const;
const commonEditorSourceKinds = ["vue", "art-vue"] as const;
const fileTopologies = ["unicode-space", "crlf", "symlink"] as const;
const projectTopologies = [...V1_TOPOLOGIES] as const;

const profiles: readonly Profile[] = [
  {
    id: "compiler-vdom-vue3",
    owner: "compiler/vize_atelier_dom",
    surface: "compiler",
    product: "native-compiler",
    vue: "3.x",
    backend: "vdom",
    sourceKinds: ["sfc-js", "sfc-ts", "sfc-jsx", "sfc-tsx"],
    topologies: fileTopologies,
    requiredQualificationSets: ["native-node-bun"],
    evidence: [
      ".github/actions/check-vue-parity/action.yml",
      "tests/tooling/differential-compiler.test.mjs",
      "tests/tooling/differential-compiler-api.test.ts",
    ],
  },
  {
    id: "compiler-vapor-vue36",
    owner: "compiler/vize_atelier_vapor",
    surface: "compiler",
    product: "native-compiler",
    vue: "3.6+-vapor",
    backend: "vapor",
    sourceKinds: ["sfc-js", "sfc-ts", "sfc-jsx", "sfc-tsx"],
    topologies: fileTopologies,
    requiredQualificationSets: ["native-node-bun"],
    condition: "Current public tier is experimental; the exact Vue 3.6 baseline is unresolved.",
    evidence: [
      ".github/actions/test-native-vapor/action.yml",
      "tests/tooling/native-vapor-reference.test.mjs",
      "crates/vize_atelier_jsx/tests/vue_jsx_vapor_compat_oracle.rs",
    ],
  },
  {
    id: "vite-host-vdom-vue27",
    owner: "integrations/@vizejs/vite-plugin",
    surface: "integration",
    product: "vite",
    vue: "2.7",
    backend: "host-vdom",
    sourceKinds: sfcSourceKinds,
    topologies: projectTopologies,
    requiredQualificationSets: ["native-node-bun", "fresh-package-managers"],
    condition: "Vize delegates SFC compilation to the installed Vue 2.7 host compiler.",
    evidence: [
      "npm/builder/vite/src/plugin/vue-version.ts",
      "tests/tooling/vue-config-fixture.test.ts",
      ".github/workflows/native-smoke.yml",
    ],
  },
  {
    id: "vite-vdom-vue3",
    owner: "integrations/@vizejs/vite-plugin",
    surface: "integration",
    product: "vite",
    vue: "3.x",
    backend: "vdom",
    sourceKinds: compiledSourceKinds,
    topologies: projectTopologies,
    requiredQualificationSets: ["native-node-bun", "fresh-package-managers"],
    evidence: [
      ".github/workflows/native-smoke.yml",
      ".github/workflows/vite-plus-type-compat.yml",
      "tests/tooling/release/release-smoke-init-fresh.test.ts",
    ],
  },
  {
    id: "vite-vapor-vue36",
    owner: "integrations/@vizejs/vite-plugin",
    surface: "integration",
    product: "vite",
    vue: "3.6+-vapor",
    backend: "vapor",
    sourceKinds: compiledSourceKinds,
    topologies: projectTopologies,
    requiredQualificationSets: ["native-node-bun", "fresh-package-managers"],
    condition: "Current public tier is experimental; the exact Vue 3.6 baseline is unresolved.",
    evidence: [
      ".github/workflows/native-smoke.yml",
      "npm/builder/vite/src/markdown-runtime.test.ts",
      "tests/tooling/native-vapor-reference.test.mjs",
    ],
  },
  {
    id: "vite-vdom-vue3-explicit-jsx",
    owner: "integrations/@vizejs/vite-plugin",
    surface: "integration",
    product: "vite",
    vue: "3.x",
    backend: "vdom",
    sourceKinds: ["jsx-explicit-include", "tsx-explicit-include"],
    topologies: projectTopologies,
    requiredQualificationSets: ["native-node-bun", "fresh-package-managers"],
    condition: "Standalone JSX and TSX compilation requires an explicit include match.",
    evidence: [
      "docs/content/guide/jsx.md",
      "npm/builder/vite/src/plugin/load.ts",
      "npm/builder/vite/src/plugin/load-jsx-passthrough.test.ts",
    ],
  },
  {
    id: "vite-vapor-vue36-explicit-jsx",
    owner: "integrations/@vizejs/vite-plugin",
    surface: "integration",
    product: "vite",
    vue: "3.6+-vapor",
    backend: "vapor",
    sourceKinds: ["jsx-explicit-include", "tsx-explicit-include"],
    topologies: projectTopologies,
    requiredQualificationSets: ["native-node-bun", "fresh-package-managers"],
    condition: "Explicit include is required and module-level Vapor parity remains incomplete.",
    evidence: [
      "docs/content/guide/vapor.md",
      "npm/builder/vite/src/plugin/load.ts",
      "tests/tooling/native-vapor-reference.test.mjs",
    ],
  },
  {
    id: "nuxt-host-vdom-vue27",
    owner: "integrations/@vizejs/nuxt",
    surface: "integration",
    product: "nuxt",
    vue: "2.7",
    backend: "host-vdom",
    sourceKinds: sfcSourceKinds,
    topologies: projectTopologies,
    requiredQualificationSets: ["native-node-bun", "fresh-package-managers"],
    condition: "Vize delegates SFC compilation to the installed Vue 2.7 host compiler.",
    evidence: [
      ".github/workflows/nuxt2-module-build.yml",
      ".github/workflows/nuxt2-bridge-module-build.yml",
      "npm/framework/nuxt/src/compiler-option-bridge.ts",
    ],
  },
  {
    id: "nuxt-vdom-vue3",
    owner: "integrations/@vizejs/nuxt",
    surface: "integration",
    product: "nuxt",
    vue: "3.x",
    backend: "vdom",
    sourceKinds: sfcSourceKinds,
    topologies: projectTopologies,
    requiredQualificationSets: ["native-node-bun", "fresh-package-managers"],
    evidence: [
      ".github/workflows/nuxt3-module-build.yml",
      ".github/workflows/native-smoke.yml",
      "tests/tooling/release/release-smoke-init-nuxt.test.ts",
    ],
  },
  {
    id: "nuxt-vapor-vue36",
    owner: "integrations/@vizejs/nuxt",
    surface: "integration",
    product: "nuxt",
    vue: "3.6+-vapor",
    backend: "vapor",
    sourceKinds: sfcSourceKinds,
    topologies: projectTopologies,
    requiredQualificationSets: ["native-node-bun", "fresh-package-managers"],
    condition: "Current public tier is experimental; the exact Vue 3.6 baseline is unresolved.",
    evidence: [
      "npm/framework/nuxt/src/vapor-vue.ts",
      "npm/framework/nuxt/src/vapor-vue.test.ts",
      ".github/workflows/nuxt3-module-build.yml",
    ],
  },
  ...(["2.7", "3.x"] as const).flatMap((vue) => [
    {
      id: `native-linter-vue${vue.replaceAll(".", "")}`,
      owner: "linter/vize_patina",
      surface: "linter" as const,
      product: "native-linter",
      vue,
      backend: "analysis" as const,
      sourceKinds: lintSourceKinds,
      topologies: projectTopologies,
      requiredQualificationSets: ["native-node-bun", "fresh-package-managers"],
      condition:
        "Rule applicability is partitioned; this is not an all-rules by all-languages claim.",
      evidence: [
        ".github/workflows/lint-reference-coverage-contracts.yml",
        ".github/workflows/real-project-matrix.yml",
        "tests/tooling/differential-linter-api.test.ts",
      ],
    },
    {
      id: `content-mapper-vue${vue.replaceAll(".", "")}`,
      owner: "typechecker/vize_canon-content-mapper",
      surface: "typechecker" as const,
      product: "content-mapper",
      vue,
      backend: "analysis" as const,
      sourceKinds: ["vue-content-mapper"],
      topologies: projectTopologies,
      requiredQualificationSets: ["native-node-bun", "fresh-package-managers", "wsl2-node"],
      condition: "Content Mapper consumes .vue files; plain JS and TS are TypeScript host inputs.",
      evidence: [
        ".github/workflows/content-mapper-conformance.yml",
        ".github/workflows/typecheck-wsl2.yml",
        "tests/tooling/content-mapper-workflow.test.ts",
      ],
    },
    {
      id: `vize-check-vue${vue.replaceAll(".", "")}`,
      owner: "typechecker/vize_canon-check",
      surface: "typechecker" as const,
      product: "vize-check",
      vue,
      backend: "analysis" as const,
      sourceKinds: typecheckSourceKinds,
      topologies: projectTopologies,
      requiredQualificationSets: ["native-node-bun", "fresh-package-managers", "wsl2-node"],
      evidence: [
        ".github/workflows/typecheck-wsl2.yml",
        "docs/content/guide/type-checking.md",
        "tests/tooling/typecheck-baseline-outside-jsx.test.ts",
      ],
    },
    {
      id: `vize-check-conditional-js-vue${vue.replaceAll(".", "")}`,
      owner: "typechecker/vize_canon-check",
      surface: "typechecker" as const,
      product: "vize-check",
      vue,
      backend: "analysis" as const,
      sourceKinds: conditionalTypecheckSourceKinds,
      topologies: projectTopologies,
      requiredQualificationSets: ["native-node-bun", "fresh-package-managers", "wsl2-node"],
      condition: "Plain JS requires allowJs/checkJs; JSX additionally requires jsxTypecheck.",
      evidence: [
        "crates/vize/src/commands/check/tsconfig_inputs/tests/allow_js.rs",
        "docs/content/guide/type-checking.md",
        "tests/tooling/typecheck-baseline-outside-jsx.test.ts",
      ],
    },
    {
      id: `lsp-server-vue${vue.replaceAll(".", "")}`,
      owner: "lsp/vize_maestro",
      surface: "lsp-editor" as const,
      product: "lsp-server",
      vue,
      backend: "analysis" as const,
      sourceKinds: lspSourceKinds,
      topologies: projectTopologies,
      requiredQualificationSets: ["native-node-bun", "wsl2-node"],
      condition: "Core server recognition is broader than the shared shipped-editor surface.",
      evidence: [
        ".github/workflows/lsp-current-baseline.yml",
        ".github/workflows/typecheck-wsl2.yml",
        "tests/tooling/lsp-authored-script-diagnostics.test.ts",
      ],
    },
    {
      id: `shipped-editors-vue${vue.replaceAll(".", "")}`,
      owner: "editors/vscode-zed-nvim-vim-helix-emacs",
      surface: "lsp-editor" as const,
      product: "shipped-editors",
      vue,
      backend: "analysis" as const,
      sourceKinds: commonEditorSourceKinds,
      topologies: projectTopologies,
      requiredQualificationSets: ["native-node-bun", "shipped-editors"],
      condition:
        "The common editor surface is .vue/.art.vue and editor scenarios have different depth.",
      evidence: [
        ".github/workflows/editor-conformance.yml",
        ".github/workflows/release-zed-host.yml",
        "tests/tooling/editor-conformance.test.ts",
      ],
    },
    {
      id: `vscode-jsx-vue${vue.replaceAll(".", "")}`,
      owner: "editors/vscode",
      surface: "lsp-editor" as const,
      product: "vscode-jsx-tsx",
      vue,
      backend: "analysis" as const,
      sourceKinds: ["jsx-jsx-typecheck", "tsx-jsx-typecheck"],
      topologies: projectTopologies,
      requiredQualificationSets: ["native-node-bun", "shipped-editors"],
      condition: "Standalone JSX and TSX editor routing is VS Code-only and requires jsxTypecheck.",
      evidence: [
        "editors/vscode/src/extension-core.ts",
        "editors/vscode/src/jsx-routing.ts",
        "tests/tooling/vscode-extension-core-behavior.test.ts",
      ],
    },
  ]),
];

function manifest(pathname: string): Record<string, unknown> {
  return JSON.parse(readFileSync(pathname, "utf8")) as Record<string, unknown>;
}

function bunVersion(root: string): string {
  const source = readFileSync(
    path.join(root, ".github/actions/setup-runtime-package-managers/action.yml"),
    "utf8",
  );
  const version = /bun-version:\s*"([^"]+)"/u.exec(source)?.[1];
  if (!version) throw new Error("setup-runtime-package-managers must pin Bun");
  return version;
}

function currentTier(profile: Profile): string {
  if (profile.backend === "vapor") return "experimental";
  if (profile.product === "nuxt") return "compatibility-preview";
  if (profile.product === "shipped-editors") return "incubating";
  if (profile.product === "lsp-server" || profile.product === "vscode-jsx-tsx")
    return "experimental";
  if (profile.product === "native-linter" || profile.product === "content-mapper")
    return "mixed-cli-alpha-crate-preview";
  return "alpha-supported";
}

function qualifiedBaseline(profile: Profile): string {
  if (profile.vue === "2.7") return "vue@2.7.16-host-compiler";
  if (profile.vue === "3.6+-vapor") return "unresolved-vue-3.6-prerelease";
  return "shipping-vue@3.5.42-differential-vue@3.5.35";
}

export function buildV1SupportMatrix(root = process.cwd()) {
  const nativeManifest = manifest(path.join(root, "npm/native/package.json"));
  const cliManifest = manifest(path.join(root, "npm/cli/package.json"));
  const nativeTargets = (
    (nativeManifest.napi as { targets?: string[] } | undefined)?.targets ?? []
  ).toSorted();
  const releasedNativeTargets = nativeReleasePlatforms.map(({ target }) => target).toSorted();
  const releasedCliTargets = cliReleasePlatforms.map(({ target }) => target).toSorted();
  const vscodeProviderTargets = editorReleasePlatforms
    .map(({ rustTarget }) => rustTarget)
    .toSorted();
  const nodeEngine = (cliManifest.engines as { node?: string } | undefined)?.node;
  const bun = bunVersion(root);
  const combinations = profiles
    .flatMap((profile) =>
      profile.sourceKinds.map((sourceKind) => ({
        id: `${profile.id}/${sourceKind}`,
        owner: profile.owner,
        surface: profile.surface,
        product: profile.product,
        vue: profile.vue,
        backend: profile.backend,
        sourceKind,
        requiredCases: [...V1_CASES] as V1Case[],
        topologies: [...profile.topologies],
        requiredQualificationSets: [...profile.requiredQualificationSets],
        evidence: [...profile.evidence].toSorted(),
        condition: profile.condition ?? null,
        currentTier: currentTier(profile),
        v1Target: "required",
        qualifiedBaseline: qualifiedBaseline(profile),
        qualification: "unbound",
      })),
    )
    .toSorted((left, right) => left.id.localeCompare(right.id));

  const nodeHosts = nativeTargets.flatMap((target) =>
    (target.endsWith("-musl") ? ["24"] : ["22", "24"]).map((version) => ({
      runtime: "node",
      version,
      target,
    })),
  );
  const bunHosts = nativeTargets
    .filter((target) => !target.endsWith("-musl"))
    .map((target) => ({ runtime: "bun", version: bun, target }));
  const packageManagers = Object.values(PACKAGE_MANAGERS)
    .map(({ id, binary, corepackSpec, lockfile }) => ({
      id,
      binary,
      qualifiedVersion:
        corepackSpec ??
        (id === "bun"
          ? `bun@${bun}`
          : id === "npm"
            ? "bundled-with-qualified-node"
            : "setup-vp-v1-action"),
      lockfile,
    }))
    .toSorted((left, right) => left.id.localeCompare(right.id));
  const freshProjects = FRESH_INIT_MATRIX.map((cell) => ({ ...cell })).toSorted((left, right) =>
    `${left.packageManager}/${left.shape}`.localeCompare(`${right.packageManager}/${right.shape}`),
  );
  const editors = [
    {
      id: "emacs",
      advertisedBaseline: "Emacs 29.3 / Eglot 1.12.29",
      testedBaseline: "ubuntu-runner emacs-nox",
      hostOs: "linux",
      lane: "real-project-configuration",
    },
    {
      id: "helix",
      advertisedBaseline: "Helix integration",
      testedBaseline: "Helix 25.07.1",
      hostOs: "linux",
      lane: "real-ts45",
    },
    {
      id: "neovim",
      advertisedBaseline: "Neovim 0.11+",
      testedBaseline: "Neovim 0.11.4",
      hostOs: "linux",
      lane: "real-ts45",
    },
    {
      id: "vim",
      advertisedBaseline: "Vim with vim-lsp",
      testedBaseline: "ubuntu-runner Vim / vim-lsp 474c656659b5fb51eec6770309e0211c8aa49b5b",
      hostOs: "linux",
      lane: "real-project-configuration",
    },
    {
      id: "vscode",
      advertisedBaseline: "VS Code 1.75+",
      testedBaseline: "VS Code 1.107.1 / languageclient 9.0.1",
      hostOs: "linux",
      lane: "real-ts45-and-extension-host",
    },
    {
      id: "zed",
      advertisedBaseline: "Zed extension current",
      testedBaseline: "Zed 1.27.0 GUI host plus protocol TS-45",
      hostOs: "linux",
      lane: "real-release-host-and-protocol-replay",
    },
  ];
  const hostRuntimeCombinations = [
    ...nodeHosts,
    ...bunHosts,
    {
      runtime: "node",
      version: "24.14.0",
      target: "wsl2-ubuntu-24.04-x64",
      filesystems: ["ext4", "windows-mount"],
      observedSurfaces: ["content-mapper", "lsp-server"],
    },
  ];
  const evidenceSnapshots = [
    {
      id: "release-v0.755.0",
      classification: "historical-manual-context",
      source: "26d0c1ea5e758cc54397264031ca15d4bc9e46cc",
      run: 43678823127,
      workflow: "release.yml",
      conclusion: "success",
      coverage: "8 CLI, 8 native and 8 VS Code provider builds, packed smoke and publication",
      url: "https://github.com/ubugeeei-prod/vize/actions/runs/43678823127",
    },
    {
      id: "public-v0.755.0",
      classification: "historical-manual-context",
      source: "26d0c1ea5e758cc54397264031ca15d4bc9e46cc",
      run: 43680943221,
      workflow: "release-verify.yml",
      conclusion: "success",
      coverage: "6 GNU, macOS and Windows plus 2 musl Node 24 fresh consumers",
      url: "https://github.com/ubugeeei-prod/vize/actions/runs/43680943221",
    },
    {
      id: "jsr-v0.755.0",
      classification: "historical-manual-context",
      source: "26d0c1ea5e758cc54397264031ca15d4bc9e46cc",
      run: 43681461563,
      workflow: "release-jsr.yml",
      conclusion: "success",
      coverage: "Node 22/24, macOS, Windows and Bun 1.3.14 JSR consumers",
      url: "https://github.com/ubugeeei-prod/vize/actions/runs/43681461563",
    },
    {
      id: "native-smoke-8003302",
      classification: "historical-manual-context",
      source: "800330221e8da576c20b35a01e684388e0af70dc",
      run: 42639912076,
      workflow: "native-smoke.yml",
      conclusion: "success",
      coverage: "6 non-musl Node 22/24 hosts and 2 musl Node 24 hosts",
      url: "https://github.com/ubugeeei-prod/vize/actions/runs/42639912076",
    },
    {
      id: "editor-conformance-8f52ede",
      classification: "historical-manual-context",
      source: "8f52ede15970721062325adbba6d0dc4d1e722cc",
      run: 43712760021,
      workflow: "editor-conformance.yml",
      conclusion: "success",
      coverage: "4 TS-45 clients plus Vim and Emacs configuration-only lanes",
      url: "https://github.com/ubugeeei-prod/vize/actions/runs/43712760021",
    },
    {
      id: "wsl2-a0d1447",
      classification: "historical-manual-context",
      source: "a0d1447243e6180fa4fb7c7230d69f7b1c0f0449",
      run: 42370439979,
      workflow: "typecheck-wsl2.yml",
      conclusion: "success",
      coverage: "published vize 0.711.0 benchmark on WSL2 ext4 and Windows mount",
      url: "https://github.com/ubugeeei-prod/vize/actions/runs/42370439979",
    },
    {
      id: "vscode-wsl-dfdcb59",
      classification: "historical-manual-context",
      source: "dfdcb5961c31cae74b6aaac4bc2e94fc53d59b0f",
      run: 42586612047,
      workflow: "vscode-wsl-startup.yml",
      conclusion: "success",
      coverage: "VS Code WSL startup and server selection contract",
      url: "https://github.com/ubugeeei-prod/vize/actions/runs/42586612047",
    },
    {
      id: "bun-products-481d3e7",
      classification: "historical-manual-context",
      source: "481d3e7298037259a887075426e7ee630e0ddedd",
      run: 43676047430,
      workflow: "release-smoke-contracts.yml",
      conclusion: "success",
      coverage: "Bun 1.3.14 packed product parity and Linux, macOS and Windows contracts",
      url: "https://github.com/ubugeeei-prod/vize/actions/runs/43676047430",
    },
  ];
  const productProfiles = [
    ...new Map(
      profiles.map((profile) => [
        `${profile.product}/${profile.vue}/${profile.backend}`,
        {
          id: `${profile.product}/${profile.vue}/${profile.backend}`,
          owner: profile.owner,
          product: profile.product,
          vue: profile.vue,
          backend: profile.backend,
          supported: profile.condition ? "conditional" : "yes",
          currentTier: currentTier(profile),
          qualifiedBaseline: qualifiedBaseline(profile),
        },
      ]),
    ).values(),
  ].toSorted((left, right) => left.id.localeCompare(right.id));
  const requiredRows = productProfiles
    .flatMap((product) =>
      hostRuntimeCombinations.map((host) => ({
        id: `${product.id}/${host.runtime}-${host.version}/${host.target}`,
        owner: product.owner,
        product: product.product,
        vue: product.vue,
        backend: product.backend,
        host: host.target,
        runtime: host.runtime,
        runtimeVersion: host.version,
        supported: product.supported,
        currentTier: product.currentTier,
        v1Target: "required",
        qualifiedBaseline: product.qualifiedBaseline,
        candidateSource: null,
        candidateRun: null,
        evidence: [] as string[],
        positive: [] as string[],
        negative: [] as string[],
        manual: [] as string[],
        qualification: "gap",
        gap: "No exact-candidate receipt joins this product, Vue line, host and runtime to all required cases.",
      })),
    )
    .toSorted((left, right) => left.id.localeCompare(right.id));
  const knownGaps = [
    {
      id: "content-mapper-v1-qualification",
      owner: "typechecker/vize_canon-content-mapper",
      issues: [13018, 13071],
      state: "blocking",
      summary:
        "Content Mapper semantic, protocol, lifecycle, performance and real-project obligations are not joined to one exact candidate.",
      exitCondition:
        "Every contentMapper obligation has exact positive, negative, boundary, malformed-edit and repair receipts for one candidate source.",
    },
    {
      id: "case-packet-join",
      owner: "testing/v1-qualification",
      issues: [13037],
      state: "blocking",
      summary:
        "Five case names are not joined to exact diagnostics, ranges, edits, output, ordering and exit packets.",
      exitCondition:
        "Every source combination has executable positive, negative, boundary, malformed-edit and repair selectors with receipt identity.",
    },
    {
      id: "host-runtime-join",
      owner: "release/v1-qualification",
      issues: [13038, 13012],
      state: "blocking",
      summary:
        "No single exact candidate has native, Bun, WSL2 and editor evidence for every required product row.",
      exitCondition: "One candidate SHA produces admitted receipts for every requiredRows entry.",
    },
    {
      id: "linter-rule-partition",
      owner: "linter/vize_patina",
      issues: [6097],
      state: "blocking",
      summary:
        "The all-rule registry is not exact-partitioned by language, fix ownership and five-case evidence.",
      exitCondition:
        "The generated manifest consumes the complete Patina registry and rejects missing or duplicate rule obligations.",
    },
    {
      id: "vue-baseline-and-maturity",
      owner: "release/channels",
      issues: [13016],
      state: "blocking",
      summary: "Vue 3.6 pins and current stability tiers do not yet match the v1 target channel.",
      exitCondition:
        "One canonical Vue baseline and #13016 channel metadata agree across package peers, docs and release workflows.",
    },
    {
      id: "editor-depth",
      owner: "lsp/editors",
      issues: [3952, 3957],
      state: "blocking",
      summary:
        "Four editors run TS-45 while Vim and Emacs run configuration-only lanes; WSL2 is not full editor conformance.",
      exitCondition:
        "Each advertised editor has an explicitly scoped required scenario and exact candidate receipt without order normalization.",
    },
  ];
  const contentMapper = buildContentMapperV1Qualification(root, V1_CASES);
  const releaseReady =
    requiredRows.every(({ qualification }) => qualification === "complete") &&
    knownGaps.every(({ state }) => state !== "blocking") &&
    contentMapper.qualification === "complete";

  return {
    schema: "vize.v1-qualification-manifest.v1",
    generatedBy: "node tools/commands/release/v1-support-matrix.ts",
    releaseReady,
    coverageSemantics: {
      combinations: "Declared source support and conditions, not execution credit.",
      requiredRows: "Required product by Vue by host by runtime rows. A gap is release-blocking.",
      evidenceSnapshots:
        "Historical or manual context only until joined to one exact candidate row.",
      positiveNegativeManual: "Only exact admitted receipt IDs may appear in these row columns.",
    },
    scope: {
      included: ["compiler", "nuxt-vite", "native-linter", "content-mapper", "lsp-editors"],
      excluded: [
        "davinci-v3",
        "eslint",
        "prettier",
        "babel",
        "formatter-v0",
        "musea-v0",
        "ui-early",
        "fresco-early",
      ],
    },
    versions: {
      vueClaims: ["2.7-host-compiler", "3.x-vdom", "3.6+-vapor-experimental"],
      testedNode: ["22", "24"],
      acceptedNodeEngine: nodeEngine,
      bun,
    },
    combinations,
    requiredRows,
    evidenceSnapshots,
    knownGaps,
    contentMapper,
    environments: {
      nativeTargets,
      releasedNativeTargets,
      releasedCliTargets,
      vscodeProviderTargets,
      hostRuntimeCombinations,
      packageManagers,
      freshProjects,
      editors,
    },
    topologies: V1_TOPOLOGIES.map((id) => ({
      id,
      qualification: "unbound",
      evidenceCandidates: {
        monorepo: [
          "crates/vize/tests/check_monorepo_tsconfig_cli.rs",
          "crates/vize/tests/check_nuxt_monorepo_cli.rs",
          "tools/support/compat/npm/smoke-release-init-workspace.ts",
        ],
        "project-reference": [
          "crates/vize/src/commands/check/tsconfig_inputs/tests/reference_programs.rs",
          "crates/vize/tests/content_mapper_tsgo_package_references.rs",
        ],
        "alias-package-export": [
          "crates/vize/tests/check_default_imports_cli/workspace_package_vue_exports.rs",
          "crates/vize/tests/check_nuxt_tsconfig_paths_cli.rs",
        ],
        "unicode-space": [
          "crates/vize/tests/content_mapper_tsgo_cli/unicode_prop_bindings.rs",
          "crates/vize/tests/lsp_unicode_diagnostics_cli.rs",
        ],
        crlf: [
          "crates/vize_atelier_vapor/tests/patterned_template_recovery.rs",
          "crates/vize/tests/lsp_patterned_navigation_cli.rs",
        ],
        symlink: [
          "crates/vize/tests/check_symlinked_node_modules_cli.rs",
          "crates/vize/tests/lsp_symlink_workspace_staging_cli.rs",
        ],
      }[id],
    })),
    identityContracts: {
      source: ["github.sha", "git rev-parse HEAD"],
      fixture: ["sha256", "git submodule revision"],
      dependency: ["pnpm-lock.yaml", "frozen lockfile", "packed tarball sha256"],
      runtime: ["runtime version", "runner OS", "architecture", "libc", "filesystem"],
      binary: ["native build token", "server sha256", "package version", "target triple"],
      producers: [
        ".github/workflows/native-smoke.yml",
        ".github/workflows/real-project-matrix.yml",
        ".github/workflows/editor-conformance.yml",
        ".github/workflows/typecheck-wsl2.yml",
      ],
    },
    unsupported: [
      {
        combination: "vue-2.7/vapor",
        disposition: "unsupported",
        reason: "Vapor starts at Vue 3.6; Vue 2.7 integration retains the host VDOM compiler.",
      },
      {
        combination: "linux-musl/bun/content-mapper",
        disposition: "unsupported",
        reason: "The bundled TypeScript provider is glibc-only; musl remains a Node 24 lane.",
      },
      {
        combination: "linux-musl/node-22",
        disposition: "unsupported",
        reason: "The current packed musl qualification runs Node 24 only.",
      },
      {
        combination: "wsl2/shipped-editor-hosts",
        disposition: "unsupported",
        reason: "WSL2 qualifies the typechecker and LSP server, not every shipped editor host.",
      },
      {
        combination: "vue-pre-2.7/v1",
        disposition: "outside-v1",
        reason: "Older legacy dialect compatibility is not a v1 support claim.",
      },
      {
        combination: "eslint-prettier-babel",
        disposition: "outside-v1",
        reason: "These adapters are deferred beyond v1.",
      },
    ],
    counts: {
      declaredCombinations: combinations.length,
      caseObligations: combinations.length * V1_CASES.length,
      hostRuntimeCombinations: hostRuntimeCombinations.length,
      requiredRows: requiredRows.length,
      completeRequiredRows: requiredRows.filter(({ qualification }) => qualification === "complete")
        .length,
      blockingGaps: knownGaps.length,
      packageManagers: packageManagers.length,
      freshProjectCells: freshProjects.length,
      editors: editors.length,
      topologies: V1_TOPOLOGIES.length,
    },
  };
}

export function renderV1SupportMatrix(root = process.cwd()): string {
  return `${JSON.stringify(buildV1SupportMatrix(root), null, 2)}\n`;
}

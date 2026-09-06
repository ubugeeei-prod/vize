import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import { compareTypecheckDiagnostics } from "../../legacy-tools/fixtures/typecheck-divergence.mjs";
import * as compatNonVacuity from "./compat-nonvacuity.ts";
import type { CompatNonVacuity } from "./compat-nonvacuity.ts";
import { repoRoot, symlinkDirectory, withPinnedFixtureWorkspace } from "./realworld-patch.ts";
import {
  resolveTsgoBinary,
  resolveVueTscBinary,
  runVizeCheck,
  runVueTsc,
  symlinkVueTypes,
} from "./realworld-typecheck.ts";
import { resolveVueTscManifestPath } from "./vue-tsc-manifest.ts";

/**
 * Per-PR drop-in compatibility probes. Each probe copies a pinned set of real
 * files out of a hydrated vue-parity fixture, points vize check and vue-tsc at
 * the exact same isolated workspace, and classifies the diagnostics with the
 * same comparator the weekly real-project matrix uses
 * (legacy-tools/fixtures/typecheck-divergence.mjs, schema
 * vize.fixtureTypecheckDivergence). Divergence — not diagnostic count — is the
 * metric: unresolved project-internal imports are expected and must surface
 * identically from both tools.
 */
export type CompatProbe = {
  fixtureId: string;
  /** Files or directories copied from the pinned fixture into the workspace. */
  includePaths: string[];
  /** tsconfig `include` entries; both tools are driven by the same tsconfig. */
  include: string[];
  /** Packages symlinked from the repo workspace node_modules, when present. */
  packages?: string[];
  /** Extra compilerOptions merged over the canonical probe tsconfig. */
  compilerOptions?: Record<string, unknown>;
};

export type CompatSummary = {
  vizeDiagnosticCount: number;
  baselineDiagnosticCount: number;
  sharedCount: number;
  /**
   * Pairs that agree on file, severity, line, column and code but not on the
   * message text. Split out of `sharedCount` so a divergence that lives only in
   * the wording is visible to the ratchet (#3447).
   */
  messageMismatchCount: number;
  /** Same-span pairs cancelled by tests/_fixtures/compat-documented-differences.json. */
  documentedDifferenceCount: number;
  falsePositiveCount: number;
  falseNegativeCount: number;
  falsePositiveRatio: number;
  falseNegativeRatio: number;
};

export type CompatDocumentedDifferences = {
  schema: "vize.compatDocumentedDifferences";
  version: 1;
  differences: Array<{ project: string }>;
};

export type CompatBaseline = {
  schema: "vize.compatBaseline";
  version: 1;
  vueTsc: string;
  projects: Record<string, CompatBaselineEntry>;
};

export type CompatBaselineEntry = CompatSummary & {
  revision: string;
  accepted: boolean;
};

export type CompatProbeResult = {
  revision: string;
  summary: CompatSummary;
  accepted: boolean;
  nonVacuity: CompatNonVacuity | null;
  vizeDurationMs: number;
  vueTscDurationMs: number;
};

export type TypecheckPerformanceBudget = {
  enabled?: boolean;
  maxFalsePositiveRatio?: number;
  maxFalseNegativeRatio?: number;
};

export const compatBaselinePath = path.join(repoRoot, "tests/_fixtures/compat-baseline.json");

export const compatDocumentedDifferencesPath = path.join(
  repoRoot,
  "tests/_fixtures/compat-documented-differences.json",
);

export const compatProbes: CompatProbe[] = [
  {
    fixtureId: "create-vue",
    includePaths: ["template/code/typescript-default/src"],
    include: ["template/code/typescript-default/src"],
  },
  {
    fixtureId: "element-plus",
    includePaths: ["docs/examples/badge", "docs/examples/tag"],
    include: ["docs/examples/badge", "docs/examples/tag"],
    packages: ["element-plus"],
    compilerOptions: { types: ["element-plus/global"] },
  },
  {
    fixtureId: "misskey",
    includePaths: [
      "packages/frontend/src/components/global/MkCondensedLine.vue",
      "packages/frontend/src/components/global/MkEllipsis.vue",
      "packages/frontend/src/components/global/MkTime.vue",
      "packages/frontend/src/components/MkAnimBg.vue",
    ],
    include: ["packages/frontend/src"],
    compilerOptions: { paths: { "@/*": ["./packages/frontend/src/*"] } },
  },
  {
    fixtureId: "nuxt-ui",
    includePaths: ["docs/app/components/content/examples/tabs"],
    include: ["docs/app/components/content/examples/tabs"],
  },
  {
    fixtureId: "pinia",
    includePaths: [
      "packages/docs/.vitepress/theme/components/PiniaLogo.vue",
      "packages/docs/.vitepress/theme/components/VueSchoolLink.vue",
      "packages/docs/.vitepress/theme/components/HomeSponsorsGroup.vue",
    ],
    include: ["packages/docs/.vitepress/theme/components"],
  },
  {
    fixtureId: "vue-router",
    includePaths: [
      "packages/experiments-playground/src/App.vue",
      "packages/experiments-playground/src/pages",
    ],
    include: ["packages/experiments-playground/src"],
    packages: ["vue-router"],
  },
  {
    fixtureId: "vue-element-admin",
    includePaths: ["src/components/Hamburger/index.vue", "src/components/Breadcrumb/index.vue"],
    include: ["src"],
    compilerOptions: {
      allowJs: true,
      // Deliberately no `checkJs`: the probe mirrors the project's own tsconfig, so it
      // measures the drop-in behavior this gate ships (JavaScript SFCs stay unchecked,
      // exactly like vue-tsc). Turning `checkJs` on moves the vue-tsc side of the ledger
      // and would require regenerating tests/_fixtures/compat-baseline.json.
      paths: { "@/*": ["./src/*"] },
      strict: false,
    },
  },
  {
    fixtureId: "vitepress",
    includePaths: [
      "src/client/theme-default/NotFound.vue",
      "src/client/theme-default/composables/data.ts",
    ],
    include: ["src/client/theme-default"],
  },
  {
    fixtureId: "vue-vben-admin",
    includePaths: ["playground/src/views/demos/access"],
    include: ["playground/src/views/demos/access"],
    packages: ["vue-router"],
  },
];

export function isFixtureHydrated(fixtureId: string): boolean {
  return fs.existsSync(path.join(repoRoot, "tests/_fixtures/_git", fixtureId, "package.json"));
}

export function resolveCompatVueTscVersion(): string {
  // Derive the version from the same binary the probes execute so the pin can
  // never drift from the tool actually producing the baseline side.
  const binary = resolveVueTscBinary();
  const manifestPath = resolveVueTscManifestPath(binary);
  assert.ok(
    manifestPath,
    `vue-tsc package manifest not found for ${binary}; neither the bin entry itself nor any ` +
      "cmd-shim target it names led to a package.json declaring vue-tsc",
  );
  const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8")) as { version?: unknown };
  assert.equal(typeof manifest.version, "string", "vue-tsc package version must be a string");
  return manifest.version as string;
}

export async function runCompatProbe(probe: CompatProbe): Promise<CompatProbeResult> {
  const corsaPath = resolveTsgoBinary();
  const vueTscPath = resolveVueTscBinary();

  return await withPinnedFixtureWorkspace(
    { fixtureId: probe.fixtureId, includePaths: probe.includePaths },
    async (fixture) => {
      symlinkVueTypes(fixture.workspaceDir);
      for (const name of probe.packages ?? []) {
        symlinkWorkspacePackage(fixture.workspaceDir, name);
      }
      fixture.write("tsconfig.json", `${JSON.stringify(probeTsconfig(probe), null, 2)}\n`);

      const vizeStartedAt = Date.now();
      let vize: ReturnType<typeof runVizeCheck>;
      try {
        vize = runVizeCheck(fixture.workspaceDir, corsaPath, []);
      } catch (error) {
        throw new Error(
          `vize check did not produce a JSON report for ${probe.fixtureId}; ` +
            "build a vize binary (for example cargo build --release -p vize) and expose it via VIZE_TEST_BIN",
          { cause: error },
        );
      }
      const vizeDurationMs = Date.now() - vizeStartedAt;
      assert.ok(
        vize.status === 0 || vize.status === 1,
        `vize check must complete for ${probe.fixtureId}: exit ${vize.status}\n${vize.stderr}`,
      );

      const vueTscStartedAt = Date.now();
      const vueTsc = runVueTsc(fixture.workspaceDir, vueTscPath);
      const vueTscDurationMs = Date.now() - vueTscStartedAt;
      assert.ok(
        vueTsc.status === 0 || vueTsc.status === 1 || vueTsc.status === 2,
        `vue-tsc must complete for ${probe.fixtureId}: exit ${vueTsc.status}\n${vueTsc.stderr}`,
      );

      const documentedDifferences = readCompatDocumentedDifferences().differences;
      const divergence = compareTypecheckDiagnostics({
        projectId: probe.fixtureId,
        cwd: fixture.workspaceDir,
        vizeReport: vize.report,
        vueTscOutput: `${vueTsc.stdout}\n${vueTsc.stderr}`,
        documentedDifferences,
      }) as { summary: CompatSummary & Record<string, number> };

      const summary = compatNonVacuity.compatSummaryFromDivergence(divergence.summary);
      const nonVacuity = compatNonVacuity.isDiagnosticFreeSummary(summary)
        ? compatNonVacuity.runCompatNonVacuityProbe({
            probe,
            fixture,
            corsaPath,
            vueTscPath,
            documentedDifferences,
          })
        : null;
      return {
        revision: fixture.entry.revision,
        summary,
        accepted: isAcceptedByRegistryBudget(probe.fixtureId, summary),
        nonVacuity,
        vizeDurationMs,
        vueTscDurationMs,
      };
    },
  );
}

export function readCompatBaseline(): CompatBaseline {
  const baseline = JSON.parse(fs.readFileSync(compatBaselinePath, "utf8")) as CompatBaseline;
  assert.equal(baseline.schema, "vize.compatBaseline", "compat baseline schema must match");
  assert.equal(baseline.version, 1, "compat baseline version must match");
  return baseline;
}

export function readCompatDocumentedDifferences(): CompatDocumentedDifferences {
  const ledger = JSON.parse(
    fs.readFileSync(compatDocumentedDifferencesPath, "utf8"),
  ) as CompatDocumentedDifferences;
  assert.equal(
    ledger.schema,
    "vize.compatDocumentedDifferences",
    "documented difference ledger schema must match",
  );
  assert.equal(ledger.version, 1, "documented difference ledger version must match");
  assert.ok(
    Array.isArray(ledger.differences),
    "documented difference ledger must list differences",
  );
  return ledger;
}

export function writeCompatBaseline(baseline: CompatBaseline): void {
  const projects = Object.fromEntries(
    Object.keys(baseline.projects)
      .sort()
      .map((id) => [id, baseline.projects[id]]),
  );
  fs.writeFileSync(compatBaselinePath, `${JSON.stringify({ ...baseline, projects }, null, 2)}\n`);
}

function probeTsconfig(probe: CompatProbe): Record<string, unknown> {
  return {
    compilerOptions: {
      lib: ["ES2022", "DOM", "DOM.Iterable"],
      module: "ESNext",
      moduleResolution: "bundler",
      noEmit: true,
      skipLibCheck: true,
      strict: true,
      target: "ES2022",
      types: [],
      ...probe.compilerOptions,
    },
    include: probe.include,
  };
}

function symlinkWorkspacePackage(workspaceDir: string, name: string): void {
  const source = [
    path.join(repoRoot, "tests/node_modules", name),
    path.join(repoRoot, "node_modules", name),
  ].find((candidate) => fs.existsSync(candidate));
  assert.ok(source, `package ${name} must be installed in the repo workspace`);
  symlinkDirectory(source, path.join(workspaceDir, "node_modules", name));
}

function isAcceptedByRegistryBudget(fixtureId: string, summary: CompatSummary): boolean {
  const registry = JSON.parse(
    fs.readFileSync(path.join(repoRoot, "tests/_fixtures/vue-ecosystem-fixtures.json"), "utf8"),
  ) as {
    projects: Array<{
      id: string;
      typecheckPerformance?: TypecheckPerformanceBudget;
    }>;
  };
  const performance = registry.projects.find(
    (project) => project.id === fixtureId,
  )?.typecheckPerformance;
  return isAcceptedByTypecheckBudget(summary, performance);
}

export function isAcceptedByTypecheckBudget(
  summary: CompatSummary,
  performance: TypecheckPerformanceBudget | undefined,
): boolean {
  if (performance?.enabled !== true) {
    return summary.falsePositiveCount === 0 && summary.falseNegativeCount === 0;
  }
  return (
    summary.falsePositiveRatio <= (performance.maxFalsePositiveRatio ?? 1) &&
    summary.falseNegativeRatio <= (performance.maxFalseNegativeRatio ?? 1)
  );
}

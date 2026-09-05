/**
 * Per-PR drop-in compatibility ratchet (part of the #3227 compat roadmap).
 *
 * The weekly real-project matrix (.github/workflows/real-project-matrix.yml)
 * is the baseline-refresh authority: it typechecks the full pinned ecosystem
 * with installed dependencies and records the authoritative divergence ledger
 * (schema vize.fixtureTypecheckDivergence). This suite is the per-PR gate over
 * the hydrated vue-parity fixtures: it recomputes vize-versus-vue-tsc
 * divergence on pinned probe workspaces with the same comparator the weekly
 * run uses, and asserts every metric holds or improves relative to
 * tests/_fixtures/compat-baseline.json. A PR can never silently loosen the
 * ledger; intentional improvements must tighten the baseline in the same PR.
 *
 * Differences that are expected — vize is faithful and vue-tsc's answer is an
 * artifact of its own checker — are recorded entry by entry in
 * tests/_fixtures/compat-documented-differences.json and counted separately from
 * false positives and false negatives, so the probe stops reporting a divergence
 * it has already reviewed while every diagnostic stays accounted for.
 *
 * Regenerate the baseline (hydrated fixtures + a fresh vize binary required):
 *   UPDATE_COMPAT_BASELINE=1 VIZE_TEST_BIN=target/release/vize \
 *     node --test tests/tooling/compat-ratchet.test.ts
 */
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { after, test } from "node:test";

import {
  type CompatBaseline,
  type CompatBaselineEntry,
  type CompatSummary,
  compatBaselinePath,
  compatProbes,
  isAcceptedByTypecheckBudget,
  isFixtureHydrated,
  readCompatBaseline,
  readCompatDocumentedDifferences,
  resolveCompatVueTscVersion,
  runCompatProbe,
  writeCompatBaseline,
} from "../_helpers/compat-ratchet.ts";
import { resolveVueTscManifestPath } from "../_helpers/vue-tsc-manifest.ts";

const updateBaseline = process.env.UPDATE_COMPAT_BASELINE === "1";
const baselineExists = fs.existsSync(compatBaselinePath);
const refreshInstruction =
  "regenerate it in this PR with: UPDATE_COMPAT_BASELINE=1 VIZE_TEST_BIN=target/release/vize " +
  "node --test tests/tooling/compat-ratchet.test.ts (hydrate the vue-parity fixtures first)";
const updatedEntries = new Map<string, CompatBaselineEntry>();

const exactParity: CompatSummary = {
  vizeDiagnosticCount: 0,
  baselineDiagnosticCount: 0,
  sharedCount: 0,
  messageMismatchCount: 0,
  documentedDifferenceCount: 0,
  falsePositiveCount: 0,
  falseNegativeCount: 0,
  falsePositiveRatio: 0,
  falseNegativeRatio: 0,
};

type RatchetRule = {
  metric: keyof CompatSummary;
  /** "<=" holds-or-improves downward, ">=" upward, "==" must stay exact. */
  direction: "<=" | ">=" | "==";
};

const ratchetRules: RatchetRule[] = [
  { metric: "falsePositiveCount", direction: "<=" },
  { metric: "falsePositiveRatio", direction: "<=" },
  { metric: "falseNegativeCount", direction: "<=" },
  { metric: "falseNegativeRatio", direction: "<=" },
  { metric: "sharedCount", direction: ">=" },
  // A diagnostic vize reports at vue-tsc's exact span and code but with
  // different text is a divergence the (file, severity, line, column, code)
  // identity cannot see, which is how the #3397 `.vue.ts` specifier leak lived
  // on main undetected (#3447). Every probe is at zero today, so any wording
  // drift fails here.
  { metric: "messageMismatchCount", direction: "<=" },
  { metric: "baselineDiagnosticCount", direction: "==" },
  // Neither growth nor decay is silent: a new expected difference has to land
  // with its ledger entry and a fresh baseline, and one that stops reproducing
  // has to be retired from the ledger in the PR that fixes it.
  { metric: "documentedDifferenceCount", direction: "==" },
];

for (const probe of compatProbes) {
  const hydrated = isFixtureHydrated(probe.fixtureId);
  test(
    `compat ratchet holds or improves typecheck divergence for ${probe.fixtureId}`,
    { skip: hydrated ? false : `${probe.fixtureId} fixture is not hydrated` },
    async () => {
      const result = await runCompatProbe(probe);
      const { summary } = result;
      const paired = summary.sharedCount + summary.messageMismatchCount;
      assert.equal(
        summary.vizeDiagnosticCount,
        paired + summary.documentedDifferenceCount + summary.falsePositiveCount,
        `${probe.fixtureId}: divergence summary lost vize diagnostics`,
      );
      assert.equal(
        summary.baselineDiagnosticCount,
        paired + summary.documentedDifferenceCount + summary.falseNegativeCount,
        `${probe.fixtureId}: divergence summary lost vue-tsc diagnostics`,
      );
      console.log(
        `${probe.fixtureId}: shared=${summary.sharedCount}` +
          ` messageMismatches=${summary.messageMismatchCount}` +
          ` documented=${summary.documentedDifferenceCount}` +
          ` falsePositives=${summary.falsePositiveCount} (${summary.falsePositiveRatio.toFixed(4)})` +
          ` falseNegatives=${summary.falseNegativeCount} (${summary.falseNegativeRatio.toFixed(4)})` +
          ` vize=${result.vizeDurationMs}ms vue-tsc=${result.vueTscDurationMs}ms`,
      );

      if (updateBaseline) {
        updatedEntries.set(probe.fixtureId, {
          revision: result.revision,
          accepted: result.accepted,
          ...summary,
        });
        return;
      }

      assert.ok(
        baselineExists,
        `tests/_fixtures/compat-baseline.json is missing; ${refreshInstruction}`,
      );
      const baseline = readCompatBaseline();
      const entry = baseline.projects[probe.fixtureId];
      assert.ok(
        entry,
        `compat baseline has no entry for ${probe.fixtureId}; ${refreshInstruction}`,
      );
      assert.equal(
        entry.revision,
        result.revision,
        `${probe.fixtureId} fixture revision changed (baseline ${entry.revision}); ` +
          `the probe inputs moved, so ${refreshInstruction}`,
      );

      const failures: string[] = [];
      const improvements: string[] = [];
      for (const rule of ratchetRules) {
        const baselineValue = entry[rule.metric];
        const current = summary[rule.metric];
        if (current === baselineValue) continue;
        const regressed =
          rule.direction === "==" ||
          (rule.direction === "<=" ? current > baselineValue : current < baselineValue);
        const line = `${rule.metric}: baseline ${baselineValue} -> current ${current}`;
        if (regressed) failures.push(line);
        else improvements.push(line);
      }
      if (entry.accepted && !result.accepted) {
        failures.push(`accepted: baseline true -> current false (registry budget exceeded)`);
      }

      assert.equal(
        failures.length,
        0,
        `${probe.fixtureId} regressed drop-in typecheck compatibility:\n  ${failures.join("\n  ")}\n` +
          `Per-PR divergence must hold or improve against tests/_fixtures/compat-baseline.json. ` +
          `Fix the regression, or if the baseline itself is stale (fixture, vue-tsc, or probe change), ${refreshInstruction}`,
      );
      if (improvements.length > 0) {
        console.log(
          `${probe.fixtureId} improved on the compat baseline (tighten it in this PR with ` +
            `UPDATE_COMPAT_BASELINE=1):\n  ${improvements.join("\n  ")}`,
        );
      }
    },
  );
}

test("compat probes without a configured budget accept only exact parity", () => {
  assert.equal(isAcceptedByTypecheckBudget(exactParity, undefined), true);
  assert.equal(isAcceptedByTypecheckBudget(exactParity, { enabled: false }), true);
  assert.equal(
    isAcceptedByTypecheckBudget(
      {
        ...exactParity,
        vizeDiagnosticCount: 1,
        baselineDiagnosticCount: 1,
        documentedDifferenceCount: 1,
      },
      { enabled: false },
    ),
    true,
  );
  assert.equal(
    isAcceptedByTypecheckBudget(
      { ...exactParity, vizeDiagnosticCount: 1, falsePositiveCount: 1, falsePositiveRatio: 1 },
      undefined,
    ),
    false,
  );
  assert.equal(
    isAcceptedByTypecheckBudget(
      { ...exactParity, baselineDiagnosticCount: 1, falseNegativeCount: 1, falseNegativeRatio: 1 },
      undefined,
    ),
    false,
  );
});

test(
  "compat baseline is pinned to the workspace vue-tsc",
  {
    skip: updateBaseline || !baselineExists ? "baseline is being regenerated or absent" : false,
  },
  () => {
    const baseline = readCompatBaseline();
    assert.equal(
      baseline.vueTsc,
      resolveCompatVueTscVersion(),
      `compat baseline was recorded against vue-tsc ${readCompatBaseline().vueTsc}; ` +
        `the installed vue-tsc changed, so the divergence ledger moved — ${refreshInstruction}`,
    );
    for (const probe of compatProbes) {
      assert.ok(
        baseline.projects[probe.fixtureId],
        `compat baseline has no entry for ${probe.fixtureId}; ${refreshInstruction}`,
      );
    }
  },
);

test(
  "every documented difference belongs to a typecheck gate",
  {
    skip: updateBaseline || !baselineExists ? "baseline is being regenerated or absent" : false,
  },
  () => {
    const baseline = readCompatBaseline();
    const differences = readCompatDocumentedDifferences().differences;
    const probeIds = new Set(compatProbes.map((probe) => probe.fixtureId));
    const registryPath = path.join(path.dirname(compatBaselinePath), "vue-ecosystem-fixtures.json");
    const registry = JSON.parse(fs.readFileSync(registryPath, "utf8")) as {
      projects: Array<{ id: string; typecheckPerformance?: { enabled?: boolean } }>;
    };
    const typecheckProjectIds = new Set(
      registry.projects
        .filter((project) => project.typecheckPerformance?.enabled === true)
        .map((project) => project.id),
    );
    for (const difference of differences) {
      assert.ok(
        probeIds.has(difference.project) || typecheckProjectIds.has(difference.project),
        `documented difference names an unknown typecheck project: ${difference.project}`,
      );
    }
    for (const probe of compatProbes) {
      const entry = baseline.projects[probe.fixtureId];
      if (entry == null) continue;
      assert.equal(
        entry.documentedDifferenceCount,
        differences.filter((difference) => difference.project === probe.fixtureId).length,
        `${probe.fixtureId}: the baseline counts documented differences the ledger does not ` +
          `describe (or the reverse); every expected difference needs an entry in ` +
          `tests/_fixtures/compat-documented-differences.json`,
      );
    }
  },
);

test("vue-tsc version resolution survives every pnpm bin layout", (t) => {
  // The version pin above is only as trustworthy as this resolution. pnpm picks
  // between a store symlink and a cmd-shim script depending on platform and
  // settings; a resolver that only understands one shape makes the whole gate
  // unrunnable on the other, which is how it can stop being exercised unnoticed.
  // realpath the sandbox: on macOS the temp dir is itself a symlink, and the
  // resolver reports realpaths.
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-vue-tsc-bin-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));

  const packageDir = path.join(root, "node_modules/.pnpm/vue-tsc@9.9.9/node_modules/vue-tsc");
  fs.mkdirSync(path.join(packageDir, "bin"), { recursive: true });
  const entry = path.join(packageDir, "bin/vue-tsc.js");
  fs.writeFileSync(entry, "#!/usr/bin/env node\n");
  const manifestPath = path.join(packageDir, "package.json");
  fs.writeFileSync(manifestPath, `${JSON.stringify({ name: "vue-tsc", version: "9.9.9" })}\n`);
  // An enclosing workspace manifest must never be mistaken for the package.
  fs.writeFileSync(
    path.join(root, "package.json"),
    `${JSON.stringify({ name: "workspace-root", version: "0.0.0" })}\n`,
  );

  const binDir = path.join(root, "node_modules/.bin");
  fs.mkdirSync(binDir, { recursive: true });
  const layouts: Array<[layout: string, write: (binPath: string) => void]> = [
    ["store symlink", (binPath) => fs.symlinkSync(entry, binPath)],
    [
      "cmd-shim naming its target",
      (binPath) =>
        fs.writeFileSync(binPath, `#!/bin/sh\nexec node "$@"\n# cmd-shim-target=${entry}\n`),
    ],
    [
      // Shaped like a real marker-less shim: the interpreter is named through
      // `$basedir` too, and it comes first, so the target cannot be taken as
      // simply the first `$basedir`-relative path in the script.
      "cmd-shim without the target marker",
      (binPath) =>
        fs.writeFileSync(
          binPath,
          '#!/bin/sh\nbasedir_win="$basedir"\nexec "$basedir/node"  ' +
            '"$basedir/../.pnpm/vue-tsc@9.9.9/node_modules/vue-tsc/bin/vue-tsc.js" "$@"\n',
        ),
    ],
  ];

  for (const [layout, write] of layouts) {
    const binPath = path.join(binDir, "vue-tsc");
    fs.rmSync(binPath, { force: true });
    write(binPath);
    assert.equal(resolveVueTscManifestPath(binPath), manifestPath, `${layout} must resolve`);
  }

  // A bin entry that leads nowhere must report that, not slice a bogus path.
  const orphan = path.join(binDir, "vue-tsc-orphan");
  fs.writeFileSync(orphan, "#!/bin/sh\nexec node /nowhere/vue-tsc.js\n");
  assert.equal(resolveVueTscManifestPath(orphan), undefined);
});

after(() => {
  if (!updateBaseline || updatedEntries.size === 0) return;
  const previous: CompatBaseline | null = baselineExists ? readCompatBaseline() : null;
  const missing = compatProbes
    .map((probe) => probe.fixtureId)
    .filter((id) => !updatedEntries.has(id) && previous?.projects[id] == null);
  assert.equal(
    missing.length,
    0,
    `cannot regenerate a complete compat baseline; hydrate and rerun for: ${missing.join(", ")}`,
  );
  const baseline: CompatBaseline = {
    schema: "vize.compatBaseline",
    version: 1,
    vueTsc: resolveCompatVueTscVersion(),
    projects: { ...previous?.projects, ...Object.fromEntries(updatedEntries) },
  };
  writeCompatBaseline(baseline);
  console.log(`compat baseline updated: ${compatBaselinePath}`);
});

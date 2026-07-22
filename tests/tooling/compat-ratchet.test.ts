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
 * Regenerate the baseline (hydrated fixtures + a fresh vize binary required):
 *   UPDATE_COMPAT_BASELINE=1 VIZE_TEST_BIN=target/release/vize \
 *     node --test tests/tooling/compat-ratchet.test.ts
 */
import assert from "node:assert/strict";
import fs from "node:fs";
import { after, test } from "node:test";

import {
  type CompatBaseline,
  type CompatBaselineEntry,
  type CompatSummary,
  compatBaselinePath,
  compatProbes,
  isFixtureHydrated,
  readCompatBaseline,
  resolveCompatVueTscVersion,
  runCompatProbe,
  writeCompatBaseline,
} from "../_helpers/compat-ratchet.ts";

const updateBaseline = process.env.UPDATE_COMPAT_BASELINE === "1";
const baselineExists = fs.existsSync(compatBaselinePath);
const refreshInstruction =
  "regenerate it in this PR with: UPDATE_COMPAT_BASELINE=1 VIZE_TEST_BIN=target/release/vize " +
  "node --test tests/tooling/compat-ratchet.test.ts (hydrate the vue-parity fixtures first)";
const updatedEntries = new Map<string, CompatBaselineEntry>();

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
  { metric: "baselineDiagnosticCount", direction: "==" },
];

for (const probe of compatProbes) {
  const hydrated = isFixtureHydrated(probe.fixtureId);
  test(
    `compat ratchet holds or improves typecheck divergence for ${probe.fixtureId}`,
    { skip: hydrated ? false : `${probe.fixtureId} fixture is not hydrated` },
    async () => {
      const result = await runCompatProbe(probe);
      const { summary } = result;
      assert.equal(
        summary.vizeDiagnosticCount,
        summary.sharedCount + summary.falsePositiveCount,
        `${probe.fixtureId}: divergence summary lost vize diagnostics`,
      );
      assert.equal(
        summary.baselineDiagnosticCount,
        summary.sharedCount + summary.falseNegativeCount,
        `${probe.fixtureId}: divergence summary lost vue-tsc diagnostics`,
      );
      console.log(
        `${probe.fixtureId}: shared=${summary.sharedCount}` +
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

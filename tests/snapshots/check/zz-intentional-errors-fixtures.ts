import assert from "node:assert/strict";
import { before, describe, it } from "node:test";

import {
  classComponentApp,
  compilerMacrosApp,
  ecosystemProductsApp,
  nuxtParityApp,
  optionsApiApp,
  requireVizeAndCorsaBins,
  stylePreprocessorsApp,
  typecheckErrorsApp,
  vueVbenAdminApp,
} from "../../_helpers/apps.ts";
import { genericBuildApp, typecheckVueImportsApp } from "../../_helpers/fixture-apps.ts";
import { runVizeCheckWithInjectedTypeError } from "../_helpers/realworld.ts";

const fixtureApps = [
  typecheckErrorsApp,
  typecheckVueImportsApp,
  compilerMacrosApp,
  stylePreprocessorsApp,
  ecosystemProductsApp,
  genericBuildApp,
  nuxtParityApp,
  optionsApiApp,
  classComponentApp,
  // #3747: keep one Tier-L batch broken -> repaired oracle on every PR.
  vueVbenAdminApp,
] as const;

describe("fixture vize check injected type errors", () => {
  before(requireVizeAndCorsaBins);

  for (const app of fixtureApps) {
    it(`${app.name} catches and repairs an injected TS2322`, () => {
      const summary = runVizeCheckWithInjectedTypeError(app, { timeoutMs: 120_000 });
      assert.deepEqual(summary.repairedDiagnostics, []);
      console.log(
        `${app.name}: file=${summary.file}, fileCount=${summary.fileCount}, errorCount=${summary.errorCount}, durationMs=${summary.durationMs.toFixed(0)}, repairDurationMs=${summary.repairDurationMs.toFixed(0)}`,
      );
    });
  }
});

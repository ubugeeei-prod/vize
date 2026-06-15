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
] as const;

describe("fixture vize check injected type errors", () => {
  before(requireVizeAndCorsaBins);

  for (const app of fixtureApps) {
    it(`${app.name} catches an injected TS2322`, () => {
      const summary = runVizeCheckWithInjectedTypeError(app, { timeoutMs: 120_000 });
      console.log(
        `${app.name}: file=${summary.file}, fileCount=${summary.fileCount}, errorCount=${summary.errorCount}, durationMs=${summary.durationMs.toFixed(0)}`,
      );
    });
  }
});

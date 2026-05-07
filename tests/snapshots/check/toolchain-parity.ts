import { before, describe, it } from "node:test";
import {
  compilerMacrosApp,
  stylePreprocessorsApp,
  typecheckErrorsApp,
} from "../../_helpers/apps.ts";
import {
  assertOfficialCompilerAccepts,
  assertVueTscDiagnosticSurface,
  hasToolchainParityBinaries,
} from "../../_helpers/toolchain-parity.ts";

const fixtureApps = [
  { app: compilerMacrosApp, expectErrors: false },
  { app: stylePreprocessorsApp, expectErrors: false },
  { app: typecheckErrorsApp, expectErrors: true },
] as const;

describe("check fixture parity with Vue toolchain", () => {
  before(() => {
    if (!hasToolchainParityBinaries()) {
      console.log("Skipping: vize, checker, or vue-tsc binary was not found");
      process.exit(0);
    }
  });

  for (const { app, expectErrors } of fixtureApps) {
    it(`${app.name} matches vue-tsc diagnostic surface`, () => {
      assertVueTscDiagnosticSurface(app, { expectErrors });
    });

    it(`${app.name} compiles with @vue/compiler-sfc`, () => {
      assertOfficialCompilerAccepts(app);
    });
  }
});

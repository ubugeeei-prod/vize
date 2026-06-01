import { before, describe, it } from "node:test";
import {
  compilerMacrosApp,
  stylePreprocessorsApp,
  typecheckErrorsApp,
} from "../../_helpers/apps.ts";
import {
  assertOfficialCompilerAccepts,
  assertVueTscDiagnosticSurface,
  requireToolchainParityBinaries,
} from "../../_helpers/toolchain-parity.ts";

const fixtureApps = [
  { app: compilerMacrosApp, expectErrors: false },
  { app: stylePreprocessorsApp, expectErrors: false },
  { app: typecheckErrorsApp, expectErrors: true },
] as const;

describe("check fixture parity with Vue toolchain", () => {
  before(requireToolchainParityBinaries);

  for (const { app, expectErrors } of fixtureApps) {
    it(`${app.name} matches vue-tsc diagnostic surface`, () => {
      assertVueTscDiagnosticSurface(app, { expectErrors });
    });

    it(`${app.name} compiles with @vue/compiler-sfc`, () => {
      assertOfficialCompilerAccepts(app);
    });
  }
});

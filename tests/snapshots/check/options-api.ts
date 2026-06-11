import { before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import * as path from "node:path";
import { CORSA_BIN, optionsApiApp, VIZE_BIN } from "../../_helpers/apps.ts";
import {
  assertOfficialCompilerAccepts,
  assertVueTscDiagnosticSurface,
  requireToolchainParityBinaries,
} from "../../_helpers/toolchain-parity.ts";

const app = optionsApiApp;

// The Options API template-binding bridge is an opt-in feature, enabled here
// through the fixture's `vize.config.json` (`typeChecker.optionsApi: true`). The
// `vize check` invocation below therefore resolves `this`-typed `data` /
// `computed` / `methods` / `props` bindings in templates exactly like vue-tsc.
describe(`${app.name} check parity with Vue toolchain`, () => {
  before(requireToolchainParityBinaries);

  it("matches vue-tsc diagnostic surface (intentional error caught at same spot)", () => {
    // The fixture is intentionally not clean: `TypeErrorCounter.vue` calls
    // `count.toFixed(true)` (TS2345). vize and vue-tsc must agree on which files
    // report errors and share the diagnostic code, while the valid components
    // (`App.vue`, `Counter.vue`) stay clean for both.
    assertVueTscDiagnosticSurface(app, { expectErrors: true });
  });

  it("compiles with @vue/compiler-sfc", () => {
    assertOfficialCompilerAccepts(app);
  });

  it("reports the single intentional error at TypeErrorCounter.vue:16:33 [TS2345]", () => {
    const check = app.check!;
    const patterns = check.patterns.map((pattern) => `${pattern}`);
    let stdout: string;
    try {
      stdout = execFileSync(
        VIZE_BIN,
        ["check", ...patterns, "--format", "json", "--quiet", "--corsa-path", CORSA_BIN],
        { cwd: check.cwd, maxBuffer: 64 * 1024 * 1024, timeout: 120_000 },
      ).toString();
    } catch (error) {
      const e = error as { status?: number; stdout?: Buffer; stderr?: Buffer };
      if (e.status === 1 && e.stdout) {
        stdout = e.stdout.toString();
      } else {
        throw new Error(`vize check crashed (exit ${e.status}): ${e.stderr?.toString()}`);
      }
    }

    const parsed = JSON.parse(stdout) as {
      errorCount: number;
      files: Array<{ diagnostics: string[]; file: string }>;
    };

    assert.equal(parsed.errorCount, 1, "exactly one intentional error expected");
    const byFile = new Map(parsed.files.map((f) => [f.file.replaceAll(path.sep, "/"), f]));

    for (const clean of ["src/App.vue", "src/Counter.vue"]) {
      assert.deepEqual(
        byFile.get(clean)?.diagnostics ?? [],
        [],
        `${clean} should be clean (props/data/computed/methods/mixins/extends all resolve)`,
      );
    }

    const errorFile = byFile.get("src/TypeErrorCounter.vue");
    assert.ok(errorFile, "TypeErrorCounter.vue should be in the report");
    assert.equal(errorFile.diagnostics.length, 1);
    assert.match(errorFile.diagnostics[0]!, /^error:16:33 \[TS2345\]/);
  });
});

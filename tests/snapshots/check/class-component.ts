import { before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import {
  CORSA_BIN,
  classComponentApp,
  requireVizeAndCorsaBins,
  VIZE_BIN,
} from "../../_helpers/apps.ts";

const app = classComponentApp;

// vue-class-component / vue-property-decorator parity.
//
// Unlike the Options API fixture we do NOT assert a full vue-tsc diagnostic
// surface here. vue-tsc's official SFC virtual code for a `export default class
// extends Vue` component (vue-class-component v8 without an explicit runtime
// component) emits a *synthetic* default export that collides with the class
// (TS2528 "A module cannot have multiple default exports") and fails to resolve
// the class members on its component public-instance type (TS2339 "Property
// 'greeting' does not exist ..."). Those are vue-tsc artifacts, not real type
// errors — vize resolves the class instance type correctly and does not report
// them. So full diagnostic-surface parity is infeasible for the class case; we
// instead assert vize's own expected diagnostics (clean valid components + the
// single intentional error) and verify the intentional error is *also* seen by
// vue-tsc so the fixture stays meaningful. See docs/release/vue-parity-matrix.md.
//
// The Options API template-binding bridge (shared by class components) is opt-in
// and enabled via the fixture's `vize.config.json` (`typeChecker.optionsApi`).

interface VizeCheckReport {
  errorCount: number;
  files: Array<{ diagnostics: string[]; file: string }>;
}

function runVizeCheck(): VizeCheckReport {
  const check = app.check!;
  let stdout: string;
  try {
    stdout = execFileSync(
      VIZE_BIN,
      ["check", ...check.patterns, "--format", "json", "--quiet", "--corsa-path", CORSA_BIN],
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
  return JSON.parse(stdout) as VizeCheckReport;
}

function resolveVueTsc(): string | null {
  const testsRoot = path.resolve(import.meta.dirname, "..", "..");
  const repoRoot = path.resolve(testsRoot, "..");
  for (const dir of [testsRoot, repoRoot]) {
    const candidate = path.join(dir, "node_modules", ".bin", "vue-tsc");
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }
  return null;
}

describe(`${app.name} check (vue-class-component / vue-property-decorator)`, () => {
  before(requireVizeAndCorsaBins);

  it("vize check: valid class components clean, intentional error caught", () => {
    const report = runVizeCheck();
    const byFile = new Map(report.files.map((f) => [f.file.replaceAll(path.sep, "/"), f]));

    // @Prop / data field / getter / method all resolve as template bindings and
    // through the class instance type, so the valid components stay clean.
    for (const clean of ["src/App.vue", "src/HelloDecorator.vue"]) {
      assert.deepEqual(byFile.get(clean)?.diagnostics ?? [], [], `${clean} should be clean`);
    }

    const errorFile = byFile.get("src/TypeErrorDecorator.vue");
    assert.ok(errorFile, "TypeErrorDecorator.vue should be in the report");
    assert.equal(errorFile.diagnostics.length, 1, "exactly one intentional error expected");
    assert.match(errorFile.diagnostics[0]!, /^error:10:31 \[TS2345\]/);
    assert.equal(report.errorCount, 1);
  });

  it("vue-tsc agrees on the intentional TS2345 (class-member noise notwithstanding)", () => {
    const vueTsc = resolveVueTsc();
    if (vueTsc == null) {
      // vue-tsc is not installed in every checkout; the vize assertions above
      // are the source of truth for this fixture.
      return;
    }

    const result = spawnSync(vueTsc, ["--noEmit", "--pretty", "false", "-p", "tsconfig.json"], {
      cwd: app.check!.cwd,
      encoding: "utf8",
      env: { ...process.env, LANG: "C", LC_ALL: "C" },
      maxBuffer: 64 * 1024 * 1024,
      timeout: 120_000,
    });

    const output = `${result.stdout}${result.stderr}`;
    assert.match(
      output,
      /TypeErrorDecorator\.vue\(10,31\): error TS2345:/,
      "vue-tsc must also flag the intentional toFixed(true) error",
    );
  });
});

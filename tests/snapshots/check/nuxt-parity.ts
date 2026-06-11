import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { before, describe, it } from "node:test";
import {
  CORSA_BIN,
  VIZE_BIN,
  nuxtParityApp,
  requireVizeAndCorsaBins,
} from "../../_helpers/apps.ts";
import {
  assertOfficialCompilerAccepts,
  assertVueTscDiagnosticSurface,
  requireToolchainParityBinaries,
} from "../../_helpers/toolchain-parity.ts";

interface VizeCheckJson {
  errorCount: number;
  fileCount: number;
  files: Array<{ diagnostics: string[]; file: string }>;
}

const app = nuxtParityApp;
const check = app.check!;

// Run `vize check` over the given patterns and return the parsed JSON envelope.
// `vize check` exits non-zero when diagnostics are found, which `execFileSync`
// surfaces as a throw; the JSON envelope is still on stdout, so recover it.
function runVizeCheck(patterns: string[]): VizeCheckJson {
  let stdout: string;
  try {
    stdout = execFileSync(
      VIZE_BIN,
      ["check", ...patterns, "--format", "json", "--quiet", "--corsa-path", CORSA_BIN],
      { cwd: check.cwd, maxBuffer: 64 * 1024 * 1024, timeout: 120_000 },
    ).toString();
  } catch (error) {
    const err = error as { status?: number; stdout?: Buffer; stderr?: Buffer };
    if (err.status === 1 && err.stdout) {
      stdout = err.stdout.toString();
    } else {
      throw new Error(`vize check crashed (exit ${err.status}): ${err.stderr?.toString() ?? ""}`);
    }
  }
  return JSON.parse(stdout) as VizeCheckJson;
}

function diagnosticsFor(result: VizeCheckJson, file: string): string[] {
  return result.files.find((entry) => entry.file === file)?.diagnostics ?? [];
}

describe(`${app.name} check (Nuxt auto-import parity)`, () => {
  before(requireVizeAndCorsaBins);

  // No false positives: a page that uses an auto-imported composable, an
  // auto-registered component and a path alias declared in `.nuxt/tsconfig.json`
  // must type-check clean — none of these may degrade to permissive `any`.
  it("no false positives on auto-imports, components, and aliases", () => {
    const result = runVizeCheck(["pages/index.vue", "components/TheGreeting.vue"]);
    assert.deepEqual(diagnosticsFor(result, "pages/index.vue"), []);
    assert.deepEqual(diagnosticsFor(result, "components/TheGreeting.vue"), []);
    assert.equal(result.errorCount, 0, JSON.stringify(result.files, null, 2));
  });

  // Real errors are caught: the auto-imported `useCounter` is strongly typed,
  // so a wrong-typed call must surface as TS2345 — proving it resolves to real
  // types from `.nuxt/imports.d.ts`, not an `any` stub.
  it("catches a wrong-typed auto-import call (TS2345)", () => {
    const result = runVizeCheck(["pages/broken.vue"]);
    const diagnostics = diagnosticsFor(result, "pages/broken.vue");
    assert.equal(result.errorCount, 1, JSON.stringify(result.files, null, 2));
    assert.ok(
      diagnostics.some((d) => /\[TS2345\]/.test(d) && /not assignable to parameter/.test(d)),
      `expected a TS2345 in ${JSON.stringify(diagnostics)}`,
    );
  });
});

describe(`${app.name} matches the Vue toolchain`, () => {
  before(requireToolchainParityBinaries);

  // Same diagnostic surface as vue-tsc across the whole project: vue-tsc, with
  // Nuxt's generated `.nuxt/tsconfig.json` + ambient `.d.ts`, reports exactly
  // the intentional error and nothing else; `vize check` must agree file-for-file.
  it("reports the same diagnostic surface as vue-tsc", () => {
    assertVueTscDiagnosticSurface(app, { expectErrors: true });
  });

  it("compiles with @vue/compiler-sfc", () => {
    assertOfficialCompilerAccepts(app);
  });
});

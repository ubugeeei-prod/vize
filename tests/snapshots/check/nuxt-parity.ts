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

// vue-tsc 3.3.14 reports the same three rows for `pages/broken-types.vue`.
const BROKEN_TYPES = [
  "error:5:7 [TS2322] Type '\"green\"' is not assignable to type 'ColorName'.",
  "error:6:28 [TS2322] Type 'string' is not assignable to type 'number'.",
  "error:8:7 [TS2322] Type 'Ref<number, number>' is not assignable to type 'Ref<string, string>'.\nType 'number' is not assignable to type 'string'.",
];

// vue-tsc 3.3.14 reports the same two rows for `pages/lazy-components.vue`.
const LAZY_COMPONENTS = [
  "error:6:16 [TS2304] Cannot find name 'UnwrapRef'.",
  "error:10:21 [TS2322] Type 'number' is not assignable to type 'string'. (source: <LazyTheGreeting :name=\"5\" />; binding: 'name')",
];

// Serialized island wrappers must retain their SFC props, including when lazy.
const ISLAND_COMPONENTS = [
  "error:3:17 [TS2322] Type 'number' is not assignable to type 'string'. (source: <ServerProps :title=\"123\" />; binding: 'title')",
  "error:4:4 [TS2345] Missing required prop `title`. (source: <ServerProps />)",
  "error:6:21 [TS2322] Type 'number' is not assignable to type 'string'. (source: <LazyServerProps :title=\"123\" />; binding: 'title')",
  "error:7:4 [TS2345] Missing required prop `title`. (source: <LazyServerProps />)",
];

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

  // Auto-imported types reach the program as re-exports inside a global
  // augmentation of `.nuxt/imports.d.ts`. A user type, a generic user type and
  // Vue's `Ref` with both of its type parameters must all be the real types.
  it("no false positives on auto-imported types", () => {
    const result = runVizeCheck(["pages/types.vue"]);
    assert.deepEqual(diagnosticsFor(result, "pages/types.vue"), []);
    assert.equal(result.errorCount, 0, JSON.stringify(result.files, null, 2));
  });

  // Real errors are caught: a value that does not fit the auto-imported type is
  // TS2322 against that type. An unresolved name would be TS2304 instead, and a
  // type that collapsed to `any` would report nothing.
  it("catches wrong values for auto-imported types (TS2322)", () => {
    const result = runVizeCheck(["pages/broken-types.vue"]);
    assert.deepEqual(diagnosticsFor(result, "pages/broken-types.vue"), BROKEN_TYPES);
    assert.equal(result.errorCount, 3, JSON.stringify(result.files, null, 2));
  });

  // Nuxt 4 types `<LazyTheGreeting>` through `LazyComponent`, an alias local to
  // `.nuxt/types/components.d.ts`, so its props stay checked (TS2322). A type
  // the manifest does not re-export, such as `UnwrapRef`, is not in scope.
  it("checks Nuxt 4 lazy components and invents no auto-imported type", () => {
    const result = runVizeCheck(["pages/lazy-components.vue"]);
    assert.deepEqual(diagnosticsFor(result, "pages/lazy-components.vue"), LAZY_COMPONENTS);
    assert.equal(result.errorCount, 2, JSON.stringify(result.files, null, 2));
  });

  it("checks wrong and missing props through serialized island and lazy wrappers", () => {
    const result = runVizeCheck(["pages/island-components.vue"]);
    assert.deepEqual(diagnosticsFor(result, "pages/island-components.vue"), ISLAND_COMPONENTS);
    assert.equal(result.errorCount, 4, JSON.stringify(result.files, null, 2));
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

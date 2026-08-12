import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { before, describe, it } from "node:test";
import { fileURLToPath } from "node:url";
import { CORSA_BIN, VIZE_BIN, requireVizeAndCorsaBins } from "../../_helpers/apps.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const TESTS_ROOT = path.resolve(__dirname, "..", "..");
const PROJECTS_DIR = path.join(TESTS_ROOT, "_fixtures", "_projects");
const DECLARED_DIR = path.join(PROJECTS_DIR, "nuxt-template-globals");
const UNDECLARED_DIR = path.join(PROJECTS_DIR, "nuxt-i18n-undeclared-globals");
const PATTERNS = ["pages/**/*.vue"];

// One diagnostic reduced to what both tools state identically: the authored
// file, the TypeScript code, and the authored line/column. The rendered type
// inside a TS2339 message is the one part the two tools word differently —
// `vue-tsc` prints the structural template context it synthesizes, Vize prints
// Vue's `ComponentPublicInstance` — so message text is asserted separately,
// against Vize's own complete output.
interface DiagnosticRow {
  code: string;
  column: number;
  file: string;
  line: number;
}

function resolveVueTsc(): string {
  const candidates = [
    path.join(TESTS_ROOT, "node_modules", ".bin", "vue-tsc"),
    path.join(TESTS_ROOT, "..", "node_modules", ".bin", "vue-tsc"),
  ];
  const found = candidates.find((candidate) => fs.existsSync(candidate));
  assert.ok(found, "vue-tsc binary should exist. Install JS dependencies with `vp install`.");
  return found;
}

function runVueTsc(cwd: string): DiagnosticRow[] {
  const result = spawnSync(
    resolveVueTsc(),
    ["--noEmit", "--pretty", "false", "-p", "tsconfig.json"],
    {
      cwd,
      encoding: "utf8",
      env: { ...process.env, LANG: "C", LC_ALL: "C" },
      maxBuffer: 64 * 1024 * 1024,
      timeout: 180_000,
    },
  );
  if (result.error != null) {
    throw result.error;
  }
  const output = `${result.stdout}${result.stderr}`;
  const rows: DiagnosticRow[] = [];
  for (const line of output.split("\n")) {
    const match = /^(.+?)\((\d+),(\d+)\): error (TS\d+): /.exec(line);
    if (match == null) {
      continue;
    }
    rows.push({
      code: match[4],
      column: Number(match[3]),
      file: match[1].split(path.sep).join("/"),
      line: Number(match[2]),
    });
  }
  return sortRows(rows);
}

interface VizeCheckJson {
  errorCount: number;
  fileCount: number;
  files: Array<{ diagnostics: string[]; file: string }>;
}

function runVize(cwd: string): VizeCheckJson {
  let stdout: string;
  try {
    stdout = execFileSync(
      VIZE_BIN,
      ["check", ...PATTERNS, "--format", "json", "--quiet", "--corsa-path", CORSA_BIN],
      { cwd, maxBuffer: 64 * 1024 * 1024, timeout: 180_000 },
    ).toString();
  } catch (error) {
    const err = error as { status?: number; stderr?: Buffer; stdout?: Buffer };
    if (err.status === 1 && err.stdout) {
      stdout = err.stdout.toString();
    } else {
      throw new Error(`vize check crashed (exit ${err.status}): ${err.stderr?.toString() ?? ""}`);
    }
  }
  return JSON.parse(stdout) as VizeCheckJson;
}

function vizeRows(result: VizeCheckJson): DiagnosticRow[] {
  const rows: DiagnosticRow[] = [];
  for (const entry of result.files) {
    for (const diagnostic of entry.diagnostics) {
      const match = /^error:(\d+):(\d+) \[(TS\d+)\] /.exec(diagnostic);
      assert.ok(match, `unexpected diagnostic shape: ${diagnostic}`);
      rows.push({
        code: match[3],
        column: Number(match[2]),
        file: entry.file,
        line: Number(match[1]),
      });
    }
  }
  return sortRows(rows);
}

function sortRows(rows: DiagnosticRow[]): DiagnosticRow[] {
  return [...rows].sort(
    (a, b) =>
      a.file.localeCompare(b.file) ||
      a.line - b.line ||
      a.column - b.column ||
      a.code.localeCompare(b.code),
  );
}

function diagnosticsFor(result: VizeCheckJson, file: string): string[] {
  return result.files.find((entry) => entry.file === file)?.diagnostics ?? [];
}

// The rendered instance type inside Vize's TS2339 text. Pinned so the message
// keeps naming Vue's own public type and never leaks a generated helper name.
//
// Why the two tools word this differently, and why that is left alone: Vue
// Language Tools builds an anonymous object type per component that inlines the
// setup bindings and then spreads the public-instance members, so its message
// names that structural type and the component's own bindings appear inside it.
// Vize resolves a template instance global on
// `import('vue').ComponentPublicInstance`, which already merges
// `ComponentCustomProperties` (the interface Nuxt's generated types and
// packages like vue-i18n augment), so the same missing property is reported
// against Vue's own public type. Matching vue-tsc's wording would mean
// synthesizing and naming a per-component structural clone of the instance
// purely so an error message can quote it: it changes nothing about which names
// resolve, and it would put a generated per-component type name in front of
// users. This rendering applies to every TS2339 Vize reports on an undeclared
// `$`-prefixed template global (#913). The code, severity, and authored range
// agree, which is exactly what the vue-tsc comparisons above assert.
const INSTANCE_TYPE =
  "ComponentPublicInstance<{}, {}, {}, {}, {}, {}, {}, {}, false, ComponentOptionsBase<any, any, " +
  "any, any, any, any, any, any, any, {}, {}, string, {}, {}, {}, string, ComponentProvideOptions>," +
  " ... 4 more ..., any>";

function missingProperty(name: string): string {
  return `Property '${name}' does not exist on type '${INSTANCE_TYPE}'.`;
}

const WRONG_ARGUMENT = "Argument of type 'number' is not assignable to parameter of type 'string'.";

describe("nuxt template globals check (generated declarations are authoritative)", () => {
  before(requireVizeAndCorsaBins);

  // A Nuxt project whose generated `.nuxt` graph declares the auto-imports and
  // the `$`-prefixed template globals: the issue's reproduction page must be
  // clean, wrong-typed calls must surface through the real declared signatures,
  // and a name nothing declares must still be reported.
  it("matches vue-tsc exactly when the generated graph declares the globals", () => {
    const vize = runVize(DECLARED_DIR);

    assert.deepEqual(vizeRows(vize), runVueTsc(DECLARED_DIR));

    assert.deepEqual(diagnosticsFor(vize, "pages/index.vue"), []);
    assert.deepEqual(diagnosticsFor(vize, "pages/wrong-argument.vue"), [
      `error:7:31 [TS2345] ${WRONG_ARGUMENT}`,
      `error:11:29 [TS2345] ${WRONG_ARGUMENT}`,
      `error:11:45 [TS2345] ${WRONG_ARGUMENT}`,
    ]);
    assert.deepEqual(diagnosticsFor(vize, "pages/undeclared-global.vue"), [
      "error:7:15 [TS2304] Cannot find name 'notDeclaredAnywhere'.",
      `error:11:9 [TS2339] ${missingProperty("$missing")}`,
      `error:12:9 [TS2339] ${missingProperty("$missing")}`,
      `error:13:14 [TS2339] ${missingProperty("$missing")}`,
    ]);
    assert.equal(vize.errorCount, 7, JSON.stringify(vize.files, null, 2));
    assert.equal(vize.fileCount, 3);
  });

  // An auto-imported `useI18n` is not a declaration of `$t`. When the generated
  // graph never augments `ComponentCustomProperties`, `vue-tsc` reports every
  // authored `$t`, and Vize must report the same code at the same authored
  // ranges instead of inventing the global from the composable's presence.
  it("reports every authored $t when the generated graph declares none", () => {
    assert.ok(
      !fs.existsSync(repairPath()),
      "the undeclared fixture must stay undeclared; a repair file was left behind",
    );

    const vize = runVize(UNDECLARED_DIR);

    assert.deepEqual(vizeRows(vize), runVueTsc(UNDECLARED_DIR));
    assert.deepEqual(diagnosticsFor(vize, "pages/index.vue"), [
      `error:11:9 [TS2339] ${missingProperty("$t")}`,
      `error:12:14 [TS2339] ${missingProperty("$t")}`,
    ]);
    assert.equal(vize.errorCount, 2, JSON.stringify(vize.files, null, 2));
    assert.equal(vize.fileCount, 1);
  });

  // Repaired variant: adding the declaration the module would generate clears
  // the diagnostics in both tools, with no leftover report from either.
  it("goes clean in both tools once the module declares the global", () => {
    const repair = repairPath();
    fs.mkdirSync(path.dirname(repair), { recursive: true });
    fs.writeFileSync(repair, REPAIR_DECLARATION);
    try {
      const vize = runVize(UNDECLARED_DIR);
      assert.deepEqual(runVueTsc(UNDECLARED_DIR), []);
      assert.deepEqual(vizeRows(vize), []);
      assert.equal(vize.errorCount, 0, JSON.stringify(vize.files, null, 2));
    } finally {
      fs.rmSync(repair, { force: true });
    }
  });
});

function repairPath(): string {
  return path.join(UNDECLARED_DIR, ".nuxt", "types", "i18n-plugin.d.ts");
}

const REPAIR_DECLARATION = `// Written by the repaired-variant test, removed again when it finishes.
declare module "vue" {
  interface ComponentCustomProperties {
    $t: (key: string) => string;
  }
}

export {};
`;

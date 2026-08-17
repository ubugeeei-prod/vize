import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { before, describe, it } from "node:test";
import { fileURLToPath } from "node:url";
import { CORSA_BIN, VIZE_BIN, requireVizeAndCorsaBins } from "../../_helpers/apps.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const TESTS_ROOT = path.resolve(__dirname, "..", "..");
const PROJECT_DIR = path.join(
  TESTS_ROOT,
  "_fixtures",
  "_projects",
  "nuxt-vue-module-augmentations",
);
const PATTERNS = ["pages/**/*.vue"];

// One diagnostic reduced to what both tools state identically: the authored
// file, the TypeScript code, and the authored line/column. The rendered type
// inside a TS2339 message is the one part the two tools word differently —
// `vue-tsc` prints the `ComponentPublicInstance` it synthesizes, Vize prints its
// strict template context — so message text is asserted separately, against
// Vize's own complete output.
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

const STRICT_TEMPLATE_CONTEXT_TYPE = "__VizeStrictTemplateContext";
const WRONG_ARGUMENT = "Argument of type 'number' is not assignable to parameter of type 'string'.";

function missingProperty(name: string): string {
  return `Property '${name}' does not exist on type '${STRICT_TEMPLATE_CONTEXT_TYPE}'.`;
}

// A `declare module "vue"` augmentation binds to whichever `vue` its own file
// resolves — and a package with separate `import`/`require` type entries offers
// two. Mirroring a project `.d.ts` into the virtual project under a `.d.cts`
// name flipped it to the `require` entry, so `ComponentCustomProperties` split
// in two and the members declared on the far side of the split stopped reaching
// the `ComponentPublicInstance` the generated template context reads.
describe("vue module augmentations reach the template context", () => {
  before(requireVizeAndCorsaBins);

  it("matches vue-tsc exactly across generated, project, and package augmentations", () => {
    const vize = runVize(PROJECT_DIR);

    assert.deepEqual(vizeRows(vize), runVueTsc(PROJECT_DIR));

    // Every global on this page is declared by some `ComponentCustomProperties`
    // augmentation: the package's (`$t`), the generated `.nuxt` types' (`$shout`),
    // and the project's own (`$local`).
    assert.deepEqual(diagnosticsFor(vize, "pages/index.vue"), []);
    // The declared signatures are real, not widened stand-ins: a wrong-typed
    // call is the same TS2345 at the same authored column in both tools.
    assert.deepEqual(diagnosticsFor(vize, "pages/wrong-argument.vue"), [
      `error:5:16 [TS2345] ${WRONG_ARGUMENT}`,
      `error:6:16 [TS2345] ${WRONG_ARGUMENT}`,
    ]);
    // Negative control: a global nothing declares still reports once per use.
    assert.deepEqual(diagnosticsFor(vize, "pages/undeclared-global.vue"), [
      `error:4:9 [TS2339] ${missingProperty("$missing")}`,
      `error:5:9 [TS2339] ${missingProperty("$missing")}`,
      `error:6:14 [TS2339] ${missingProperty("$missing")}`,
    ]);
    assert.equal(vize.errorCount, 5, JSON.stringify(vize.files, null, 2));
    assert.equal(vize.fileCount, 3);
  });
});

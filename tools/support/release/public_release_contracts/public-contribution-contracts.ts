import assert from "node:assert/strict";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { arg, readJson, type PackageManifest } from "./utils.ts";

interface FixtureCase {
  id: string;
  inputs: Array<{ file: string; source: string }>;
  diagnostics: unknown[];
}
interface FixturePack {
  cases: FixtureCase[];
  checkerOptions?: { optionsApi?: boolean };
}
interface ContributionRow {
  pack: string;
  id: string;
  syntax: string | null;
  exit: number | null;
  expected: unknown[];
  report: Record<string, unknown>;
  actual: unknown[] | null;
}

const workspace = path.resolve(arg(0));
const fixtures = arg(1);
const out = arg(2);
const require = createRequire(path.join(workspace, "package.json"));
const native = require("@vizejs/native") as {
  generateVueLanguageCode(
    source: string,
    options: { filename: string; optionsApi: boolean },
  ): string;
};
const { babelParse } = require("vue/compiler-sfc") as {
  babelParse: (source: string, options: { sourceType: string; plugins: string[] }) => unknown;
};
const rows: ContributionRow[] = [];
for (const packName of ["empty-v-slot-expression", "options-api-reserved-component-reference"]) {
  const packRoot = path.join(fixtures, packName);
  const pack = readJson<FixturePack>(path.join(packRoot, "cases.json.txt"));
  for (const entry of pack.cases) {
    const directory = path.join(workspace, "contribution-checks", packName, entry.id);
    fs.mkdirSync(directory, { recursive: true });
    for (const input of entry.inputs) {
      const destination = path.join(directory, input.file);
      fs.mkdirSync(path.dirname(destination), { recursive: true });
      fs.writeFileSync(destination, fs.readFileSync(path.join(packRoot, input.source + ".txt")));
    }
    const authored = entry.inputs.find(
      (input) =>
        input.file.endsWith(".vue") &&
        !input.file.endsWith("/Box.vue") &&
        !input.file.endsWith("/Page.vue"),
    );
    assert.ok(authored);
    const source = fs.readFileSync(path.join(packRoot, authored.source + ".txt"), "utf8");
    const projection = JSON.parse(
      native.generateVueLanguageCode(source, {
        filename: path.join(directory, authored.file),
        optionsApi: pack.checkerOptions?.optionsApi ?? false,
      }),
    ) as { text: string };
    let syntax: string | null = null;
    try {
      babelParse(projection.text, { sourceType: "module", plugins: ["typescript"] });
    } catch (error) {
      syntax = error instanceof Error ? error.message : String(error);
    }
    const result = spawnSync(
      process.execPath,
      [
        path.join(workspace, "node_modules/vize/bin/vize"),
        "check",
        "--no-config",
        "--format",
        "json",
        "--tsconfig",
        "tsconfig.json",
      ],
      { cwd: directory, encoding: "utf8", timeout: 90000 },
    );
    let report: Record<string, unknown>;
    try {
      report = JSON.parse(result.stdout) as Record<string, unknown>;
    } catch {
      report = { stdout: result.stdout, stderr: result.stderr };
    }
    const actual = Array.isArray(report.diagnostics)
      ? report.diagnostics
      : Array.isArray(report.files)
        ? report.files.flatMap((file: unknown) => {
            const value = file as { diagnostics?: unknown[] };
            return value.diagnostics ?? [];
          })
        : null;
    rows.push({
      pack: packName,
      id: entry.id,
      syntax,
      exit: result.status,
      expected: entry.diagnostics,
      report,
      actual,
    });
  }
}
const manifest = require("@vizejs/native/package.json") as PackageManifest;
fs.writeFileSync(out, JSON.stringify({ native: manifest.version, rows }, null, 2) + "\n");
console.log(
  JSON.stringify(
    rows.map((row) => ({
      id: row.id,
      syntax: row.syntax,
      exit: row.exit,
      actual: row.actual,
      reportFields: Object.keys(row.report),
    })),
  ),
);
assert.ok(rows.every((row) => row.syntax === null));
for (const row of rows) {
  assert.equal(row.exit, row.expected.length ? 1 : 0, row.id);
  assert.ok(row.actual, "structured production diagnostics required");
  assert.equal(row.actual.length, row.expected.length, row.id);
}

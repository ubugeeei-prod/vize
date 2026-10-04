import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { type PinnedFixtureWorkspace } from "../../../_helpers/realworld-patch.ts";
import { type CommandResult, type VizeCheckResult } from "../../../_helpers/realworld-typecheck.ts";
import { offsetToPosition } from "../../../tooling/support/lsp/assertions.ts";
import type { LspDiagnostic } from "../../../tooling/support/lsp/protocol.ts";
import { preparePublishedProvider } from "./provider.ts";

export const ROUTER_REVISION = "feed382f2fbfe38b3892ea780f5aea3d5459986a";
export const TYPEOF_REVISION = "071f1969be1348e797a55d0d8afb72b8068154dc";
export const PLAYGROUND = "packages/playground-file-based";
export const PAGE_PATH = `${PLAYGROUND}/src/pages/users/[userId=int].vue`;
export const ROUTES_PATH = `${PLAYGROUND}/src/routes.d.ts`;
export const PLUGIN = "vue-router/volar/sfc-typed-router";
export const PAGE_NAME_TYPE =
  "import('vue-router/auto-routes')._RouteNamesForFilePath<'src/pages/users/[userId=int].vue'>";

export type ErrorSpec = { code: number; needle: string; token: string; message?: RegExp };
type DiagnosticRow = { file: string; line: number; column: number; code: number; message: string };
export type Config = {
  enabled?: boolean;
  compilerRoot?: string | false;
  pluginRoot?: string;
  inherited?: boolean;
  projectRootMap?: boolean;
};

export function control(name: string): string {
  return fs.readFileSync(path.join(path.dirname(fileURLToPath(import.meta.url)), name), "utf8");
}

export function json(value: unknown): string {
  return `${JSON.stringify(value, null, 2)}\n`;
}

export function sha256(bytes: string): string {
  return createHash("sha256").update(bytes).digest("hex");
}

/** Original pinned page/map, with the distinct verified official published provider. */
export function prepareProvider(fixture: PinnedFixtureWorkspace) {
  assert.equal(fixture.entry.revision, ROUTER_REVISION);
  const provider = preparePublishedProvider(fixture);
  const evidence = {
    ...provider.evidence,
    page: sha256(fixture.read(PAGE_PATH)),
    generatedRoutes: sha256(fixture.read(ROUTES_PATH)),
  };
  provider.record({
    evidence,
    originalPage: fixture.read(PAGE_PATH),
    originalGeneratedRoutes: fixture.read(ROUTES_PATH),
  });
  return { ...provider, evidence };
}

export function configure(
  fixture: PinnedFixtureWorkspace,
  corsaPath: string,
  sourcePath = PAGE_PATH,
  options: Config = {},
): void {
  const compilerOptions: Record<string, unknown> = {
    allowJs: true,
    checkJs: true,
    lib: ["ES2022", "DOM"],
    module: "ESNext",
    moduleResolution: "bundler",
    noEmit: true,
    skipLibCheck: true,
    strict: true,
    target: "ES2022",
    types: ["vue-router/auto-routes"],
  };
  if (options.compilerRoot !== false) compilerOptions.rootDir = options.compilerRoot ?? PLAYGROUND;
  const plugin =
    options.pluginRoot === undefined
      ? PLUGIN
      : { name: PLUGIN, options: { rootDir: fixture.resolve(options.pluginRoot) } };
  const config: Record<string, unknown> = {
    compilerOptions,
    include: [sourcePath, ROUTES_PATH],
    vueCompilerOptions: { plugins: options.enabled === false ? [] : [plugin] },
  };
  if (options.inherited) {
    fixture.write(
      `${PLAYGROUND}/compiler-options.json`,
      json({ compilerOptions: { rootDir: "." } }),
    );
    delete compilerOptions.rootDir;
    config.extends = `./${PLAYGROUND}/compiler-options.json`;
  }
  if (options.projectRootMap) {
    fixture.write("project-root-map.d.ts", control("project-root-map.d.ts"));
    config.include = [sourcePath, ROUTES_PATH, "project-root-map.d.ts"];
  }
  fixture.write("tsconfig.json", json(config));
  fixture.write(
    "vize.config.json",
    json({
      lsp: { completion: true, editor: true, hover: true, lint: false, typecheck: true },
      typeChecker: { corsaPath },
    }),
  );
}

export function sourceRange(source: string, expected: ErrorSpec) {
  const anchor = source.indexOf(expected.needle);
  assert.notEqual(anchor, -1, expected.needle);
  assert.equal(
    source.indexOf(expected.needle, anchor + expected.needle.length),
    -1,
    expected.needle,
  );
  const within = expected.needle.lastIndexOf(expected.token);
  assert.notEqual(within, -1, expected.token);
  const start = offsetToPosition(source, anchor + within);
  return { start, end: { line: start.line, character: start.character + expected.token.length } };
}

function message(value: string): string {
  return value.replace(/\s+/g, " ").trim();
}

function vueRows(result: CommandResult): DiagnosticRow[] {
  const rows: DiagnosticRow[] = [];
  for (const line of `${result.stdout}\n${result.stderr}`.split(/\r?\n/)) {
    const header = /^(.*?)\((\d+),(\d+)\): error TS(\d+): (.*)$/.exec(line);
    if (header) {
      rows.push({
        file: header[1].replaceAll("\\", "/"),
        line: Number(header[2]),
        column: Number(header[3]),
        code: Number(header[4]),
        message: header[5],
      });
    } else if (/error TS\d+:/.test(line)) {
      assert.fail(`unparsed oracle diagnostic: ${line}`);
    } else if (/^\s+\S/.test(line) && rows.length) {
      rows.at(-1)!.message += ` ${line.trim()}`;
    }
  }
  return rows.map((row) => ({ ...row, message: message(row.message) }));
}

export function assertCli(
  vize: VizeCheckResult,
  oracle: CommandResult,
  source: string,
  errors: ErrorSpec[],
  sourcePath = PAGE_PATH,
): DiagnosticRow[] {
  const expected = vueRows(oracle);
  assert.equal(oracle.status, errors.length === 0 ? 0 : 2, oracle.stderr || oracle.stdout);
  assert.equal(expected.length, errors.length, oracle.stdout);
  const actual = vize.report.files.flatMap((file) =>
    file.diagnostics.map((diagnostic) => {
      const parts = /^error:(\d+):(\d+) \[TS(\d+)\] ([\s\S]*)$/.exec(diagnostic);
      assert.ok(parts, diagnostic);
      return {
        file: file.file,
        line: Number(parts[1]),
        column: Number(parts[2]),
        code: Number(parts[3]),
        message: message(parts[4]),
      };
    }),
  );
  assert.deepEqual(actual, expected, json(vize.report));
  assert.equal(vize.status, errors.length === 0 ? 0 : 1, vize.stderr || vize.stdout);
  assert.equal(vize.report.errorCount, errors.length);
  assert.equal(vize.report.warningCount, 0);
  assert.equal(vize.report.fileCount, 1);
  assert.deepEqual(
    vize.report.files.map((file) => file.file),
    [sourcePath],
  );
  assert.equal(vize.report.programs.length, 1, json(vize.report.programs));
  const [program] = vize.report.programs;
  assert.equal(program.root, ".");
  assert.equal(program.tsconfig, "tsconfig.json");
  assert.ok(program.files.includes(sourcePath), json(program));
  assert.ok(program.files.includes(ROUTES_PATH), json(program));
  expected.forEach((row, index) => {
    const error = errors[index];
    const range = sourceRange(source, error);
    assert.equal(row.file, sourcePath);
    assert.equal(row.code, error.code);
    assert.equal(row.line, range.start.line + 1);
    assert.equal(row.column, range.start.character + 1);
    if (error.message) assert.match(row.message, error.message);
  });
  return expected;
}

export function assertEditor(
  diagnostics: LspDiagnostic[],
  rows: DiagnosticRow[],
  source: string,
  errors: ErrorSpec[],
): void {
  assert.deepEqual(
    diagnostics.map((diagnostic) => ({
      ...diagnostic,
      code: String(diagnostic.code).replace(/^TS/, ""),
      message: message(diagnostic.message ?? ""),
    })),
    rows.map((row, index) => ({
      code: String(row.code),
      message: row.message,
      source: "vize/types",
      severity: 1,
      range: sourceRange(source, errors[index]),
    })),
  );
}

export function typeofReference(source: string): string {
  const anchor = "typeof useRoute>";
  assert.equal(source.split(anchor).length, 2, "the published typeof arm needs one type query");
  return source.replace(anchor, `typeof useRoute<${PAGE_NAME_TYPE}>>`);
}

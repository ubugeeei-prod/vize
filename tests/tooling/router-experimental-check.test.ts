import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { test } from "node:test";

import { repoRoot } from "../_helpers/realworld-patch.ts";
import { resolveTsgoBinary } from "../_helpers/realworld-typecheck.ts";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import {
  experimentalRouterPage,
  prepareExperimentalRouterProject,
} from "./support/router-experimental-provider.ts";

type ProcessRecord = {
  label: string;
  command: string;
  args: string[];
  status: number | null;
  signal: string | null;
  error: string | null;
  stdout: string;
  stderr: string;
  elapsedMs: number;
};

const unknownNative = `Object literal may only specify known properties, and 'missing' does not exist in type '{ id?: "date" | DefinePagePathParamOptions | keyof ParamParsers_Native | undefined; }'.`;
const unknownReference = `Object literal may only specify known properties, and 'missing' does not exist in type '{ id?: keyof ParamParsers_Native | "date" | DefinePagePathParamOptions | undefined; }'.`;
const dateError =
  "Type 'Date | undefined' is not assignable to type 'string | undefined'.\nType 'Date' is not assignable to type 'string'.";
const numberError = "Property 'toUpperCase' does not exist on type 'number'.";

function point(source: string, needle: string, token = needle) {
  const start = source.indexOf(needle);
  assert.notEqual(start, -1, needle);
  assert.equal(source.indexOf(needle, start + 1), -1, `unique anchor: ${needle}`);
  const offset = start + needle.indexOf(token);
  const lines = source.slice(0, offset).split("\n");
  return { line: lines.length, column: lines.at(-1)!.length + 1 };
}

function expectedReport(root: string, diagnostics: string[]) {
  return {
    files: [
      { file: "src/pages/index.vue", diagnostics: [] },
      { file: experimentalRouterPage, diagnostics },
      { file: "src/params/date.ts", diagnostics: [] },
      { file: "src/router/index.ts", diagnostics: [] },
    ],
    programs: [
      {
        root: ".",
        tsconfig: "tsconfig.json",
        compilerOptions: {
          module: "ESNext",
          moduleResolution: "Bundler",
          noEmit: true,
          rootDir: fs.realpathSync(root),
          skipLibCheck: true,
          strict: true,
          target: "ESNext",
          types: ["vue-router/auto-routes"],
        },
        files: [
          "node_modules/vue-router/vue-router-auto-routes.d.mts",
          "src/pages/index.vue",
          experimentalRouterPage,
          "src/params/date.ts",
          "src/router/index.ts",
          "typed-router.d.ts",
        ],
      },
    ],
    errorCount: diagnostics.length,
    warningCount: 0,
    fileCount: 4,
  };
}

await test("published experimental page imports preserve complete native/reference check reports", async () => {
  assert.equal(
    process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD,
    "1",
    "require exact source build custody",
  );
  const requestedBinary = process.env.VIZE_TEST_BIN ?? process.env.VIZE_LSP_BIN;
  assert.ok(requestedBinary, "source CLI required; no installed fallback");
  const [binary] = resolveVizeLaunchCommand(undefined, requestedBinary, {
    required: true,
    repoRoot,
  });
  const corsa = resolveTsgoBinary();
  const output = path.join(repoRoot, "target/differential/experimental-router-check");
  const fixture = await prepareExperimentalRouterProject(output);
  const records: ProcessRecord[] = [];
  const run = (label: string, command: string, args: string[]) => {
    const started = performance.now();
    const result = spawnSync(command, args, {
      cwd: fixture.root,
      encoding: "utf8",
      timeout: 120_000,
      maxBuffer: 16 * 1024 * 1024,
      env: { ...process.env, LANG: "C", LC_ALL: "C" },
    });
    const record: ProcessRecord = {
      label,
      command,
      args,
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      stdout: result.stdout ?? "",
      stderr: result.stderr ?? "",
      elapsedMs: performance.now() - started,
    };
    records.push(record);
    // Persist complete process output before evaluating any oracle.
    fs.writeFileSync(path.join(output, "processes.json"), `${JSON.stringify(records, null, 2)}\n`);
    assert.equal(record.error, null, label);
    assert.equal(record.signal, null, label);
    return record;
  };
  try {
    for (const [variant, newline] of [
      ["lf", "\n"],
      ["crlf", "\r\n"],
    ] as const) {
      const valid = fixture.pageSource
        .replace("import { useRoute }", `// 日本語と🍣: "quoted"\nimport { useRoute }`)
        .replaceAll("\n", newline);
      const cases = [
        { name: "valid", source: valid, errors: [] },
        {
          name: "unknown-path",
          source: valid.replace("params: {", "params: { path: { missing: 'int' },"),
          errors: [
            {
              code: 2353,
              needle: "missing: 'int'",
              token: "missing",
              native: unknownNative,
              reference: unknownReference,
            },
          ],
        },
        {
          name: "date-parser",
          source: valid.replace("const at: Date | undefined", "const at: string | undefined"),
          errors: [
            {
              code: 2322,
              needle: "const at: string",
              token: "at",
              native: dateError,
              reference: dateError.replace("\nType", "\n  Type"),
            },
          ],
        },
        {
          name: "template-method",
          source: valid.replace("route.params.page.toFixed()", "route.params.page.toUpperCase()"),
          errors: [
            {
              code: 2339,
              needle: "route.params.page.toUpperCase()",
              token: "toUpperCase",
              native: numberError,
              reference: numberError,
              template: true,
            },
          ],
        },
        { name: "repaired", source: valid, errors: [] },
      ];
      for (const control of cases) {
        fs.writeFileSync(fixture.pagePath, control.source);
        const label = `${variant}/${control.name}`;
        const native = run(`${label}/native`, binary, [
          "check",
          "src",
          "--tsconfig",
          "tsconfig.json",
          "--format",
          "json",
          "--quiet",
          "--corsa-path",
          corsa,
        ]);
        const reference = run(`${label}/reference`, process.execPath, [
          path.join(fixture.root, "node_modules/vue-tsc/bin/vue-tsc.js"),
          "--noEmit",
          "--pretty",
          "false",
          "-p",
          "tsconfig.json",
        ]);
        const diagnosticStrings = control.errors.map((error) => {
          const location = point(control.source, error.needle, error.token);
          const line = control.source.split(/\r?\n/u)[location.line - 1];
          const suffix = "template" in error && error.template ? ` (source: ${line})` : "";
          return `error:${location.line}:${location.column} [TS${error.code}] ${error.native}${suffix}`;
        });
        assert.equal(native.status, control.errors.length ? 1 : 0, label);
        assert.equal(native.stderr, "", label);
        assert.equal(
          native.stdout,
          `${JSON.stringify(expectedReport(fixture.root, diagnosticStrings), null, 2)}\n`,
          label,
        );
        const referenceOutput = control.errors
          .map((error) => {
            const location = point(control.source, error.needle, error.token);
            return `${experimentalRouterPage}(${location.line},${location.column}): error TS${error.code}: ${error.reference}\n`;
          })
          .join("");
        assert.equal(reference.status, control.errors.length ? 2 : 0, label);
        assert.equal(reference.stderr, "", label);
        assert.equal(reference.stdout, referenceOutput, label);
        assert.equal(
          fs.readFileSync(path.join(fixture.root, "typed-router.d.ts"), "utf8"),
          fixture.generatedDeclaration,
          "authored mutation must not regenerate away its error",
        );
      }
    }
  } finally {
    fixture.restore();
    fixture.cleanup();
  }
});

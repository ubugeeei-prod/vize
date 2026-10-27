// Preserve the old public inventory while independently testing the three
// hidden declarations affected by TypeScript-compatible wildcard traversal.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import type { Command } from "./javascript-workspace-cli.ts";
import type { CheckReport } from "./javascript-workspace-config.ts";
import { parseJson } from "../../json-value.ts";
import { lexical, save } from "./javascript-workspace-project.ts";

export const HIDDEN_NUXT_DECLARATIONS = [
  ".nuxt/types/nitro-nuxt.d.ts",
  ".nuxt/types/nitro.d.ts",
  ".nuxt/types/vue-shim.d.ts",
];

/**
 * The immutable public contract records the former wildcard traversal. Only
 * these three program roots change: two unreferenced siblings and the pure
 * nitro path manifest. Referenced ambient declarations remain program roots.
 * Callers must qualify the six real stock/native mutations before using this
 * transition; the historical whole packets are retained alongside the result.
 */
export function configuredHiddenMembership(original: CheckReport): CheckReport {
  const expected = structuredClone(original);
  assert.equal(expected.programs.length, 1);
  const program = expected.programs[0] as { tsconfig: string; files: string[] };
  assert.equal(program.tsconfig, "tsconfig.json");
  for (const name of HIDDEN_NUXT_DECLARATIONS)
    assert.equal(program.files.filter((file) => file === name).length, 1);
  program.files = program.files.filter((file) => !HIDDEN_NUXT_DECLARATIONS.includes(file));
  return expected;
}

export function probeHiddenMembership(options: {
  app: string;
  main: string;
  artifacts: string;
  binary: string;
  provider: string;
  vueTsc: string;
  stockPrograms: readonly { tsconfig: string }[];
  original: CheckReport;
  command: Command;
}): void {
  const { app, main, artifacts, binary, provider, vueTsc, stockPrograms, original, command } =
    options;
  const originalMain = fs.readFileSync(main, "utf8");
  assert.equal(originalMain.split("</script>").length, 2);
  const source = originalMain.replace(
    "</script>",
    "void __vizeHiddenProbe.toUpperCase();\n</script>",
  );
  const records: unknown[] = [];
  const failures: { id: string; error: string }[] = [];
  try {
    fs.writeFileSync(main, source);
    for (const [index, name] of HIDDEN_NUXT_DECLARATIONS.entries()) {
      const file = path.join(app, name);
      const bytes = fs.readFileSync(file);
      try {
        for (const type of ["number", "string"]) {
          const id = `hidden-membership-${index}-${type}`;
          fs.writeFileSync(
            file,
            `${bytes.toString()}\ndeclare global { const __vizeHiddenProbe: ${type}; }\nexport {};\n`,
          );
          const visible = name === ".nuxt/types/nitro.d.ts";
          const code = visible ? (type === "number" ? 2339 : undefined) : 2304;
          const message =
            code === 2339
              ? "Property 'toUpperCase' does not exist on type 'number'."
              : "Cannot find name '__vizeHiddenProbe'.";
          const needle = code === 2339 ? "toUpperCase" : "__vizeHiddenProbe";
          const before = source.slice(0, source.indexOf(needle)).split("\n");
          const line = before.length;
          const column = (before.at(-1)?.length ?? 0) + 1;
          const relative = path.relative(app, main).split(path.sep).join("/");
          const stock = stockPrograms.map((program, i) =>
            command(
              process.execPath,
              [vueTsc, "--noEmit", "--pretty", "false", "-p", program.tsconfig],
              app,
              artifacts,
              `${id}-stock-${i}`,
            ),
          );
          const actual = command(
            binary,
            ["check", "--format", "json", "--quiet", "--corsa-path", provider],
            app,
            artifacts,
            id,
          );
          const expected = configuredHiddenMembership(original);
          for (const value of expected.programs) {
            const program = value as { files: string[] };
            if (visible) program.files.push(name);
            program.files.sort(lexical);
          }
          if (code) {
            const item = expected.files.find((item) => item.file === relative);
            assert.ok(item);
            item.diagnostics.push(`error:${line}:${column} [TS${code}] ${message}`);
            expected.errorCount = 1;
          }
          const report = parseJson(actual.stdout);
          records.push({
            id,
            name,
            declaration: fs.readFileSync(file, "utf8"),
            source,
            stock,
            actual: report,
            expected,
            status: actual.status,
          });
          save(artifacts, "hidden-membership-mutations.json", records);
          try {
            assert.equal(
              stock.map((item) => item.stdout).join(""),
              code ? `${relative}(${line},${column}): error TS${code}: ${message}\n` : "",
            );
            assert.ok(stock.every((item) => item.stderr === ""));
            assert.equal(stock.filter((item) => item.status !== 0).length, code ? 1 : 0);
            assert.equal(actual.status, code ? 1 : 0);
            assert.deepEqual(report, expected);
          } catch (error) {
            failures.push({
              id,
              error: error instanceof Error ? (error.stack ?? error.message) : String(error),
            });
            save(artifacts, "hidden-membership-mismatches.json", failures);
          }
        }
      } finally {
        fs.writeFileSync(file, bytes);
      }
    }
  } finally {
    fs.writeFileSync(main, originalMain);
  }
  assert.deepEqual(failures, [], "all hidden declaration mutations must preserve typed visibility");
}

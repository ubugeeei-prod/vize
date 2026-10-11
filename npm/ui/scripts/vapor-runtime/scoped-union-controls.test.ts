import assert from "node:assert/strict";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

import { compileSfc } from "@vizejs/native";

const require = createRequire(import.meta.url);
const fixture = JSON.parse(
  readFileSync(
    new URL(
      "../../../../tests/_fixtures/differential/compiler/scoped-union-props/controls.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as {
  files: Record<string, string>;
  cases: { name: string; source: string }[];
  official: unknown[];
};

await test("whole unresolved, recursive, nested and foreign union compiler controls remain observable", () => {
  const directory = mkdtempSync(path.join(tmpdir(), "vize-union-controls-"));
  const official: unknown[] = [];
  const native: unknown[] = [];
  try {
    for (const [name, source] of Object.entries(fixture.files)) {
      writeFileSync(path.join(directory, name), source);
    }
    for (const input of fixture.cases) {
      const filename = path.join(directory, `${input.name}.vue`);
      writeFileSync(filename, input.source);
      for (const lane of ["ssr", "dom", "vapor"] as const) {
        // Keep complete product results, including the inherited acceptance
        // of unresolved unions and the flat resolver's foreign-alias gap.
        // Scoped completeness is metadata; this does not claim those legacy
        // compiler limitations are fixed.
        const result = compileSfc(input.source, {
          filename,
          mode: "module",
          isTs: true,
          ssr: lane === "ssr",
          vapor: lane === "vapor",
        });
        native.push({ name: input.name, lane, result });
      }
      for (const provider of ["vue", "vue-vapor-runtime-supported"]) {
        const compiler = require(`${provider}/compiler-sfc`) as typeof import("vue/compiler-sfc");
        for (const lane of provider === "vue" ? ["ssr", "dom"] : ["ssr", "dom", "vapor"]) {
          try {
            const { descriptor, errors } = compiler.parse(input.source, { filename });
            assert.deepEqual(errors, []);
            const result = compiler.compileScript(descriptor, {
              id: "union-control",
              inlineTemplate: true,
              fs: {
                fileExists: existsSync,
                readFile: (name: string) => readFileSync(name, "utf8"),
              },
              templateOptions: { ssr: lane === "ssr" },
              vapor: lane === "vapor",
            } as Parameters<typeof compiler.compileScript>[1]);
            official.push({
              name: input.name,
              provider: compiler.version,
              lane,
              code: result.content,
            });
          } catch (error) {
            assert.ok(error instanceof Error);
            official.push({
              name: input.name,
              provider: compiler.version,
              lane,
              error: {
                name: error.name,
                message: error.message.replaceAll(`${directory}/`, "<fixture>/"),
              },
            });
          }
        }
      }
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
  assert.equal(native.length, 18);
  assert.equal(official.length, 30);
  assert.deepEqual(official, fixture.official);
  console.log(
    JSON.stringify({ check: "whole scoped-union boundaries", inputs: fixture, native, official }),
  );
});

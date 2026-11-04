import assert from "node:assert/strict";
import { test } from "node:test";

import { typecheckerRunStatus } from "../../tools/support/compat/fixtures/typechecker-qualification.ts";

const configFile = "packages/frontend/editor-ui/tsconfig.json";

function parsed(files: readonly { file: string; diagnostics: readonly string[] }[]) {
  return { files };
}

test("n8n configuration diagnostics do not qualify as semantic findings", () => {
  const packet = parsed([
    { file: "src/App.vue", diagnostics: [] },
    {
      file: configFile,
      diagnostics: [
        `${configFile}(1,1): error TS2688: Cannot find type definition file for 'vitest/globals'.`,
        `${configFile}(1,1): error TS2688: Cannot find type definition file for 'vue'.`,
        `${configFile}(1,1): error TS2688: Cannot find type definition file for 'node'.`,
        `${configFile}(1,1): error TS2688: Cannot find type definition file for 'vite/client'.`,
        `${configFile}(2,1): error TS6053: File '@n8n/i18n' not found.`,
      ],
    },
  ]);
  assert.equal(typecheckerRunStatus(1, packet), "configuration");
  assert.equal(typecheckerRunStatus(0, packet), "ok");
});

test("authored type errors stay semantic findings", () => {
  const packet = parsed([
    {
      file: "src/App.vue",
      diagnostics: [
        "src/App.vue(1,1): error TS2322: Type 'string' is not assignable to type 'number'.",
      ],
    },
    {
      file: configFile,
      diagnostics: [
        `${configFile}(1,1): error TS2688: Cannot find type definition file for 'vue'.`,
      ],
    },
  ]);
  assert.equal(typecheckerRunStatus(1, packet), "findings");
});

test("a clean packet stays ok only at exit zero", () => {
  const packet = parsed([{ file: "src/App.vue", diagnostics: [] }]);
  assert.equal(typecheckerRunStatus(0, packet), "ok");
  assert.equal(typecheckerRunStatus(1, packet), "findings");
});

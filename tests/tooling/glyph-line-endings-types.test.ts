import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const require = createRequire(import.meta.url);

test("shipped native formatter types accept the same line endings as generated config types", (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "glyph-line-ending-types-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  fs.writeFileSync(
    path.join(directory, "consumer.mts"),
    [
      `import { formatSfc, type FormatOptionsNapi } from ${JSON.stringify(path.join(root, "npm/native/index.js"))};`,
      `import type { FormatterConfig } from ${JSON.stringify(path.join(root, "npm/cli/src/types/generated.js"))};`,
      'for (const endOfLine of ["lf", "crlf", "cr", "auto"] as const) {',
      "  const config: FormatterConfig = { endOfLine };",
      "  const native: FormatOptionsNapi = { endOfLine: config.endOfLine };",
      "  formatSfc('', native);",
      "}",
      "// @ts-expect-error unknown modes must remain rejected.",
      "formatSfc('', { endOfLine: 'windows' });",
      "// @ts-expect-error numeric modes must remain rejected.",
      "formatSfc('', { endOfLine: 1 });",
      "",
    ].join("\n"),
  );
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        module: "NodeNext",
        moduleResolution: "NodeNext",
        noEmit: true,
        skipLibCheck: true,
        strict: true,
        target: "ES2022",
        types: [],
      },
      files: ["consumer.mts"],
    }),
  );
  const result = spawnSync(
    process.execPath,
    [require.resolve("typescript/bin/tsc"), "-p", directory, "--pretty", "false"],
    { encoding: "utf8", timeout: 30_000 },
  );
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
});

// Pin compiler process mode independently of the runtime process under test.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

export function primaryStaticClass(rows: any[], options: any, mode = "development") {
  const result = spawnSync(process.execPath, [fileURLToPath(import.meta.url)], {
    input: JSON.stringify({ rows, options }),
    encoding: "utf8",
    env: { ...process.env, NODE_ENV: mode },
    timeout: 30_000,
    maxBuffer: 8 * 1024 * 1024,
  });
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const { rows, options } = JSON.parse(fs.readFileSync(0, "utf8"));
  const ui = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
  const vue = createRequire(ui.resolve("vue/package.json"));
  const outputs = rows.map((row: any) => {
    const compiler = vue(`@vue/compiler-${row.target}`);
    assert.equal(vue(`@vue/compiler-${row.target}/package.json`).version, "3.5.35");
    const actual = compiler.compile(row.template ?? row.source, {
      ...options,
      ...(row.scopeId ? { scopeId: row.scopeId } : {}),
    });
    return { code: actual.code, map: actual.map };
  });
  process.stdout.write(JSON.stringify(outputs));
}

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import path from "node:path";
import type { Diagnostic } from "./upstream/vue-language-tools.ts";

const require = createRequire(import.meta.url);

export function assertVueTsc(directory: string): void {
  assert.deepEqual(vueTscDiagnostics(directory), []);
}

export function vueTscDiagnostics(
  directory: string,
): Pick<Diagnostic, "file" | "line" | "column" | "code">[] {
  const cli = path.join(path.dirname(require.resolve("vue-tsc/package.json")), "bin/vue-tsc.js");
  const result = spawnSync(process.execPath, [cli, "--noEmit", "--pretty", "false"], {
    cwd: directory,
    encoding: "utf8",
    timeout: 60_000,
  });
  assert.equal(result.error, undefined);
  assert.ok(result.status === 0 || result.status === 2, result.stdout + result.stderr);
  const diagnostics = [...result.stdout.matchAll(/^(.+?)\((\d+),(\d+)\): error TS(\d+):/gm)].map(
    (match) => ({
      file: match[1].replaceAll("\\", "/"),
      line: Number(match[2]),
      column: Number(match[3]),
      code: Number(match[4]),
    }),
  );
  assert.equal(result.status === 0, diagnostics.length === 0, result.stdout + result.stderr);
  assert.equal(
    (result.stdout.match(/error TS\d+:/g) ?? []).length,
    diagnostics.length,
    result.stdout,
  );
  return diagnostics;
}

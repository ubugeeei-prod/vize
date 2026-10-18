import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { inspect } from "node:util";
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
  captureName?: string,
): Pick<Diagnostic, "file" | "line" | "column" | "code">[] {
  const cli = path.join(path.dirname(require.resolve("vue-tsc/package.json")), "bin/vue-tsc.js");
  if (captureName) assert.match(captureName, /^[a-z0-9]+(?:-[a-z0-9]+)*$/);
  const argv = [cli, "--noEmit", "--pretty", "false"];
  const result = spawnSync(process.execPath, argv, {
    cwd: directory,
    encoding: "utf8",
    timeout: 60_000,
  });
  let captureError: unknown;
  if (captureName) {
    try {
      const root = fileURLToPath(new URL("../../..", import.meta.url));
      const output = path.join(
        root,
        "target/differential/fallthrough-listener-inputs",
        captureName,
      );
      assert.ok(fs.realpathSync(output).startsWith(fs.realpathSync(root) + path.sep));
      fs.writeFileSync(path.join(output, "vue-tsc-stdout.txt"), result.stdout ?? "");
      fs.writeFileSync(path.join(output, "vue-tsc-stderr.txt"), result.stderr ?? "");
      fs.writeFileSync(
        path.join(output, "vue-tsc-process.json"),
        JSON.stringify({
          executable: process.execPath,
          argv,
          cwd: directory,
          pid: result.pid,
          status: result.status,
          signal: result.signal,
          sourceRevision: process.env.GITHUB_SHA,
          error: result.error
            ? {
                name: result.error.name,
                message: result.error.message,
                stack: result.error.stack,
                ownProperties: Object.getOwnPropertyNames(result.error),
                raw: inspect(result.error, { showHidden: true, depth: null, customInspect: false }),
              }
            : null,
        }),
      );
    } catch (original) {
      captureError = original;
    }
  }
  let diagnostics: Pick<Diagnostic, "file" | "line" | "column" | "code">[];
  try {
    assert.equal(result.error, undefined);
    assert.ok(result.status === 0 || result.status === 2, result.stdout + result.stderr);
    diagnostics = [...result.stdout.matchAll(/^(.+?)\((\d+),(\d+)\): error TS(\d+):/gm)].map(
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
  } catch (original) {
    if (captureError)
      throw new AggregateError([original, captureError], "vue-tsc and mandatory capture failed", {
        cause: { checkerError: original, process: result },
      });
    throw original;
  }
  if (captureError)
    throw new AggregateError([captureError], "mandatory vue-tsc capture failed", { cause: result });
  return diagnostics;
}

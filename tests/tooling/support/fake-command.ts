import fs, { writeFileSync } from "node:fs";
import path from "node:path";

const nodePrelude = [
  "import { createRequire } from 'node:module';",
  "const require = createRequire(import.meta.url);",
].join("\n");

// Fake registry probes run concurrently, just like the real HTTP requests.
// Serialize their counter transactions and publish whole JSON files atomically
// so independent provider processes neither lose updates nor expose truncation.
export const fakeJsonStateTransaction = [
  "function mutateFakeJsonState(statePath, update) {",
  "  const stateFs = require('node:fs');",
  "  const lock = statePath + '.lock';",
  "  const deadline = Date.now() + 10000;",
  "  for (;;) {",
  "    try { stateFs.mkdirSync(lock); break; }",
  "    catch (error) {",
  "      if (error.code !== 'EEXIST' || Date.now() >= deadline) throw error;",
  "      Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 1);",
  "    }",
  "  }",
  "  const temporary = statePath + '.' + process.pid + '.tmp';",
  "  try {",
  "    const state = JSON.parse(stateFs.readFileSync(statePath, 'utf8'));",
  "    update(state);",
  "    stateFs.writeFileSync(temporary, JSON.stringify(state));",
  "    stateFs.renameSync(temporary, statePath);",
  "    return state;",
  "  } finally {",
  "    stateFs.rmSync(temporary, { force: true });",
  "    stateFs.rmdirSync(lock);",
  "  }",
  "}",
].join("\n");

export function writeFakeCommand(binDir: string, name: string, body: string): void {
  const unixPath = path.join(binDir, name);
  writeFileSync(unixPath, `#!/usr/bin/env node\n${nodePrelude}\n${body}`);
  fs.chmodSync(unixPath, 0o755);

  if (process.platform === "win32") {
    writeFileSync(path.join(binDir, `${name}.cmd`), `@echo off\r\nnode "%~dp0\\${name}" %*\r\n`);
  }
}

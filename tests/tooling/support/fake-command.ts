import fs, { writeFileSync } from "node:fs";
import path from "node:path";

const nodePrelude = [
  "import { createRequire } from 'node:module';",
  "const require = createRequire(import.meta.url);",
].join("\n");

export function writeFakeCommand(binDir: string, name: string, body: string): void {
  const unixPath = path.join(binDir, name);
  writeFileSync(unixPath, `#!/usr/bin/env node\n${nodePrelude}\n${body}`);
  fs.chmodSync(unixPath, 0o755);

  if (process.platform === "win32") {
    writeFileSync(path.join(binDir, `${name}.cmd`), `@echo off\r\nnode "%~dp0\\${name}" %*\r\n`);
  }
}

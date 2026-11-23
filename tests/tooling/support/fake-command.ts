import { readToolingProgram, toolingProgram } from "./program-path.ts";
import fs, { writeFileSync } from "node:fs";
import path from "node:path";

// Fake registry probes run concurrently, just like the real HTTP requests.
// Serialize their counter transactions and publish whole JSON files atomically
// so independent provider processes neither lose updates nor expose truncation.
export const fakeStateTransactionPath = toolingProgram("process/update-json-state.cjs");

export function writeFakeCommand(binDir: string, name: string, body: string): void {
  const unixPath = path.join(binDir, name);
  writeFileSync(unixPath, readToolingProgram("process/run-fake-command.js.in", { BODY: body }));
  fs.chmodSync(unixPath, 0o755);

  if (process.platform === "win32") {
    writeFileSync(
      path.join(binDir, `${name}.cmd`),
      readToolingProgram("process/node-sibling.cmd", { ENTRY: name }).replace(/\r?\n/gu, "\r\n"),
    );
  }
}

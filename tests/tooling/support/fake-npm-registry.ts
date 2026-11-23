import { readToolingProgram, toolingProgram } from "./program-path.ts";
import { fakeStateTransactionPath, writeFakeCommand } from "./fake-command.ts";

// Existing metadata fixtures now model the archive required by successful publication.
export const fakeRegistryArchivePath = toolingProgram("process/serve-registry-archive.cjs");

export function writeFakeNpmRegistry(binDir: string): void {
  writeFakeCommand(
    binDir,
    "curl",
    readToolingProgram("commands/fake-npm-registry-ts/curl.js.in", {
      ARCHIVE_MODULE: JSON.stringify(fakeRegistryArchivePath),
      STATE_MODULE: JSON.stringify(fakeStateTransactionPath),
    }),
  );
}

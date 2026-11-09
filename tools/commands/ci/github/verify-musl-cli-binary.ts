import { spawnSync } from "node:child_process";
import path from "node:path";

import { isEntrypoint, runEntrypoint } from "../../../support/command-runtime.ts";

function commandOutput(command: string, arguments_: readonly string[]): string {
  // `strings` over the release binary prints tens of MB.
  const result = spawnSync(command, [...arguments_], { encoding: "utf8", maxBuffer: 1024 ** 3 });
  if (result.error !== undefined) throw new Error(`failed to run ${command}: ${result.error}`);
  if (result.status !== 0) {
    throw new Error(
      `${command} ${arguments_.join(" ")} failed with exit ${result.status ?? 1}\n${result.stderr}${result.stdout}`.trim(),
    );
  }
  return `${result.stdout}${result.stderr}`;
}

export function glibcRequirementLines(text: string): string[] {
  return [...new Set(text.split(/\r?\n/u).filter((line) => /GLIBC_\d/u.test(line)))].sort();
}

export function verifyMuslCliBinary(arguments_: readonly string[]): number {
  if (arguments_.length !== 1) {
    throw new Error("Usage: node tools/commands/ci/github/verify-musl-cli-binary.ts <rust-target>");
  }
  const binary = path.join("target", arguments_[0]!, "release", "vize");
  const fileStatus = spawnSync("file", [binary], { stdio: "inherit" });
  if (fileStatus.error !== undefined) throw new Error(`failed to run file: ${fileStatus.error}`);
  if (fileStatus.status !== 0) {
    throw new Error(`file ${binary} failed with exit ${fileStatus.status ?? 1}`);
  }

  const readelf = commandOutput("readelf", ["-l", binary]);
  if (readelf.includes("Requesting program interpreter")) {
    console.error("::error ::musl CLI binary has a dynamic interpreter");
    process.stdout.write(readelf);
    throw new Error("musl CLI binary verification failed");
  }
  const glibc = glibcRequirementLines(commandOutput("readelf", ["--version-info", binary]));
  if (glibc.length > 0) {
    console.error("::error ::musl CLI binary contains glibc version requirements");
    for (const line of glibc) console.log(line);
    throw new Error("musl CLI binary verification failed");
  }
  return 0;
}

if (isEntrypoint(import.meta.url)) {
  runEntrypoint(() => verifyMuslCliBinary(process.argv.slice(2)));
}

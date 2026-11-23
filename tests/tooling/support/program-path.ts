import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

export function toolingProgram(name: string): string {
  return fileURLToPath(new URL("../programs/" + name, import.meta.url));
}

/** A fixture with a restricted PATH must launch the very same Node binary as its parent. */
export function installToolingProgram(destination: string, name: string): void {
  const body = readFileSync(toolingProgram(name), "utf8");
  writeFileSync(destination, "#!" + process.execPath + "\n" + body, { mode: 0o755 });
}

interface ProcessResult {
  stdout?: string | number[];
  stderr?: string | number[];
  status?: number;
  delayMs?: number;
  keepAlive?: boolean;
  echoStdin?: boolean;
}

export function processResultArgs(result: ProcessResult): string[] {
  return [toolingProgram("process/emit-result.js"), JSON.stringify(result)];
}

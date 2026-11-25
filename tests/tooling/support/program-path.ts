import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import {
  commentStyle,
  withoutGeneratedHeader,
} from "../../../tools/support/ci/generated-header.ts";

export function toolingProgram(name: string): string {
  return fileURLToPath(new URL("../programs/" + name, import.meta.url));
}

export function readToolingProgram(
  name: string | URL,
  values: Readonly<Record<string, unknown>> = {},
): string {
  const file = typeof name === "string" ? toolingProgram(name) : fileURLToPath(name);
  const source = withoutGeneratedHeader(readFileSync(file, "utf8"), commentStyle(file));
  return source.replace(/@@([A-Z][A-Z0-9_]*)@@/gu, (_, key: string) => {
    if (!Object.hasOwn(values, key))
      throw new Error("Missing fixture argument " + key + " in " + name);
    return String(values[key]);
  });
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

import { writeFileSync } from "node:fs";
import path from "node:path";

export function writeFakeVize(directory: string, body: string): string {
  const executable = path.join(directory, "fake-vize.mjs");
  writeFileSync(executable, "#!/usr/bin/env node\n" + body + "\n", { mode: 0o755 });
  return executable;
}

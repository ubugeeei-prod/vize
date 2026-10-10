import { execFile } from "node:child_process";
import { fileURLToPath } from "node:url";
const cli = fileURLToPath(new URL("../../src/cli/index.ts", import.meta.url));

export async function runCli(cwd: string, args: string[]) {
  return new Promise<{ status: number; stdout: string; stderr: string }>((resolve, reject) => {
    execFile(
      process.execPath,
      ["--import", import.meta.resolve("tsx"), cli, ...args],
      { cwd, timeout: 45000, encoding: "utf8" },
      (error, stdout, stderr) => {
        if (error && typeof error.code !== "number") return reject(error);
        resolve({ status: error?.code ?? 0, stdout, stderr });
      },
    );
  });
}

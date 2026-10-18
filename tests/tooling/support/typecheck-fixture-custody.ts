import type { SpawnSyncReturns } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

/** Retain the command and its owned synthetic inputs before assertions or cleanup. */
export function retainFixtureCommand(
  evidenceRoot: string,
  command: { executable: string; args: string[]; cwd: string; timeoutMs?: number },
  result: SpawnSyncReturns<Buffer>,
  directories: Record<string, string>,
) {
  fs.mkdirSync(evidenceRoot, { recursive: true });
  const directory = fs.mkdtempSync(path.join(evidenceRoot, "mutation-deadline-"));
  const files: { path: string; sizeBytes: number; sha256: string }[] = [];
  function retain(relative: string, bytes: Buffer) {
    const destination = path.join(directory, relative);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, bytes);
    files.push({
      path: relative,
      sizeBytes: bytes.length,
      sha256: createHash("sha256").update(bytes).digest("hex"),
    });
  }
  function copy(source: string, relative: string) {
    const stat = fs.lstatSync(source);
    if (stat.isDirectory()) {
      fs.mkdirSync(path.join(directory, relative), { recursive: true });
      for (const name of fs.readdirSync(source).sort())
        copy(path.join(source, name), path.join(relative, name));
    } else if (stat.isFile()) {
      retain(relative, fs.readFileSync(source));
      fs.chmodSync(path.join(directory, relative), stat.mode & 0o777);
    } else {
      throw new Error(`refusing non-physical fixture evidence: ${source}`);
    }
  }
  retain("stdout.bin", result.stdout ?? Buffer.alloc(0));
  retain("stderr.bin", result.stderr ?? Buffer.alloc(0));
  const error =
    result.error == null
      ? null
      : Object.fromEntries(
          Object.getOwnPropertyNames(result.error).map((key) => [
            key,
            Reflect.get(result.error!, key),
          ]),
        );
  const receipt = {
    schema: "vize.toolingFixtureCommand",
    version: 1,
    command,
    result: {
      pid: result.pid,
      status: result.status,
      signal: result.signal,
      error,
      stdoutCaptured: result.stdout != null,
      stderrCaptured: result.stderr != null,
    },
    directories,
  };
  retain("command.json", Buffer.from(`${JSON.stringify(receipt, null, 2)}\n`));
  for (const [name, source] of Object.entries(directories)) copy(source, name);
  fs.writeFileSync(path.join(directory, "files.json"), `${JSON.stringify(files, null, 2)}\n`);
  return directory;
}

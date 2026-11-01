/** Preserve native raw/status before any post-native validation can fail. */
import fs from "node:fs";
import path from "node:path";
type InputRow = Record<string, unknown>;
const write = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
export interface ReplayOutcome {
  stdout: Buffer | null;
  stderr: Buffer | null;
  status: number | null;
  signal: NodeJS.Signals | null;
  pid?: number;
  error?: Error;
}

/** Persist observations before any caller-supplied build-info/seal work can fail. */
export function recordReplayBeforeInspection<T>(
  out: string,
  replay: ReplayOutcome,
  elapsedNs: string,
  inspect: (status: InputRow) => T,
): T {
  const role = "native-replay";
  fs.writeFileSync(path.join(out, role + ".stdout.bin"), replay.stdout ?? Buffer.alloc(0), {
    flag: "wx",
  });
  fs.writeFileSync(path.join(out, role + ".stderr.bin"), replay.stderr ?? Buffer.alloc(0), {
    flag: "wx",
  });
  const status = {
    status: replay.status,
    signal: replay.signal,
    error: replay.error?.message ?? null,
    invokedPid: replay.pid ?? null,
    elapsedNs,
  };
  write(path.join(out, role + ".status.json"), status);
  return inspect(status);
}

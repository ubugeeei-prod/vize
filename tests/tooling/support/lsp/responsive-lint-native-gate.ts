import assert from "node:assert/strict";
import fs from "node:fs";
import { processTreeRss } from "../../../performance/support/process-metrics.ts";

export type NativeLife = {
  pid: number;
  birthTicks: string;
  executable: string;
  state: string;
  rawStat: string;
};

function readLife(pid: number): NativeLife {
  const raw = fs.readFileSync(`/proc/${pid}/stat`, "utf8");
  const fields = raw
    .slice(raw.lastIndexOf(") ") + 2)
    .trim()
    .split(/\s+/u);
  assert.match(fields[19], /^\d+$/u);
  return {
    pid,
    rawStat: raw,
    birthTicks: fields[19],
    state: fields[0],
    executable: fs.realpathSync(`/proc/${pid}/exe`),
  };
}

export class NativeDiagnosticsGate {
  readonly lives: NativeLife[];
  readonly signals: { signal: string; life: NativeLife }[] = [];
  private stopped = new Set<number>();

  constructor(
    private readonly owner: number,
    private readonly executable: string,
  ) {
    const descendants = processTreeRss(owner);
    assert.ok(descendants, "actual server process tree unavailable");
    this.lives = descendants.members
      .filter(({ pid }) => pid !== owner)
      .map(({ pid }) => readLife(pid))
      .filter((life) => life.executable === executable && life.state !== "Z");
    assert.ok(this.lives.length > 0, "real native typecheck must have started before gating");
  }

  private current(entry: NativeLife): NativeLife {
    assert.ok(processTreeRss(this.owner)?.members.some(({ pid }) => pid === entry.pid));
    const current = readLife(entry.pid);
    assert.equal(current.birthTicks, entry.birthTicks, "do not signal a reused PID");
    assert.equal(current.executable, this.executable);
    assert.notEqual(current.state, "Z");
    return current;
  }

  async stop(): Promise<void> {
    for (const entry of this.lives) {
      const current = this.current(entry);
      process.kill(entry.pid, "SIGSTOP");
      this.stopped.add(entry.pid);
      this.signals.push({ signal: "SIGSTOP", life: current });
    }
    const deadline = Date.now() + 2_000;
    while (!this.lives.every((entry) => this.current(entry).state === "T")) {
      assert.ok(Date.now() < deadline, "native processes did not enter stopped state");
      await new Promise((resolve) => setTimeout(resolve, 10));
    }
  }

  assertStopped(): void {
    for (const entry of this.lives) assert.equal(this.current(entry).state, "T");
  }

  resume(): void {
    for (const entry of this.lives) {
      if (!this.stopped.has(entry.pid)) continue;
      const current = this.current(entry);
      process.kill(entry.pid, "SIGCONT");
      this.stopped.delete(entry.pid);
      this.signals.push({ signal: "SIGCONT", life: current });
    }
  }
}

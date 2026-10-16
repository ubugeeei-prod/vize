import assert from "node:assert/strict";
import fs from "node:fs";

export type ProcessIdentity = { pid: number; birth_ticks: number };
type CurrentIdentity = { birth_ticks: number; state: string } | null;
type Snapshot = { processes: ProcessIdentity[] };

function currentIdentity(pid: number): { birth_ticks: number; state: string } | null {
  try {
    const stat = fs.readFileSync(`/proc/${pid}/stat`, "utf8");
    const fields = stat
      .slice(stat.lastIndexOf(") ") + 2)
      .trim()
      .split(/\s+/u);
    assert.match(fields[19], /^\d+$/u);
    return { birth_ticks: Number(fields[19]), state: fields[0] };
  } catch (failure) {
    if ((failure as NodeJS.ErrnoException).code === "ENOENT") return null;
    throw failure;
  }
}

export function nativeIdentities(sample: Snapshot, executable: string): ProcessIdentity[] {
  return sample.processes.filter((entry) => {
    const current = currentIdentity(entry.pid);
    if (!current || current.state === "Z" || current.birth_ticks !== entry.birth_ticks)
      return false;
    try {
      return fs.realpathSync(`/proc/${entry.pid}/exe`) === executable;
    } catch (failure) {
      if ((failure as NodeJS.ErrnoException).code === "ENOENT") return false;
      throw failure;
    }
  });
}

export async function awaitRetired(identities: ProcessIdentity[]) {
  const deadline = Date.now() + 5_000;
  while (true) {
    const observed = identities.map((entry) => ({ ...entry, current: currentIdentity(entry.pid) }));
    if (
      observed.every(
        (entry) =>
          !entry.current ||
          entry.current.state === "Z" ||
          entry.current.birth_ticks !== entry.birth_ticks,
      )
    ) {
      return observed;
    }
    assert.ok(
      Date.now() < deadline,
      `observed native process lives remain: ${JSON.stringify(observed)}`,
    );
    await new Promise((resolve) => setTimeout(resolve, 25));
  }
}

/** A nonrunning zombie still occupies the original child identity until wait/reaping completes. */
export function isReaped(identity: ProcessIdentity, current: CurrentIdentity): boolean {
  return current === null || current.birth_ticks !== identity.birth_ticks;
}

/** Retain every observation, including a complete negative result at the existing five-second bound. */
export async function observeReaped(
  identities: ProcessIdentity[],
  observer = {
    current: currentIdentity,
    now: Date.now,
    pause: () => new Promise<void>((resolve) => setTimeout(resolve, 25)),
  },
) {
  assert.ok(identities.length > 0, "the final recovered provider must supply an actual identity");
  const started = observer.now();
  const probes = [];
  while (true) {
    const observed = identities.map((identity) => {
      try {
        return { ...identity, current: observer.current(identity.pid) };
      } catch (error) {
        return { ...identity, readError: error instanceof Error ? error.stack : String(error) };
      }
    });
    const elapsedMs = observer.now() - started;
    probes.push({ elapsedMs, observed });
    const readFailed = observed.some((entry) => entry.readError !== undefined);
    const reaped =
      !readFailed &&
      observed.every((entry) => entry.current !== undefined && isReaped(entry, entry.current));
    if (reaped || readFailed || elapsedMs >= 5_000)
      return { criterion: "absent-or-different-birth", timeoutMs: 5_000, reaped, probes };
    await observer.pause();
  }
}

export async function retireNative(
  sample: () => Snapshot,
  executable: string,
  identities: ProcessIdentity[],
) {
  assert.ok(identities.length > 0);
  const signals = [];
  for (const entry of identities) {
    // No await separates this fresh descendant/birth/physical-binary check
    // from the signal. A reused PID is never signaled as the old native life.
    const owned = nativeIdentities(sample(), executable);
    assert.ok(
      owned.some(
        (current) => current.pid === entry.pid && current.birth_ticks === entry.birth_ticks,
      ),
    );
    assert.equal(currentIdentity(entry.pid)?.birth_ticks, entry.birth_ticks);
    assert.equal(fs.realpathSync(`/proc/${entry.pid}/exe`), executable);
    process.kill(entry.pid, "SIGKILL");
    signals.push({ ...entry, signal: "SIGKILL", immediatelyVerifiedDescendants: owned });
  }
  return { signals, retired: await awaitRetired(identities) };
}

import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";

export const processPacketLimit = 128;
export type ObservationPhase = "interval" | "terminal";
type ProcessPacket = { pid: number; startTicks: string; observedAt: number };
type Retained = {
  slot: number;
  sequence: number;
  identity: string;
  filename: string;
  observedAt: number;
  pid: number;
  startTicks: string;
  sha256: string;
  phase: ObservationPhase;
};
type Attempt = {
  phase: ObservationPhase;
  startedAt: number;
  finishedAt: number | null;
  before: number;
};
type Retention = {
  next: number;
  dropped: number;
  persistenceFailures: number;
  pendingOwned: boolean;
  attempts: number;
  firstAttemptAt: number | null;
  lastAttemptAt: number | null;
  terminal: Attempt | null;
  retained: Map<number, Retained>;
};
const states = new WeakMap<Set<string>, Retention>();
function state(seen: Set<string>): Retention {
  let value = states.get(seen);
  if (!value) {
    value = {
      next: 0,
      dropped: 0,
      persistenceFailures: 0,
      pendingOwned: false,
      attempts: 0,
      firstAttemptAt: null,
      lastAttemptAt: null,
      terminal: null,
      retained: new Map(),
    };
    states.set(seen, value);
  }
  return value;
}

export function beginProcessObservation(seen: Set<string>, phase: ObservationPhase): Attempt {
  const value = state(seen);
  const attempt = { phase, startedAt: Date.now() / 1000, finishedAt: null, before: value.next };
  value.attempts++;
  value.firstAttemptAt ??= attempt.startedAt;
  value.lastAttemptAt = attempt.startedAt;
  if (phase === "terminal") value.terminal = attempt;
  return attempt;
}

export function finishProcessObservation(attempt: Attempt): void {
  attempt.finishedAt = Date.now() / 1000;
}

/** Keep the initial whole packet and the latest127 whole packets. Replacing a
 * fixed slot atomically never exposes a129th retained JSON packet. A failed
 * write/rename keeps the old packet and the first persistence error intact. */
export function retainProcessPacket(
  seen: Set<string>,
  identity: string,
  packet: ProcessPacket,
  output: string,
  phase: ObservationPhase,
): void {
  const value = state(seen);
  const key = phase === "terminal" ? "terminal:" + identity : identity;
  if (seen.has(key)) return;
  const sequence = value.next;
  const slot = sequence === 0 ? 0 : 1 + ((sequence - 1) % (processPacketLimit - 1));
  const filename = slot === 0 ? "initial.json" : `tail-${String(slot).padStart(3, "0")}.json`;
  const directory = path.join(output, "processes");
  const destination = path.join(directory, filename);
  const pending = path.join(directory, ".retained-packet.pending");
  const raw = Buffer.from(JSON.stringify(packet, null, 2) + "\n");
  try {
    fs.mkdirSync(directory, { recursive: true });
    if (value.pendingOwned) {
      fs.rmSync(pending);
      value.pendingOwned = false;
    }
    fs.writeFileSync(pending, raw, { flag: "wx" });
    value.pendingOwned = true;
    fs.renameSync(pending, destination);
    value.pendingOwned = false;
  } catch (error) {
    value.persistenceFailures++;
    if (value.pendingOwned) {
      try {
        fs.rmSync(pending);
        value.pendingOwned = false;
      } catch {
        /* Preserve the original persistence failure. */
      }
    }
    throw error;
  }
  const previous = value.retained.get(slot);
  if (previous) {
    seen.delete(previous.identity);
    value.dropped++;
  }
  seen.add(key);
  value.retained.set(slot, {
    slot,
    sequence,
    identity: key,
    filename,
    observedAt: packet.observedAt,
    pid: packet.pid,
    startTicks: packet.startTicks,
    sha256: createHash("sha256").update(raw).digest("hex"),
    phase,
  });
  value.next++;
}

export function processRetention(seen: Set<string>) {
  const value = state(seen);
  const retained = [...value.retained.values()].sort((a, b) => a.sequence - b.sequence);
  const times = retained.map((row) => row.observedAt);
  const tail = retained.filter((row) => row.slot !== 0).map((row) => row.observedAt);
  return {
    policy: "initial packet plus rolling127 packet tail; partial observations only",
    packetLimit: processPacketLimit,
    retainedPackets: retained.length,
    observedPackets: value.next,
    droppedPackets: value.dropped,
    persistenceFailures: value.persistenceFailures,
    observationAttempts: value.attempts,
    firstAttemptAt: value.firstAttemptAt,
    lastAttemptAt: value.lastAttemptAt,
    retainedTimeCoverage: {
      first: times.length ? Math.min(...times) : null,
      last: times.length ? Math.max(...times) : null,
      tailFirst: tail.length ? Math.min(...tail) : null,
      tailLast: tail.length ? Math.max(...tail) : null,
      complete: false,
    },
    terminalObservation:
      value.terminal === null
        ? null
        : {
            startedAt: value.terminal.startedAt,
            finishedAt: value.terminal.finishedAt,
            packetWrites: value.next - value.terminal.before,
            partial: true,
          },
    retained: retained.map(({ identity: _identity, ...row }) => row),
  };
}

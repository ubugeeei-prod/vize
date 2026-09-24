/**
 * Test-only virtual single-shot timer host shared by the state composable
 * suites. Not part of the published package: `vp pack` only builds the
 * catalogued entries.
 */
import type { TimeoutScheduler } from "../timeout-scheduler.ts";

/** One scheduled virtual timeout. */
interface FakeTimeout {
  readonly at: number;
  readonly callback: () => void;
}

/** Deterministic virtual clock implementing {@link TimeoutScheduler}. */
export class FakeTimeouts implements TimeoutScheduler {
  now = 0;
  private nextId = 1;
  readonly timers = new Map<number, FakeTimeout>();

  setTimeout(callback: () => void, delayMs: number): number {
    const id = this.nextId++;
    this.timers.set(id, { at: this.now + delayMs, callback });
    return id;
  }

  clearTimeout(handle: unknown): void {
    this.timers.delete(handle as number);
  }

  /** Advance virtual time, firing due timers in order. */
  advance(ms: number): void {
    const target = this.now + ms;
    for (;;) {
      let nextId: number | undefined;
      let next: FakeTimeout | undefined;
      for (const [id, timer] of this.timers) {
        if (timer.at <= target && (next === undefined || timer.at < next.at)) {
          next = timer;
          nextId = id;
        }
      }
      if (next === undefined || nextId === undefined) break;
      this.timers.delete(nextId);
      this.now = next.at;
      next.callback();
    }
    this.now = target;
  }
}

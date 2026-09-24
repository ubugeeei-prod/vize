/**
 * Test-only virtual clock shared by the timing composable suites. Not part of
 * the published package: `vp pack` only builds the catalogued entries.
 */
import type { IntervalScheduler } from "../use-interval.ts";
import type { FrameScheduler } from "../use-raf-fn.ts";
import type { TimeoutScheduler } from "../timeout-scheduler.ts";

/** One scheduled virtual timer. */
interface FakeTimer {
  readonly id: number;
  at: number;
  readonly every: number | undefined;
  readonly callback: () => void;
}

/** Deterministic virtual clock implementing the timeout and interval scheduler shapes. */
export class FakeClock {
  now = 0;
  private nextId = 1;
  readonly timers = new Map<number, FakeTimer>();

  readonly timeout: TimeoutScheduler = {
    setTimeout: (callback: () => void, delayMs: number): number =>
      this.add(callback, delayMs, undefined),
    clearTimeout: (handle: unknown): void => {
      this.timers.delete(handle as number);
    },
  };

  readonly interval: IntervalScheduler = {
    setInterval: (callback: () => void, intervalMs: number): number =>
      this.add(callback, intervalMs, intervalMs),
    clearInterval: (handle: unknown): void => {
      this.timers.delete(handle as number);
    },
  };

  get size(): number {
    return this.timers.size;
  }

  private add(callback: () => void, delayMs: number, every: number | undefined): number {
    const id = this.nextId++;
    this.timers.set(id, { id, at: this.now + delayMs, every, callback });
    return id;
  }

  advance(ms: number): void {
    const target = this.now + ms;
    for (;;) {
      let next: FakeTimer | undefined;
      for (const timer of this.timers.values()) {
        if (timer.at <= target && (next === undefined || timer.at < next.at)) next = timer;
      }
      if (next === undefined) break;
      this.now = next.at;
      if (next.every === undefined) this.timers.delete(next.id);
      else next.at += next.every;
      next.callback();
    }
    this.now = target;
  }
}

/** Manually stepped animation-frame host. */
export class FakeFrames implements FrameScheduler {
  private nextId = 1;
  readonly pending = new Map<number, (timestamp: number) => void>();

  requestAnimationFrame(callback: (timestamp: number) => void): number {
    const id = this.nextId++;
    this.pending.set(id, callback);
    return id;
  }

  cancelAnimationFrame(handle: unknown): void {
    this.pending.delete(handle as number);
  }

  /** Deliver one frame at `timestamp` to every pending callback. */
  frame(timestamp: number): void {
    const callbacks = [...this.pending.values()];
    this.pending.clear();
    for (const callback of callbacks) callback(timestamp);
  }
}

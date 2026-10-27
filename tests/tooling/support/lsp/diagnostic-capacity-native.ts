import assert from "node:assert/strict";
import type { CapacityCoverage } from "./diagnostic-capacity.ts";

type NativePublication = {
  started: number;
  sent: number;
  finished: number;
  completedPasses: number;
  outsidePassSent: number;
  activeSent: number | null;
};

/** A versioned empty edit notification can be lint-only. Require the actual native send too. */
export class CapacityNativeCoverage {
  private seen = 0;
  private tail = "";
  private started = 0;
  private finished = 0;
  private active = 0;
  private invalidTransitions = 0;
  private readonly index = new Map<string, NativePublication>();
  private readonly coverage: CapacityCoverage;
  private readonly stderr: () => string;
  constructor(coverage: CapacityCoverage, stderr: () => string) {
    this.coverage = coverage;
    this.stderr = stderr;
  }
  private refresh(): void {
    const text = this.stderr();
    assert.ok(text.length >= this.seen, "the complete source trace must remain available");
    const lines = (this.tail + text.slice(this.seen)).split("\n");
    this.seen = text.length;
    this.tail = lines.pop() ?? "";
    for (const line of lines) {
      const match =
        /(starting initial type diagnostics for|sent collected diagnostics for|finished initial type diagnostics for) (\S+) version (\d+)/u.exec(
          line,
        );
      if (!match) continue;
      const [, stage, uri, version] = match;
      assert.ok(stage && uri && version);
      const key = uri + "\0" + version;
      const state = this.index.get(key) ?? {
        started: 0,
        sent: 0,
        finished: 0,
        completedPasses: 0,
        outsidePassSent: 0,
        activeSent: null,
      };
      if (stage.startsWith("starting")) {
        state.started++;
        this.started++;
        if (state.activeSent !== null) this.invalidTransitions++;
        else this.active++;
        state.activeSent = 0;
      } else if (stage.startsWith("finished")) {
        state.finished++;
        this.finished++;
        if (state.activeSent === null) this.invalidTransitions++;
        else {
          if (state.activeSent > 0) state.completedPasses++;
          state.activeSent = null;
          this.active--;
        }
      } else {
        state.sent++;
        if (state.activeSent === null) state.outsidePassSent++;
        else state.activeSent++;
      }
      this.index.set(key, state);
    }
  }
  rows(version: number, repaired: boolean) {
    this.refresh();
    return this.coverage.rows(version, repaired).map((row) => {
      const state = this.index.get(row.uri + "\0" + version);
      return {
        ...row,
        nativeStarted: state?.started ?? 0,
        nativeSent: state?.sent ?? 0,
        nativeFinished: state?.finished ?? 0,
        nativeCompletedPasses: state?.completedPasses ?? 0,
        nativeSendsOutsidePass: state?.outsidePassSent ?? 0,
        complete: row.complete && state !== undefined && state.completedPasses > 0,
      };
    });
  }
  idle(): boolean {
    this.refresh();
    return (
      this.started > 0 &&
      this.started === this.finished &&
      this.active === 0 &&
      this.invalidTransitions === 0
    );
  }
  assertComplete(version: number, repaired: boolean): void {
    assert.deepEqual(
      this.rows(version, repaired)
        .filter((row) => !row.complete)
        .map((row) => row.file),
      [],
      "every URI/version needs its latest native packet and a successful source-owned send",
    );
  }
}

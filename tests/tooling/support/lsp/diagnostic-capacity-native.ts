import assert from "node:assert/strict";
import type { CapacityCoverage } from "./diagnostic-capacity.ts";

type NativePublication = { started: number; sent: number; finished: number };

/** A versioned empty edit notification can be lint-only. Require the actual native send too. */
export class CapacityNativeCoverage {
  private seen = 0;
  private tail = "";
  private started = 0;
  private finished = 0;
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
      const state = this.index.get(key) ?? { started: 0, sent: 0, finished: 0 };
      if (stage.startsWith("starting")) {
        state.started++;
        this.started++;
      } else if (stage.startsWith("finished")) {
        state.finished++;
        this.finished++;
      } else state.sent++;
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
        complete:
          row.complete &&
          state !== undefined &&
          state.started > 0 &&
          state.sent > 0 &&
          state.finished > 0,
      };
    });
  }
  idle(): boolean {
    this.refresh();
    return this.started > 0 && this.started === this.finished;
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

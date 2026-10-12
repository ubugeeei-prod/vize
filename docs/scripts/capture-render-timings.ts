import { writeFile } from "node:fs/promises";
import path from "node:path";
import type { RenderJob } from "./navigation-render-concurrency.ts";

type CaptureIdentity = {
  jobIndex: number | null;
  route: string;
  device: string;
  capture: string;
};
type CaptureFields = {
  tileIndex?: number;
  top?: number;
  file?: string;
  width?: number;
  height?: number;
  viewportHeight?: number;
  originalViewportHeight?: number;
};
type CaptureOperation = {
  name: "scroll-to" | "scroll-read";
  startMs: number;
  durationMs: number | null;
  status: "running" | "passed" | "failed";
};
type CaptureEvent = CaptureIdentity &
  CaptureFields & {
    phase: string;
    operations?: CaptureOperation[];
    startMs: number;
    durationMs: number | null;
    status: "running" | "passed" | "failed";
  };

/** Observe existing operations without wrapping promises or changing page clocks. */
export class CaptureTimer {
  private active: CaptureEvent | undefined;
  private activeOperation: CaptureOperation | undefined;
  private readonly events: CaptureEvent[];
  private readonly identity: CaptureIdentity;

  constructor(events: CaptureEvent[], identity: CaptureIdentity) {
    this.events = events;
    this.identity = identity;
  }

  start(phase: string, fields: CaptureFields = {}): void {
    this.finish("failed");
    this.active = {
      ...this.identity,
      ...fields,
      phase,
      startMs: performance.now(),
      durationMs: null,
      status: "running",
    };
    this.events.push(this.active);
  }

  startOperation(name: CaptureOperation["name"]): void {
    if (!this.active) return;
    this.finishOperation("failed");
    this.activeOperation = {
      name,
      startMs: performance.now(),
      durationMs: null,
      status: "running",
    };
    (this.active.operations ??= []).push(this.activeOperation);
  }

  finishOperation(status: "passed" | "failed" = "passed"): void {
    if (!this.activeOperation) return;
    this.activeOperation.durationMs = performance.now() - this.activeOperation.startMs;
    this.activeOperation.status = status;
    this.activeOperation = undefined;
  }

  finish(status: "passed" | "failed" = "passed"): void {
    if (!this.active) return;
    this.finishOperation("failed");
    this.active.durationMs = performance.now() - this.active.startMs;
    this.active.status = status;
    this.active = undefined;
  }
}

export type CaptureTimingSink = (capture: string) => CaptureTimer;

export class CaptureTimings {
  private readonly events: CaptureEvent[] = [];

  forJob(job: Pick<RenderJob, "index" | "route" | "device">): CaptureTimingSink {
    return (capture) =>
      new CaptureTimer(this.events, {
        jobIndex: job.index,
        route: job.route,
        device: job.device,
        capture,
      });
  }

  forControl(): CaptureTimingSink {
    return (capture) =>
      new CaptureTimer(this.events, {
        jobIndex: null,
        route: "capture-controls",
        device: "control",
        capture,
      });
  }

  async write(
    output: string,
    metadata: {
      source: string | null;
      dist: string;
      workers: number;
      startedAt: string;
      status: "passed" | "failed";
      scope: string;
    },
  ): Promise<void> {
    await writeFile(
      path.join(output, "capture-timings.json"),
      JSON.stringify(
        {
          schemaVersion: 1,
          ...metadata,
          timeOrigin: performance.timeOrigin,
          screenshotIncludesPreparationEncodingTransferAndPngWrite: true,
          events: this.events.map((event) =>
            event.status === "running" ? { ...event, status: "failed" } : event,
          ),
        },
        null,
        2,
      ) + "\n",
    );
  }
}

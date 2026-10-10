import assert from "node:assert/strict";

export type RenderWorkers = 1 | 2;
export type RenderDevice = "desktop" | "mobile";
export type RenderJob = {
  index: number;
  route: string;
  device: RenderDevice;
  viewport: { width: number; height: number };
  screenshot: string;
};
export type RenderPhase =
  | "browser-launch"
  | "capture-controls"
  | "browser-close"
  | "server-close"
  | "context"
  | "navigation"
  | "assets"
  | "japanese-fonts"
  | "metrics"
  | "navigation-assertions"
  | "theme-readability"
  | "rule-packets"
  | "command-tabs"
  | "layout"
  | "links"
  | "capture"
  | "theme-captures"
  | "context-close";
export type RenderPhaseTiming = {
  phase: RenderPhase;
  startedAt: string;
  durationMs: number;
  status: "passed" | "failed";
  error?: string;
};
export type RenderJobTiming = {
  index: number;
  route: string;
  device: RenderDevice;
  worker: number;
  startedAt: string;
  durationMs: number;
  status: "passed" | "failed";
  phases: RenderPhaseTiming[];
  error?: string;
};
export type RenderJobResult<T> = { job: RenderJob; timing: RenderJobTiming; receipt?: T };
export type MeasureRenderPhase = <T>(phase: RenderPhase, operation: () => Promise<T>) => Promise<T>;

export function parseRenderWorkers(value: string | undefined): RenderWorkers {
  assert(value === undefined || value === "1" || value === "2", "--workers must be exactly 1 or 2");
  return value === "1" ? 1 : 2;
}

export function createRenderJobs(routes: string[]): RenderJob[] {
  assert(routes.length > 0, "At least one render route is required");
  const normalized = new Set<string>();
  for (const route of routes) {
    assert(/^\/(?:[a-zA-Z0-9_-]+\/)*[a-zA-Z0-9_-]*$/.test(route), `Invalid render route: ${route}`);
    const key = route.replace(/\/$/, "") || "/";
    assert(!normalized.has(key), `Duplicate render route: ${route}`);
    normalized.add(key);
  }
  const jobs: RenderJob[] = [];
  for (const { device, viewport } of [
    { device: "desktop" as const, viewport: { width: 1440, height: 960 } },
    { device: "mobile" as const, viewport: { width: 390, height: 844 } },
  ]) {
    for (const route of routes) {
      const index = jobs.length;
      const name = route.replace(/^\//, "").replaceAll("/", "-") || "home";
      jobs.push({
        index,
        route,
        device,
        viewport,
        screenshot: `${String(index).padStart(3, "0")}-${name}-${device}.png`,
      });
    }
  }
  return jobs;
}

export function renderError(error: unknown): string {
  if (error instanceof AggregateError)
    return `${error.message}\n${error.errors.map(renderError).join("\n")}`;
  return error instanceof Error ? (error.stack ?? error.message) : String(error);
}

export async function measureRenderPhase<T>(
  phases: RenderPhaseTiming[],
  phase: RenderPhase,
  operation: () => Promise<T>,
): Promise<T> {
  assert(!phases.some((entry) => entry.phase === phase), `Duplicate render phase: ${phase}`);
  const startedAt = new Date().toISOString();
  const start = performance.now();
  try {
    const value = await operation();
    phases.push({ phase, startedAt, durationMs: performance.now() - start, status: "passed" });
    return value;
  } catch (error) {
    phases.push({
      phase,
      startedAt,
      durationMs: performance.now() - start,
      status: "failed",
      error: renderError(error),
    });
    throw error;
  }
}

/** Each page owns its context; cleanup must finish before a success receipt exists. */
export async function withRenderContext<Context extends { close(): Promise<void> }, T>(
  create: () => Promise<Context>,
  operation: (context: Context) => Promise<T>,
  close: (operation: () => Promise<void>) => Promise<void> = (operation) => operation(),
): Promise<T> {
  const context = await create();
  let result: T;
  let failed = false;
  let failure: unknown;
  try {
    result = await operation(context);
  } catch (error) {
    failed = true;
    failure = error;
  }
  try {
    await close(() => context.close());
  } catch (error) {
    if (failed)
      throw new AggregateError([failure, error], "Render verification and context cleanup failed");
    throw error;
  }
  if (failed) throw failure;
  return result!;
}

/** Drain all requested jobs even on failure so every route/device has an outcome. */
export async function runRenderJobs<T>(
  jobs: RenderJob[],
  workers: RenderWorkers,
  operation: (job: RenderJob, phase: MeasureRenderPhase) => Promise<T>,
): Promise<RenderJobResult<T>[]> {
  parseRenderWorkers(String(workers));
  const results: RenderJobResult<T>[] = [];
  let next = 0;
  await Promise.all(
    Array.from({ length: Math.min(workers, jobs.length) }, async (_, worker) => {
      while (next < jobs.length) {
        const position = next++;
        const job = jobs[position];
        const start = performance.now();
        const timing: RenderJobTiming = {
          index: job.index,
          route: job.route,
          device: job.device,
          worker,
          startedAt: new Date().toISOString(),
          durationMs: 0,
          status: "passed",
          phases: [],
        };
        const result: RenderJobResult<T> = { job, timing };
        try {
          result.receipt = await operation(job, (phase, run) =>
            measureRenderPhase(timing.phases, phase, run),
          );
        } catch (error) {
          timing.status = "failed";
          timing.error = renderError(error);
        } finally {
          timing.durationMs = performance.now() - start;
          results[position] = result;
        }
      }
    }),
  );
  return results;
}

/** Require exactly one outcome for every requested job and restore manifest order. */
export function collectRenderReceipts<T extends { route: string; device: string }>(
  jobs: RenderJob[],
  results: RenderJobResult<T>[],
): T[] {
  assert.equal(
    results.length,
    jobs.length,
    "Render receipt coverage must include every requested job",
  );
  const byIndex = new Map<number, RenderJobResult<T>>();
  for (const result of results) {
    assert(!byIndex.has(result.job.index), `duplicate render job ${result.job.index}`);
    byIndex.set(result.job.index, result);
  }
  return jobs.flatMap((job) => {
    const result = byIndex.get(job.index);
    assert(result, `Render receipt coverage missing job ${job.index}`);
    assert(
      result.job.route === job.route &&
        result.job.device === job.device &&
        result.timing.index === job.index &&
        result.timing.route === job.route &&
        result.timing.device === job.device,
      "Render outcome route/device must match its requested job",
    );
    if (result.timing.status === "failed") {
      assert.equal(result.receipt, undefined, "A failed render must not have a success receipt");
      return [];
    }
    assert(result.receipt, "A successful render must have a receipt");
    assert(
      result.receipt.route === job.route && result.receipt.device === job.device,
      "Render receipt route/device must match its requested job",
    );
    return [result.receipt];
  });
}

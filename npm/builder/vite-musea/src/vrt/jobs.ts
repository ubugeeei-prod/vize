import { availableParallelism } from "node:os";
import type { ArtFileInfo, ViewportConfig } from "../types/index.js";

export type VrtJob = {
  art: ArtFileInfo;
  variantName: string;
  viewport: ViewportConfig;
};

const MAX_AUTO_WORKERS = 8;

export function defaultVrtWorkerCount(parallelism = availableParallelism()): number {
  return Math.max(1, Math.min(MAX_AUTO_WORKERS, Math.floor(parallelism)));
}

export function normalizeVrtWorkerCount(workers: number | undefined): number {
  if (workers === undefined) return defaultVrtWorkerCount();
  if (!Number.isFinite(workers)) return 1;
  return Math.max(1, Math.floor(workers));
}

export function createVrtJobs(
  artFiles: ArtFileInfo[],
  defaultViewports: ViewportConfig[],
): VrtJob[] {
  const jobs: VrtJob[] = [];
  for (const art of artFiles) {
    for (const variant of art.variants) {
      if (variant.skipVrt) {
        continue;
      }

      const legacyViewport = variant.args?.viewport as ViewportConfig | undefined;
      const viewports = variant.viewport
        ? [variant.viewport]
        : legacyViewport
          ? [legacyViewport]
          : defaultViewports;

      for (const viewport of viewports) {
        jobs.push({ art, variantName: variant.name, viewport });
      }
    }
  }
  return jobs;
}

export async function runJobsWithWorkers<T>(
  jobs: readonly VrtJob[],
  workerCount: number,
  runJob: (job: VrtJob) => Promise<T | null>,
): Promise<T[]> {
  const results = Array.from<T | null>({ length: jobs.length }, () => null);
  let nextJobIndex = 0;

  async function runWorker(): Promise<void> {
    while (true) {
      const jobIndex = nextJobIndex++;
      const job = jobs[jobIndex];
      if (!job) {
        return;
      }
      results[jobIndex] = await runJob(job);
    }
  }

  const activeWorkers = Math.min(normalizeVrtWorkerCount(workerCount), jobs.length);
  await Promise.all(Array.from({ length: activeWorkers }, () => runWorker()));
  return results.filter((result): result is T => result !== null);
}

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { isRecord } from "../../../json-value.ts";
import { mib } from "./lsp-resource-sampler.ts";
import type { ObservedProcess, ProcessEntry, Sampler } from "./lsp-resource-sampler.ts";

interface ProfileResult {
  profile_json?: { path: string; bytes: number; sha256: string };
  finalization_error?: string;
}

interface SampledProfileResult extends ProfileResult {
  error?: string;
  rss_peak_mib?: number;
  peak_processes?: ProcessEntry[];
  observed_processes?: ObservedProcess[];
  resource_samples?: number;
}

interface Session {
  shutdown(): Promise<void>;
  kill(): Promise<void>;
  waitForClosed(): Promise<void>;
  readonly exitedGracefully: boolean;
}

/** Preserve the original workload failure while always restoring the harness. */
export async function finalizeLspResources(options: {
  session: Session;
  profilePath: string | null;
  workloadFailed: boolean;
  result: ProfileResult;
  cleanup(): void;
  persist(): void;
}): Promise<void> {
  let failed = false;
  let failure: unknown;
  const remember = (error: unknown): void => {
    if (!failed) failure = error;
    failed = true;
  };
  try {
    try {
      await options.session.shutdown();
    } catch (error) {
      await options.session.kill();
      // A forced termination cannot establish that the LSP exported a profile.
      if (options.profilePath) throw error;
    }
    if (options.profilePath) {
      await options.session.waitForClosed();
      assert.ok(options.session.exitedGracefully, "the profiled LSP must exit successfully");
      const bytes = fs.readFileSync(options.profilePath);
      const report: unknown = JSON.parse(bytes.toString("utf8"));
      assert.ok(isRecord(report) && report.command === "lsp", "the actual LSP exports its session");
      options.result.profile_json = {
        path: options.profilePath,
        bytes: bytes.byteLength,
        sha256: createHash("sha256").update(bytes).digest("hex"),
      };
    }
  } catch (error) {
    remember(error);
  } finally {
    try {
      options.cleanup();
    } catch (error) {
      remember(error);
    }
    if (failed) options.result.finalization_error = String(failure);
    try {
      options.persist();
    } catch (error) {
      remember(error);
    }
  }
  if (failed && !options.workloadFailed) throw failure;
}

/** Persist sampler evidence and restore the harness without replacing workload failures. */
export async function finalizeSampledLspRun<Result extends SampledProfileResult>(options: {
  sampler: Pick<Sampler, "stop">;
  measurement: { per_run: Result[]; active_run?: Result };
  result: Result;
  session: Session;
  profilePath: string | null;
  cleanup(): void;
  persist(): void;
}): Promise<void> {
  const workloadFailed = options.result.error !== undefined;
  let samplingFailure: unknown;
  try {
    const { peak, samples, observed_processes } = options.sampler.stop();
    options.result.rss_peak_mib = mib(peak.rss_bytes);
    options.result.peak_processes = peak.processes;
    options.result.observed_processes = observed_processes;
    options.result.resource_samples = samples;
  } catch (error) {
    samplingFailure = error;
    if (!workloadFailed) options.result.error = String(error);
  }

  options.measurement.per_run.push(options.result);
  delete options.measurement.active_run;
  await finalizeLspResources({
    session: options.session,
    profilePath: options.profilePath,
    // A sampler failure becomes the primary successful-workload failure. Keep
    // cleanup/profile errors as persisted evidence instead of replacing it.
    workloadFailed: workloadFailed || samplingFailure !== undefined,
    result: options.result,
    cleanup: () => options.cleanup(),
    persist: () => options.persist(),
  });
  if (samplingFailure !== undefined && !workloadFailed) throw samplingFailure;
}

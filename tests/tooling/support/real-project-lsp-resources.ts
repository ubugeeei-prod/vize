import { processTreeRss } from "../../performance/support/process-metrics.ts";

/** Peak resident memory and process count of one LSP session's whole tree. */
export type LspResourceEvidence = {
  peakRssKiB: number;
  peakProcesses: number;
  samples: number;
};

/**
 * Sample the server and every descendant (tsgo workers included) until
 * stopped. Returns null when the session has no OS process (test doubles) or
 * the host cannot list processes.
 */
export function sampleSessionResources(
  session: unknown,
  intervalMs = 1000,
): () => LspResourceEvidence | null {
  let processId: number | undefined;
  try {
    processId = (session as { processId?: number }).processId;
  } catch {
    processId = undefined;
  }
  if (typeof processId !== "number") return () => null;
  const evidence: LspResourceEvidence = { peakRssKiB: 0, peakProcesses: 0, samples: 0 };
  const sample = () => {
    const tree = processTreeRss(processId);
    if (tree == null) return;
    evidence.samples += 1;
    evidence.peakRssKiB = Math.max(evidence.peakRssKiB, tree.totalKiB);
    evidence.peakProcesses = Math.max(evidence.peakProcesses, tree.processes);
  };
  sample();
  const timer = setInterval(sample, intervalMs);
  timer.unref();
  return () => {
    clearInterval(timer);
    return evidence.samples > 0 ? evidence : null;
  };
}

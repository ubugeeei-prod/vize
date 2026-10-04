import { writeFileSync } from "node:fs";
import { executeWhole } from "./vue2-whole-sfc-oracle.ts";

export type RuntimeRequest = {
  origin: "reference-before" | "reference-after" | "native";
  index: number;
  id: string | null;
  source: string | null;
  runtimeRequested: boolean;
  unavailableReason: string | null;
};
export type RuntimeAttempt = {
  environment: "test" | "production";
  status: "completed" | "errored" | "unexecuted";
  packet: ReturnType<typeof executeWhole> | null;
  error: ReturnType<typeof observedError> | null;
  reason: string | null;
};
export type RuntimeObservation = RuntimeRequest & { attempts: RuntimeAttempt[] };
export const observedError = (error: unknown) => ({
  name: error instanceof Error ? error.name : "NonErrorThrow",
  message: error instanceof Error ? error.message : String(error),
  stack: error instanceof Error ? (error.stack ?? null) : null,
});

// Both environments execute independently; no expected comparison runs here.
export function observeRuntime(request: RuntimeRequest): RuntimeObservation {
  const attempts = (["test", "production"] as const).map((environment): RuntimeAttempt => {
    if (!request.runtimeRequested || request.source === null) {
      return {
        environment,
        status: "unexecuted",
        packet: null,
        error: null,
        reason: request.unavailableReason ?? "compiler-characterization-only control",
      };
    }
    try {
      return {
        environment,
        status: "completed",
        packet: executeWhole(request.source, environment),
        error: null,
        reason: null,
      };
    } catch (error) {
      return {
        environment,
        status: "errored",
        packet: null,
        error: observedError(error),
        reason: null,
      };
    }
  });
  return { ...request, attempts };
}

export function persistRuntimeObservations(
  referenceSha256: string,
  nativeCapture: { status: string; sha256: string | null; metadata: unknown; error: unknown },
  observations: RuntimeObservation[],
) {
  const path = process.env.VIZE_GLYPH_VUE2_SFC_RUNTIME_CAPTURE;
  if (!path) return;
  writeFileSync(
    path,
    JSON.stringify(
      {
        schema: "vize.native-vue2-scriptless-sfc-runtime-observation",
        version: 1,
        sourceHead: process.env.VIZE_GLYPH_VUE2_SFC_SOURCE_HEAD ?? null,
        executionCommit: process.env.GITHUB_SHA ?? null,
        workflowRun: process.env.GITHUB_RUN_ID ?? null,
        referenceSha256,
        nativeCapture,
        observations,
      },
      null,
      2,
    ) + "\n",
  );
}

import { performance } from "node:perf_hooks";

import type {
  NativeBatchCompiler,
  NativeBatchResult,
  NativeBatchVariant,
} from "./native-batch-types.ts";
import { isRecord } from "./script-types.ts";

function assertNativeBatchResult(
  result: unknown,
  expectedFiles: number,
): asserts result is NativeBatchResult {
  if (!isRecord(result)) {
    throw new Error("Vize native batch compile returned an invalid result.");
  }
  if (result.failed !== 0) {
    throw new Error(`Vize native batch compile failed for ${String(result.failed)} file(s).`);
  }
  if (result.success !== expectedFiles) {
    throw new Error(
      `Vize native batch compiled ${String(result.success)} files, expected ${expectedFiles}.`,
    );
  }
  if (typeof result.timeMs !== "number" || !Number.isFinite(result.timeMs) || result.timeMs <= 0) {
    throw new Error(`Vize native batch returned an invalid time: ${String(result.timeMs)}.`);
  }
}

export function measureNativeBatchCompile(
  native: NativeBatchCompiler,
  pattern: string,
  expectedFiles: number,
): number {
  const result = native.compileSfcBatch(pattern);
  assertNativeBatchResult(result, expectedFiles);
  return result.timeMs;
}

function measureThreadSequence(
  native: NativeBatchCompiler,
  pattern: string,
  expectedFiles: number,
  firstThreads: number,
  measuredThreads: number,
): number {
  const first = native.compileSfcBatch(pattern, { threads: firstThreads });
  assertNativeBatchResult(first, expectedFiles);
  const start = performance.now();
  const measured = native.compileSfcBatch(pattern, { threads: measuredThreads });
  const elapsed = performance.now() - start;
  assertNativeBatchResult(measured, expectedFiles);
  return elapsed;
}

export function createNativeBatchSequenceVariants({
  native,
  pattern,
  expectedFiles,
  maxThreads,
}: {
  native: NativeBatchCompiler;
  pattern: string;
  expectedFiles: number;
  maxThreads: number;
}): NativeBatchVariant[] {
  if (maxThreads <= 1) {
    return [];
  }

  return [
    {
      id: "vize-native-sequence-1-max",
      label: `Vize batch sequence (1→${maxThreads}T, measures ${maxThreads}T)`,
      files: expectedFiles,
      measure: () => measureThreadSequence(native, pattern, expectedFiles, 1, maxThreads),
    },
    {
      id: "vize-native-sequence-max-1",
      label: `Vize batch sequence (${maxThreads}T→1, measures 1T)`,
      files: expectedFiles,
      measure: () => measureThreadSequence(native, pattern, expectedFiles, maxThreads, 1),
    },
  ];
}

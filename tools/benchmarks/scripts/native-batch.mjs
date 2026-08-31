import { performance } from "node:perf_hooks";

function assertNativeBatchResult(result, expectedFiles) {
  if (!result || typeof result !== "object") {
    throw new Error("Vize native batch compile returned an invalid result.");
  }
  if (result.failed !== 0) {
    throw new Error(`Vize native batch compile failed for ${result.failed} file(s).`);
  }
  if (result.success !== expectedFiles) {
    throw new Error(
      `Vize native batch compiled ${result.success} files, expected ${expectedFiles}.`,
    );
  }
  if (!Number.isFinite(result.timeMs) || result.timeMs <= 0) {
    throw new Error(`Vize native batch returned an invalid time: ${result.timeMs}.`);
  }
}

export function measureNativeBatchCompile(native, pattern, expectedFiles) {
  const result = native.compileSfcBatch(pattern);
  assertNativeBatchResult(result, expectedFiles);
  return result.timeMs;
}

function measureThreadSequence(native, pattern, expectedFiles, firstThreads, measuredThreads) {
  const first = native.compileSfcBatch(pattern, { threads: firstThreads });
  assertNativeBatchResult(first, expectedFiles);
  const start = performance.now();
  const measured = native.compileSfcBatch(pattern, { threads: measuredThreads });
  const elapsed = performance.now() - start;
  assertNativeBatchResult(measured, expectedFiles);
  return elapsed;
}

export function createNativeBatchSequenceVariants({ native, pattern, expectedFiles, maxThreads }) {
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

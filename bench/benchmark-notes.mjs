export function buildFairnessNotes(fileCount) {
  const formattedFileCount = fileCount.toLocaleString("en-US");
  return [
    "All tools run on the same generated Vue SFC corpus from the same checkout and lockfile.",
    `The ${formattedFileCount}-SFC rows are the many-file workload; the large-SFC rows isolate one large component.`,
    "Reported times are medians; measured runs alternate variant order after warmup runs.",
    "Destructive formatter runs receive a fresh copy of the same input before each invocation.",
    "SFC compile Vize max uses `compileSfcBatchWithResults` wall time so the primary number includes generated output crossing the JS/native boundary; the stats-only native `timeMs` is shown only in variant details. Explicit sequence variants run 1→max and max→1 in one Node process and measure the second call's full JavaScript wall time, including scoped pool creation.",
    "Vite build timings exclude fixture copy/setup; the Vize max lane sets `precompileBatchSize` to the benchmark file count so Blacksmith max runs one native precompile batch instead of the memory-safe default chunks.",
    "Nuxt SPA build timings exclude synthetic app generation and compare `nuxt build` with Nuxt's default compiler against the same app with `@vizejs/nuxt` installed.",
    "Single-thread lanes are shown where useful, and the primary speedup compares the incumbent default/single-thread lane with Vize's max runner lane.",
  ];
}

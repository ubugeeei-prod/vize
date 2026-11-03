function median(values) {
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  if (sorted.length % 2 === 1) return sorted[mid];
  return (sorted[mid - 1] + sorted[mid]) / 2;
}

function formatThroughput(files, ms) {
  if (!Number.isFinite(ms) || ms <= 0) return "n/a";
  const filesPerSecond = (files / ms) * 1000;
  if (filesPerSecond >= 1000) return `${(filesPerSecond / 1000).toFixed(1)}k files/s`;
  return `${filesPerSecond.toFixed(0)} files/s`;
}

const round = (ms) => Number(ms.toFixed(3));

/**
 * A lane's `measure` returns its wall time in ms, or a process sample
 * `{ ms, cpuMs, rssKb }`. Process samples also publish the CPU and peak-RSS
 * medians and the first warmup as the cold run; the cold run never enters a
 * median.
 */
export async function measureVariants(variants, options, onRejected) {
  const rejected = new Set();
  const run = async (variant, phase, iteration) => {
    if (rejected.has(variant.id)) return;
    try {
      return await variant.measure({ phase, iteration });
    } catch (error) {
      if (!onRejected) throw error;
      onRejected(variant, error, phase);
      rejected.add(variant.id);
    }
  };
  const coldById = new Map();
  for (let i = 0; i < options.warmups; i++) {
    for (const variant of variants) {
      const sample = await run(variant, "warmup", i);
      if (i === 0 && typeof sample === "object" && sample != null) coldById.set(variant.id, sample);
    }
  }

  const samplesById = new Map(variants.map((variant) => [variant.id, []]));
  for (let i = 0; i < options.runs; i++) {
    const ordered = i % 2 === 0 ? variants : [...variants].reverse();
    for (const variant of ordered) {
      const sample = await run(variant, "measure", i);
      if (!rejected.has(variant.id)) samplesById.get(variant.id).push(sample);
    }
  }

  return variants
    .filter((variant) => !rejected.has(variant.id))
    .map((variant) => {
      const samples = samplesById.get(variant.id);
      const runs = samples.map((sample) => round(typeof sample === "number" ? sample : sample.ms));
      const medianMs = round(median(runs));
      const measured = {
        id: variant.id,
        label: variant.label,
        ...(variant.tool ? { tool: variant.tool, setting: variant.setting } : {}),
        medianMs,
        runs,
        throughput: formatThroughput(variant.files, medianMs),
      };
      const cold = coldById.get(variant.id);
      if (cold) {
        measured.coldMs = round(cold.ms);
        if (cold.cpuMs != null) measured.coldCpuMs = round(cold.cpuMs);
      }
      const every = (key) =>
        samples.length > 0 &&
        samples.every((sample) => typeof sample === "object" && sample[key] != null);
      if (every("cpuMs")) {
        measured.cpuRuns = samples.map((sample) => round(sample.cpuMs));
        measured.cpuMedianMs = round(median(measured.cpuRuns));
      }
      if (every("rssKb")) {
        measured.rssRuns = samples.map((sample) => sample.rssKb);
        measured.rssMedianKb = median(measured.rssRuns);
      }
      return measured;
    });
}

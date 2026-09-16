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
  for (let i = 0; i < options.warmups; i++) {
    for (const variant of variants) await run(variant, "warmup", i);
  }

  const runsById = new Map(variants.map((variant) => [variant.id, []]));
  for (let i = 0; i < options.runs; i++) {
    const ordered = i % 2 === 0 ? variants : [...variants].reverse();
    for (const variant of ordered) {
      const ms = await run(variant, "measure", i);
      if (!rejected.has(variant.id)) runsById.get(variant.id).push(ms);
    }
  }

  return variants
    .filter((variant) => !rejected.has(variant.id))
    .map((variant) => {
      const runs = runsById.get(variant.id).map((ms) => Number(ms.toFixed(3)));
      const medianMs = Number(median(runs).toFixed(3));
      return {
        id: variant.id,
        label: variant.label,
        medianMs,
        runs,
        throughput: formatThroughput(variant.files, medianMs),
      };
    });
}

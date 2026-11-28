// Packs units of CI work into matrix jobs. Every job pays its checkout, toolchain,
// sticky-disk and dependency setup once, and at the Blacksmith concurrency cap
// each extra job also waits for a runner slot. The estimates decide only which
// units share a job, never whether a unit runs.

// Longest-processing-time packing onto the least-loaded job. Ties resolve by
// declaration order and job index, so the same selection always yields the same jobs.
function pack<T>(items: readonly T[], seconds: (item: T) => number, total: number): T[][] {
  const loads = Array.from({ length: total }, () => 0);
  const jobs: number[][] = Array.from({ length: total }, () => []);
  const order = items
    .map((item, index) => ({ index, seconds: seconds(item) }))
    .sort((left, right) => right.seconds - left.seconds || left.index - right.index);
  for (const { index, seconds: cost } of order) {
    let target = 0;
    for (let job = 1; job < total; job += 1) if (loads[job]! < loads[target]!) target = job;
    loads[target]! += cost;
    jobs[target]!.push(index);
  }
  return jobs
    .filter((indexes) => indexes.length > 0)
    .map((indexes) => indexes.sort((left, right) => left - right).map((index) => items[index]!));
}

/** The fewest jobs whose estimated work stays within the budget (or within the heaviest unit). */
export function fewestJobs<T>(
  items: readonly T[],
  seconds: (item: T) => number,
  budget: number,
): T[][] {
  if (items.length === 0) return [];
  const limit = Math.max(budget, ...items.map(seconds));
  for (let total = 1; ; total += 1) {
    const jobs = pack(items, seconds, total);
    const fits = jobs.every((job) => job.reduce((sum, item) => sum + seconds(item), 0) <= limit);
    if (fits || total === items.length) return jobs;
  }
}

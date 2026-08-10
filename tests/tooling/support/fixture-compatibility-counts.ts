export function capabilityCounts(
  present: number,
  exercised: number,
  runtimeVerified: number,
  fixturePaths: string[],
) {
  return {
    present: { count: present, fixturePaths: present === 0 ? [] : fixturePaths },
    exercised: { count: exercised, fixturePaths: exercised === 0 ? [] : fixturePaths },
    runtimeVerified: {
      count: runtimeVerified,
      fixturePaths: runtimeVerified === 0 ? [] : fixturePaths,
    },
  };
}

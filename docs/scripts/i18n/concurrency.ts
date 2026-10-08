export async function mapConcurrent<T, U>(
  values: T[],
  concurrency: number,
  mapper: (value: T, index: number) => U | Promise<U>,
): Promise<U[]> {
  const results = Array.from({ length: values.length }) as U[];
  let nextIndex = 0;
  await Promise.all(
    Array.from({ length: Math.min(concurrency, values.length) }, async () => {
      while (nextIndex < values.length) {
        const index = nextIndex;
        nextIndex += 1;
        results[index] = await mapper(values[index], index);
      }
    }),
  );
  return results;
}

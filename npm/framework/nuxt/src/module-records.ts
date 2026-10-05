function isPlainRecord(value: unknown): value is Record<string, unknown> {
  return value != null && typeof value === "object" && !Array.isArray(value);
}

export function mergePlainRecords(
  ...values: Array<Record<string, unknown> | undefined>
): Record<string, unknown> {
  const result: Record<string, unknown> = {};

  for (const value of values) {
    if (!value) {
      continue;
    }
    for (const [key, nextValue] of Object.entries(value)) {
      const currentValue = result[key];
      result[key] =
        isPlainRecord(currentValue) && isPlainRecord(nextValue)
          ? mergePlainRecords(currentValue, nextValue)
          : nextValue;
    }
  }

  return result;
}

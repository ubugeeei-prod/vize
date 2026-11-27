/** Run `task` and return how many `Intl.DateTimeFormat` instances it constructed. */
export function countDateTimeFormats(task: () => void): number {
  const original = Intl.DateTimeFormat;
  let constructed = 0;
  Intl.DateTimeFormat = new Proxy(original, {
    construct(target, argumentList, newTarget): object {
      constructed += 1;
      return Reflect.construct(target, argumentList, newTarget) as object;
    },
  });
  try {
    task();
  } finally {
    Intl.DateTimeFormat = original;
  }
  return constructed;
}

/** Up to 128 distinct spellings of one zone id; zone ids match case-insensitively. */
export function timeZoneSpellings(count: number): string[] {
  const letters = "america".split("");
  return Array.from({ length: count }, (_, index) => {
    const region = letters.map((letter, position) =>
      ((index >> position) & 1) === 1 ? letter.toUpperCase() : letter,
    );
    return `${region.join("")}/Chicago`;
  });
}

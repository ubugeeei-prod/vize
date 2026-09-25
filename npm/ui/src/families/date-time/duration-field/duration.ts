/** Duration units edited by DurationField, largest first. */
export type DurationUnit = "years" | "months" | "weeks" | "days" | "hours" | "minutes" | "seconds";

/** Non-negative whole-unit duration. Omitted units are zero. */
export type DurationValue = { readonly [Unit in DurationUnit]?: number };

/** ISO 8601 duration text such as `P1DT2H30M`. */
export type IsoDuration = `P${string}`;

/** Canonical largest-to-smallest unit order. */
export const durationUnits: readonly DurationUnit[] = Object.freeze([
  "years",
  "months",
  "weeks",
  "days",
  "hours",
  "minutes",
  "seconds",
]);

const dateDesignators: Readonly<Partial<Record<DurationUnit, string>>> = {
  years: "Y",
  months: "M",
  weeks: "W",
  days: "D",
};
const timeDesignators: Readonly<Partial<Record<DurationUnit, string>>> = {
  hours: "H",
  minutes: "M",
  seconds: "S",
};
const isoDurationPattern =
  /^P(?:(\d+)Y)?(?:(\d+)M)?(?:(\d+)W)?(?:(\d+)D)?(?:T(?:(\d+)H)?(?:(\d+)M)?(?:(\d+)S)?)?$/u;

function isWholeUnit(value: unknown): value is number {
  return typeof value === "number" && Number.isSafeInteger(value) && value >= 0;
}

/** Copy a duration record, dropping zero and invalid units; invalid input returns `null`. */
export function normalizeDuration(value: DurationValue | null | undefined): DurationValue | null {
  if (typeof value !== "object" || value === null) return null;
  const result: { [Unit in DurationUnit]?: number } = {};
  for (const unit of durationUnits) {
    const amount = value[unit];
    if (amount === undefined) continue;
    if (!isWholeUnit(amount)) return null;
    if (amount > 0) result[unit] = amount;
  }
  return Object.freeze(result);
}

/** Whether two nullable durations have identical units. */
export function isSameDuration(
  left: DurationValue | null | undefined,
  right: DurationValue | null | undefined,
): boolean {
  if (!left || !right) return !left && !right;
  return durationUnits.every((unit) => (left[unit] ?? 0) === (right[unit] ?? 0));
}

/** Format an ISO 8601 duration; the zero duration is `PT0S`. */
export function formatIsoDuration(value: DurationValue): IsoDuration {
  const normalized = normalizeDuration(value) ?? {};
  let date = "";
  let time = "";
  for (const unit of durationUnits) {
    const amount = normalized[unit];
    if (!amount) continue;
    const dateDesignator = dateDesignators[unit];
    if (dateDesignator) date += `${amount}${dateDesignator}`;
    else time += `${amount}${timeDesignators[unit] ?? ""}`;
  }
  if (!date && !time) return "PT0S";
  return `P${date}${time ? `T${time}` : ""}`;
}

/** Parse a whole-unit ISO 8601 duration; fractions, signs, and malformed text return `null`. */
export function parseIsoDuration(value: string): DurationValue | null {
  const text = value.trim().toUpperCase();
  const match = isoDurationPattern.exec(text);
  if (!match || text === "P" || text.endsWith("T")) return null;
  const result: { [Unit in DurationUnit]?: number } = {};
  durationUnits.forEach((unit, index) => {
    const amount = match[index + 1];
    if (amount !== undefined) result[unit] = Number(amount);
  });
  return normalizeDuration(result);
}

/**
 * Total seconds for durations with only week-and-smaller units.
 *
 * Years and months have no fixed length, so durations using them return `null`.
 */
export function durationToSeconds(value: DurationValue): number | null {
  if ((value.years ?? 0) > 0 || (value.months ?? 0) > 0) return null;
  return (
    (value.weeks ?? 0) * 604_800 +
    (value.days ?? 0) * 86_400 +
    (value.hours ?? 0) * 3_600 +
    (value.minutes ?? 0) * 60 +
    (value.seconds ?? 0)
  );
}

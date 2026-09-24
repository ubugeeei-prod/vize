// Tick generation follows the d3-array 3 algorithm (ISC licensed) so tick values
// are bit-for-bit identical to d3's, which the test vectors assert.

const e10 = Math.sqrt(50);
const e5 = Math.sqrt(10);
const e2 = Math.sqrt(2);

type TickSpec = readonly [i1: number, i2: number, increment: number];

function tickSpec(start: number, stop: number, count: number): TickSpec {
  const step = (stop - start) / Math.max(0, count);
  const power = Math.floor(Math.log10(step));
  const error = step / 10 ** power;
  const factor = error >= e10 ? 10 : error >= e5 ? 5 : error >= e2 ? 2 : 1;
  let i1: number;
  let i2: number;
  let increment: number;
  if (power < 0) {
    increment = 10 ** -power / factor;
    i1 = Math.round(start * increment);
    i2 = Math.round(stop * increment);
    if (i1 / increment < start) ++i1;
    if (i2 / increment > stop) --i2;
    increment = -increment;
  } else {
    increment = 10 ** power * factor;
    i1 = Math.round(start / increment);
    i2 = Math.round(stop / increment);
    if (i1 * increment < start) ++i1;
    if (i2 * increment > stop) --i2;
  }
  if (i2 < i1 && 0.5 <= count && count < 2) return tickSpec(start, stop, count * 2);
  return [i1, i2, increment];
}

/**
 * Return about `count` human-friendly values (multiples of 1, 2, or 5 times a
 * power of ten) inside `[start, stop]`, in the same direction as the input.
 */
export function ticks(start: number, stop: number, count: number): number[] {
  if (!(count > 0)) return [];
  if (start === stop) return [start];
  const reverse = stop < start;
  const [i1, i2, increment] = reverse ? tickSpec(stop, start, count) : tickSpec(start, stop, count);
  if (!(i2 >= i1)) return [];
  const n = i2 - i1 + 1;
  const result: number[] = [];
  for (let i = 0; i < n; ++i) {
    const k = reverse ? i2 - i : i1 + i;
    result.push(increment < 0 ? k / -increment : k * increment);
  }
  return result;
}

/**
 * Tick increment for `[start, stop]`. Positive values are the step; negative
 * values are the inverse step (`-10` means `0.1`), which avoids float error.
 */
export function tickIncrement(start: number, stop: number, count: number): number {
  return tickSpec(start, stop, count)[2];
}

/** Signed distance between adjacent ticks for `[start, stop]`. */
export function tickStep(start: number, stop: number, count: number): number {
  const reverse = stop < start;
  const increment = reverse ? tickIncrement(stop, start, count) : tickIncrement(start, stop, count);
  return (reverse ? -1 : 1) * (increment < 0 ? 1 / -increment : increment);
}

/**
 * Extend `[start, stop]` outward to tick-aligned values, iterating like d3's
 * `linear.nice` until the tick increment stabilizes. Direction is preserved.
 */
export function niceExtent(start: number, stop: number, count = 10): [number, number] {
  const reverse = stop < start;
  let lo = reverse ? stop : start;
  let hi = reverse ? start : stop;
  let previous: number | undefined;
  for (let iteration = 0; iteration < 10; iteration++) {
    const step = tickIncrement(lo, hi, count);
    if (step === previous) return reverse ? [hi, lo] : [lo, hi];
    if (step > 0) {
      lo = Math.floor(lo / step) * step;
      hi = Math.ceil(hi / step) * step;
    } else if (step < 0) {
      lo = Math.ceil(lo * step) / step;
      hi = Math.floor(hi * step) / step;
    } else {
      break;
    }
    previous = step;
  }
  // Like d3, a degenerate or non-converging extent is left untouched.
  return [start, stop];
}

/** Decimal exponent of `value`, as in scientific notation (`0.05` → `-2`). */
export function decimalExponent(value: number): number {
  if (!Number.isFinite(value) || value === 0) return 0;
  return Number(Math.abs(value).toExponential().split("e")[1]);
}

/** Fraction digits needed to distinguish ticks `step` apart (d3 `precisionFixed`). */
export function precisionFixed(step: number): number {
  return Math.max(0, -decimalExponent(step));
}

/** Index of the first element greater than `value` in an ascending array. */
export function bisectRight(
  values: readonly number[],
  value: number,
  lo = 0,
  hi = values.length,
): number {
  let low = lo;
  let high = hi;
  while (low < high) {
    const mid = (low + high) >>> 1;
    const candidate = values[mid];
    if (candidate !== undefined && candidate <= value) low = mid + 1;
    else high = mid;
  }
  return low;
}

import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  niceExtent,
  precisionFixed,
  scaleBand,
  scaleLinear,
  scaleLog,
  scaleOrdinal,
  scalePoint,
  scalePow,
  scaleSqrt,
  scaleTime,
  tickIncrement,
  ticks,
  tickStep,
} from "./chart-scale.ts";
import { d3ScaleVectors as vectors } from "./scale-d3-vectors.ts";

/** JSON cannot carry NaN; the vectors store it as `null`. */
function expected(value: number | null): number {
  return value === null ? Number.NaN : value;
}

function sameNumbers(
  actual: readonly number[],
  reference: readonly (number | null)[],
  label: string,
) {
  assert.deepEqual(actual, reference.map(expected), label);
}

test("ticks, tickIncrement, and tickStep match d3-array exactly", () => {
  for (const [start, stop, count, reference] of vectors.ticks) {
    sameNumbers(ticks(start, stop, count), reference, `ticks(${start}, ${stop}, ${count})`);
  }
  for (const [start, stop, count, reference] of vectors.tickIncrement) {
    assert.equal(tickIncrement(start, stop, count), reference);
  }
  for (const [start, stop, count, reference] of vectors.tickStep) {
    assert.equal(tickStep(start, stop, count), reference);
  }
  assert.deepEqual(ticks(0, 1, 0), []);
  assert.deepEqual(ticks(3, 3, 5), [3]);
  assert.deepEqual(ticks(0, 1, Number.NaN), []);
});

test("linear scales map, invert, clamp, round, and tick like d3", () => {
  for (const reference of vectors.linear) {
    const scale = scaleLinear({
      domain: reference.domain,
      range: reference.range,
      clamp: "clamp" in reference ? reference.clamp : false,
      round: "round" in reference ? reference.round : false,
    });
    const label = JSON.stringify(reference.domain);
    sameNumbers(reference.inputs.map(scale), reference.outputs, `outputs ${label}`);
    sameNumbers(reference.range.map(scale.invert), reference.inverts, `inverts ${label}`);
    sameNumbers(scale.ticks(5), reference.ticks, `ticks ${label}`);
  }
  const scale = scaleLinear({ domain: [0, 10], range: [0, 100], unknown: -1 });
  assert.equal(scale(null), -1);
  assert.equal(scale(undefined), -1);
  assert.equal(scale(Number.NaN), -1);
});

test("nice domains iterate to a stable tick increment like d3", () => {
  for (const [start, stop, count, reference] of vectors.linearNice) {
    assert.deepEqual(niceExtent(start, stop, count), reference, `nice(${start}, ${stop})`);
    assert.deepEqual([...scaleLinear({ domain: [start, stop], nice: count }).domain], reference);
  }
  const scale = scaleLinear({ domain: [0.13, 9.87] });
  assert.deepEqual([...scale.nice().domain], [0, 10]);
  assert.deepEqual([...scale.domain], [0.13, 9.87], "nice returns a copy");
  assert.deepEqual([...scaleLinear({ domain: [0.2, 5, 9.7], nice: true }).domain], [0, 5, 10]);
});

test("number tick formats match d3 ',f' precision through Intl", () => {
  for (const [start, stop, count, reference] of vectors.linearFormat) {
    const scale = scaleLinear({ domain: [start, stop] });
    const format = scale.tickFormat(count);
    assert.deepEqual(
      scale.ticks(count).map(format),
      reference.map((label) => label.replaceAll("−", "-")),
    );
  }
  const percent = scaleLinear({ domain: [0, 1] }).tickFormat(10, { format: { style: "percent" } });
  assert.equal(percent(0.3), "30%");
  const german = scaleLinear({ domain: [0, 1] }).tickFormat(10, { locale: "de-DE" });
  assert.equal(german(0.5), "0,5");
  assert.equal(precisionFixed(0.05), 2);
  assert.equal(precisionFixed(200), 0);
});

test("pow and sqrt scales follow d3's signed power transform", () => {
  for (const reference of vectors.pow) {
    const scale = scalePow({
      domain: reference.domain,
      exponent: reference.exponent,
      range: reference.range,
    });
    sameNumbers(reference.inputs.map(scale), reference.outputs, `pow ${reference.exponent}`);
    sameNumbers(
      reference.range.map(scale.invert),
      reference.inverts,
      `pow invert ${reference.exponent}`,
    );
    sameNumbers(scale.ticks(4), reference.ticks, `pow ticks ${reference.exponent}`);
    assert.equal(scale.exponent, reference.exponent);
  }
  const sqrt = scaleSqrt({ domain: [0, 100], range: [0, 10] });
  assert.equal(sqrt(25), 5);
  assert.equal(sqrt.kind, "pow");
  assert.equal(sqrt.with({ range: [0, 20] })(25), 10);
});

test("log scales map, tick, nice, and blank minor labels like d3", () => {
  for (const reference of vectors.log) {
    const scale = scaleLog({
      base: reference.base,
      domain: reference.domain,
      range: reference.range,
    });
    const label = `log${reference.base} ${JSON.stringify(reference.domain)}`;
    sameNumbers(reference.inputs.map(scale), reference.outputs, `outputs ${label}`);
    sameNumbers(reference.range.map(scale.invert), reference.inverts, `inverts ${label}`);
    sameNumbers(scale.ticks(), reference.ticks, `ticks ${label}`);
    sameNumbers(scale.ticks(3), reference.ticks3, `ticks(3) ${label}`);
    sameNumbers([...scale.nice().domain], reference.nice, `nice ${label}`);
    const format = scale.tickFormat(5);
    assert.deepEqual(
      scale.ticks().map((value) => format(value) !== ""),
      reference.labelled,
      `labels ${label}`,
    );
  }
  assert.equal(scaleLog({ domain: [1, 1000] }).tickFormat(5)(1000), "1K");
});

test("band and point scales lay out categories like d3", () => {
  for (const reference of vectors.band) {
    const scale = scaleBand({
      domain: reference.domain,
      range: [reference.range[0], reference.range[1]],
      ...("padding" in reference ? { padding: reference.padding } : {}),
      ...("paddingInner" in reference ? { paddingInner: reference.paddingInner } : {}),
      ...("paddingOuter" in reference ? { paddingOuter: reference.paddingOuter } : {}),
      ...("align" in reference ? { align: reference.align } : {}),
      ...("round" in reference ? { round: reference.round } : {}),
    });
    const label = JSON.stringify(reference);
    sameNumbers(reference.domain.map(scale), reference.positions, `positions ${label}`);
    assert.equal(scale.bandwidth, expected(reference.bandwidth), `bandwidth ${label}`);
    assert.equal(scale.step, expected(reference.step), `step ${label}`);
  }
  for (const reference of vectors.point) {
    const scale = scalePoint({
      domain: reference.domain,
      range: [reference.range[0], reference.range[1]],
      ...("padding" in reference ? { padding: reference.padding } : {}),
    });
    sameNumbers(reference.domain.map(scale), reference.positions, JSON.stringify(reference));
    assert.equal(scale.step, expected(reference.step));
    assert.equal(scale.bandwidth, 0);
  }
  const band = scaleBand({ domain: ["a", "b", "a"], range: [0, 100] });
  assert.deepEqual([...band.domain], ["a", "b"], "duplicate categories are ignored");
  assert.ok(Number.isNaN(band("z" as "a")), "unknown categories map to NaN");
  assert.deepEqual(band.ticks(), ["a", "b"]);
  const dates = scaleBand({ domain: [new Date(0), new Date(1000)], range: [0, 10] });
  assert.equal(dates(new Date(1000)), 5, "dates compare by timestamp");
});

test("ordinal scales cycle the range without mutating the domain", () => {
  const scale = scaleOrdinal({ domain: ["a", "b", "c"], range: [1, 2] });
  assert.deepEqual(["a", "b", "c", "a"].map(scale), vectors.ordinal);
  const implicit = scaleOrdinal<string, string>({ range: ["red", "blue"] });
  assert.equal(implicit("x"), "red");
  assert.equal(implicit("y"), "blue");
  assert.equal(implicit("x"), "red", "first-seen order is stable");
  assert.deepEqual([...implicit.domain], []);
  const withUnknown = scaleOrdinal({ domain: ["a"], range: ["red"], unknown: "gray" });
  assert.equal(withUnknown("b"), "gray");
  assert.throws(() => scaleOrdinal({ range: [] })("a"), /VIZE_UI_SCALE_EMPTY_RANGE/);
});

test("UTC time ticks, nice domains, and label granularity match d3 scaleUtc", () => {
  for (const [start, stop, count, reference, nice, granularity] of vectors.utc) {
    const scale = scaleTime({ domain: [start, stop] });
    const label = `${new Date(start).toISOString()}..${new Date(stop).toISOString()} /${count}`;
    assert.deepEqual(scale.ticks(count).map(Number), reference, `ticks ${label}`);
    assert.deepEqual(scale.nice(count).domain.map(Number), nice, `nice ${label}`);
    assert.deepEqual(
      scale.ticks(count).map(scale.tickGranularity),
      granularity,
      `granularity ${label}`,
    );
  }
});

test("zoned time ticks match d3 local scaleTime run under that TZ", () => {
  for (const [zone, cases] of [
    ["America/New_York", vectors.localNewYork],
    ["Asia/Tokyo", vectors.localTokyo],
  ] as const) {
    assert.equal(cases.tz, zone);
    for (const [start, stop, count, reference, nice, granularity] of cases.cases) {
      const scale = scaleTime({ domain: [start, stop], timeZone: zone });
      const label = `${zone} ${new Date(start).toISOString()} /${count}`;
      assert.deepEqual(scale.ticks(count).map(Number), reference, `ticks ${label}`);
      assert.deepEqual(scale.nice(count).domain.map(Number), nice, `nice ${label}`);
      assert.deepEqual(
        scale.ticks(count).map(scale.tickGranularity),
        granularity,
        `granularity ${label}`,
      );
    }
  }
});

test("time scales map dates, invert, and format ticks with Intl in their zone", () => {
  const start = Date.UTC(2024, 0, 1);
  const scale = scaleTime({ domain: [new Date(start), start + 86_400_000], range: [0, 240] });
  assert.equal(scale(new Date(start + 43_200_000)), 120);
  assert.equal(scale(start + 21_600_000), 60);
  assert.equal(scale.invert(60).toISOString(), "2024-01-01T06:00:00.000Z");
  assert.ok(Number.isNaN(scale(new Date(Number.NaN))));
  assert.equal(scale.timeZone, "UTC");
  const format = scale.tickFormat();
  assert.equal(format(new Date(Date.UTC(2024, 0, 1))), "2024");
  assert.equal(format(new Date(Date.UTC(2024, 4, 1))), "May");
  assert.equal(format(new Date(Date.UTC(2024, 4, 5))), "May 5");
  assert.equal(format(new Date(Date.UTC(2024, 4, 6))), "6 Mon");
  assert.equal(format(new Date(Date.UTC(2024, 4, 6, 15))), "3 PM");
  const tokyo = scaleTime({ domain: [start, start + 1], timeZone: "Asia/Tokyo" });
  assert.equal(
    tokyo.tickFormat()(new Date(Date.UTC(2023, 11, 31, 15))),
    "2024",
    "midnight in Tokyo",
  );
  const japanese = tokyo.tickFormat({ locale: "ja-JP" });
  assert.equal(japanese(new Date(Date.UTC(2024, 3, 30, 15))), "5月");
  assert.throws(() => scaleTime({ timeZone: "Mars/Olympus" }), RangeError);
});

test("scales are immutable and expose their options", () => {
  const scale = scaleLinear({ domain: [0, 10], range: [0, 1] });
  const wider = scale.with({ range: [0, 2] });
  assert.equal(scale(5), 0.5);
  assert.equal(wider(5), 1);
  assert.deepEqual(scale.options(), { domain: [0, 10], nice: false, range: [0, 1] });
  assert.ok(Object.isFrozen(scale.domain));
  const log = scaleLog({ domain: [1, 100] }).with({ base: 2 });
  assert.equal(log.base, 2);
  const time = scaleTime({ domain: [0, 1000] }).with({ timeZone: "Europe/Paris" });
  assert.equal(time.timeZone, "Europe/Paris");
});

test("scales snapshot mutable caller options and returned time domains", () => {
  const range = [0, 10];
  const linear = scaleLinear({ domain: [0, 10], range });
  const logarithmic = scaleLog({ domain: [1, 10], range });
  const start = new Date(Date.UTC(2024, 0, 1));
  const time = scaleTime({ domain: [start, new Date(Date.UTC(2024, 0, 2))], range });
  range[1] = 100;
  start.setUTCFullYear(2030);

  assert.deepEqual(linear.options().range, [0, 10]);
  assert.deepEqual(logarithmic.options().range, [0, 10]);
  assert.deepEqual(time.options().range, [0, 10]);
  assert.equal(linear.with({ clamp: true })(10), 10);
  assert.equal(logarithmic.with({ clamp: true })(10), 10);
  assert.equal(time.with({ clamp: true })(Date.UTC(2024, 0, 2)), 10);

  const changedOptions = linear.options() as { range: number[] };
  changedOptions.range = [0, 100];
  assert.deepEqual(linear.options().range, [0, 10]);
  const firstDomain = time.domain[0];
  assert.ok(firstDomain);
  firstDomain.setUTCFullYear(2030);
  assert.equal(time.domain[0]?.getUTCFullYear(), 2024);
  assert.equal(time.with({ clamp: true }).domain[0]?.getUTCFullYear(), 2024);

  const category = new Date(0);
  const band = scaleBand({ domain: [category], range: [0, 10] });
  const ordinal = scaleOrdinal({ domain: [category], range: ["first"] });
  category.setTime(1000);
  band.domain[0]?.setTime(2000);
  ordinal.domain[0]?.setTime(2000);
  band.ticks()[0]?.setTime(2000);
  assert.equal(band(new Date(0)), 0);
  assert.equal(band.domain[0]?.getTime(), 0);
  assert.equal(band.with({}).domain[0]?.getTime(), 0);
  assert.equal(ordinal(new Date(0)), "first");
  assert.equal(ordinal.domain[0]?.getTime(), 0);
  assert.equal(ordinal.with({}).domain[0]?.getTime(), 0);
});

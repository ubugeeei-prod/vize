import assert from "node:assert/strict";
import test from "node:test";
import { parseRawMemorySamples } from "../../tools/benchmarks/scripts/cold-typechecker-scale-memory-report.ts";

const raw = [
  "PeriodType: space bytes",
  "Period: 524288",
  "Samples:",
  "alloc_objects/count alloc_space/bytes inuse_objects/count inuse_space/bytes[dflt]",
  " 2 200 1 100: 1 2 1",
  "                label:雪🌸",
  " 3 300 0 0: 2",
  "Locations",
  "1: location",
  "",
].join("\n");
test("memory raw denominator retains all four ordered weights, stack IDs and raw labels", () => {
  const result = parseRawMemorySamples(raw);
  assert.deepEqual(result.totals, ["5", "500", "1", "100"]);
  assert.deepEqual(result.samples, [
    {
      values: ["2", "200", "1", "100"],
      locations: ["1", "2", "1"],
      rawLabels: ["                label:雪🌸"],
    },
    { values: ["3", "300", "0", "0"], locations: ["2"], rawLabels: [] },
  ]);
  const zeroInuse = parseRawMemorySamples(raw.replace("2 200 1 100", "2 200 0 0"));
  assert.deepEqual(zeroInuse.totals, ["5", "500", "0", "0"]);
});
test("memory raw refuses wrong type/unit, missing/extra/negative weights, unknown rows and missing boundaries", () => {
  for (const mutant of [
    raw.replace("alloc_space/bytes", "alloc_space/count"),
    raw.replace("inuse_objects/count", "alloc_objects/count"),
    raw.replace("2 200 1 100:", "2 200 1:"),
    raw.replace("2 200 1 100:", "2 200 1 100 9:"),
    raw.replace("2 200 1 100:", "-2 200 1 100:"),
    raw.replace(" 3 300 0 0: 2", "                -3 300 0 0: 2"),
    raw.replace(" 3 300 0 0: 2", "                unexplained-token"),
    raw.replace(" 3 300 0 0: 2", "unexplained nonlabel bytes"),
    raw.replace("Samples:", "not-samples"),
    raw.replace("Locations", "not-locations"),
    raw.replace("Locations", "Locations\nLocations"),
    raw.replace(" 2 200 1 100:", " 0 0 1 100:").replace(" 3 300 0 0:", " 0 0 0 0:"),
  ])
    assert.throws(() => parseRawMemorySamples(mutant));
});

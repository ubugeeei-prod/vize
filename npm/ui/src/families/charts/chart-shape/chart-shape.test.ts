import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  arcCentroid,
  arcPath,
  areaPath,
  barRects,
  createPath,
  curveCatmullRom,
  linePath,
  pieLayout,
  roundedRectPath,
  stackLayout,
} from "./chart-shape.ts";
import type { CurveFactory, CurveName, StackOffset, StackOrder } from "./chart-shape.ts";
import { d3ShapeVectors as vectors } from "./shape-d3-vectors.ts";

type Point = readonly [number, number | null];

const curveCases: readonly (readonly [keyof typeof vectors.lines, CurveName | CurveFactory])[] = [
  ["linear", "linear"],
  ["monotoneX", "monotoneX"],
  ["step", "step"],
  ["stepBefore", "stepBefore"],
  ["stepAfter", "stepAfter"],
  ["catmullRom", "catmullRom"],
  ["catmullRom0", curveCatmullRom(0)],
  ["catmullRom1", curveCatmullRom(1)],
];

const x = (point: Point) => point[0];
const y = (point: Point) => point[1] ?? Number.NaN;
const defined = (point: Point) => point[1] !== null;

test("line paths match d3-shape for every curve", () => {
  for (const [name, curve] of curveCases) {
    assert.equal(linePath(vectors.points, { x, y, curve }), vectors.lines[name], `line ${name}`);
  }
});

test("area paths close the baseline in reverse like d3-shape", () => {
  for (const [name, curve] of curveCases) {
    assert.equal(
      areaPath(vectors.points, { x, y0: 50, y1: y, curve }),
      vectors.areas[name],
      `area ${name}`,
    );
  }
});

test("undefined data split lines and areas into segments like d3-shape", () => {
  for (const [name, curve] of curveCases) {
    assert.equal(
      linePath(vectors.gap, { x, y, defined, curve }),
      vectors.gapLines[name],
      `gap line ${name}`,
    );
    assert.equal(
      areaPath(vectors.gap, { x, y0: () => 0, y1: y, defined, curve }),
      vectors.gapAreas[name],
      `gap area ${name}`,
    );
  }
});

test("degenerate inputs and rounding digits match d3-shape", () => {
  for (const [name, curve] of curveCases) {
    const [single, pair, empty] = vectors.singles[name];
    assert.equal(linePath([[1, 2] as const], { x, y, curve }), single, `single ${name}`);
    assert.equal(
      linePath([[1, 2] as const, [3, 4] as const], { x, y, curve }),
      pair,
      `pair ${name}`,
    );
    assert.equal(linePath<Point>([], { x, y, curve }), empty, `empty ${name}`);
  }
  assert.equal(
    linePath([[0, 1 / 3] as const, [2 / 3, 1] as const], { x, y, digits: null }),
    vectors.fullPrecision,
  );
  assert.equal(
    linePath([[0.4, 1.6] as const, [2.5, 3.49] as const], { x, y, digits: 0 }),
    vectors.zeroDigits,
  );
  assert.throws(() => createPath(-1), /VIZE_UI_PATH_DIGITS/);
  assert.throws(() => createPath().arc(0, 0, -1, 0, 1), /VIZE_UI_PATH_RADIUS/);
});

test("arcs, donuts, padding, and rounded corners match d3-shape", () => {
  for (const [geometry, path, centroid] of vectors.arcs) {
    const label = JSON.stringify(geometry);
    assert.equal(arcPath(geometry), path, `arc ${label}`);
    // JSON drops the sign of zero, so normalize -0 before comparing.
    assert.deepEqual(
      arcCentroid(geometry).map((value) => value + 0),
      centroid,
      `centroid ${label}`,
    );
  }
});

test("pie layouts match d3-shape angles, ordering, and padding", () => {
  const data = [{ v: 3 }, { v: 1 }, { v: 4 }, { v: 1 }, { v: 5 }, { v: 0 }, { v: -2 }];
  const strip = <T>(slices: readonly { readonly data: T }[]) =>
    slices.map(({ data: _data, ...rest }) => rest);
  assert.deepEqual(strip(pieLayout(data, { value: (d) => d.v })), vectors.pies.default);
  assert.deepEqual(strip(pieLayout(data, { value: (d) => d.v, sort: "none" })), vectors.pies.none);
  assert.deepEqual(
    strip(
      pieLayout(data, {
        value: (d) => d.v,
        padAngle: 0.02,
        startAngle: -Math.PI / 2,
        endAngle: Math.PI / 2,
      }),
    ),
    vectors.pies.padded,
  );
  assert.deepEqual(
    strip(pieLayout(data, { value: (d) => d.v, startAngle: Math.PI, endAngle: 0 })),
    vectors.pies.reversed,
  );
  const sorted = pieLayout(data, { value: (d) => d.v, sort: (a, b) => a.v - b.v });
  assert.equal(sorted[6]?.index, 0, "comparators sort data ascending");
  assert.equal(sorted[0]?.data, data[0], "slices keep input order and data");
});

test("stack layouts match d3-shape for every order and offset", () => {
  type Row = (typeof vectors.rows)[number];
  const keys = ["a", "b", "c"] as const;
  for (const [name, reference] of Object.entries(vectors.stacks)) {
    const [order, offset] = name.split("/") as [StackOrder, StackOffset];
    const series = stackLayout<Row, (typeof keys)[number]>(vectors.rows, {
      keys,
      offset,
      order,
      value: (row, key) => row[key],
    });
    assert.deepEqual(
      series.map((entry) => ({
        key: entry.key,
        index: entry.index,
        points: entry.map((point) => [point[0], point[1]]),
      })),
      reference.map((entry) => ({
        ...entry,
        points: entry.points.map(([lo, hi]) => [lo ?? Number.NaN, hi ?? Number.NaN]),
      })),
      name,
    );
    assert.equal(series[0]?.[0]?.data, vectors.rows[0]);
  }
});

test("bar rectangles grow from the baseline in both orientations", () => {
  const category = Object.assign((value: string) => (value === "a" ? 0 : 50), { bandwidth: 40 });
  const value = (input: number) => 100 - input;
  const rows = [
    { name: "a", total: 30 },
    { name: "b", total: -20 },
  ];
  const vertical = barRects(rows, {
    category: (row) => row.name,
    categoryScale: category,
    value: (row) => row.total,
    valueScale: value,
  });
  assert.deepEqual(
    vertical.map(({ x: left, y: top, width, height }) => [left, top, width, height]),
    [
      [0, 70, 40, 30],
      [50, 100, 40, 20],
    ],
  );
  const horizontal = barRects(rows, {
    category: (row) => row.name,
    categoryScale: category,
    orientation: "horizontal",
    value: (row) => row.total,
    valueScale: (input) => input * 2,
    baseline: 10,
  });
  assert.deepEqual(
    horizontal.map(({ x: left, y: top, width, height }) => [left, top, width, height]),
    [
      [20, 0, 40, 40],
      [-40, 50, 60, 40],
    ],
  );
  assert.equal(horizontal[1]?.value, -20);
});

test("rounded rectangles clamp radii to the rectangle", () => {
  assert.equal(
    roundedRectPath({ x: 0, y: 0, width: 10, height: 20 }, 0),
    "M0,0L10,0L10,20L0,20L0,0Z",
  );
  assert.equal(
    roundedRectPath({ x: 0, y: 0, width: 10, height: 20 }, { topLeft: 4, topRight: 4 }),
    "M4,0L6,0A4,4,0,0,1,10,4L10,20L0,20L0,4A4,4,0,0,1,4,0Z",
  );
  assert.equal(
    roundedRectPath({ x: 0, y: 0, width: 10, height: 10 }, 50),
    "M5,0L5,0A5,5,0,0,1,10,5L10,5A5,5,0,0,1,5,10L5,10A5,5,0,0,1,0,5L0,5A5,5,0,0,1,5,0Z",
  );
});

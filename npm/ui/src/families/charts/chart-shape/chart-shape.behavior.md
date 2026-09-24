# Chart Shape Behavior Contract

Normative input -> outcome table for `@vizejs/ui/chart-shape`: line and area
paths with linear, monotoneX, step, and Catmull–Rom curves, arcs (pie, donut,
padded, rounded), pie and stack layouts, and bar geometry. Every row is proven
by the named test in `chart-shape.test.ts` or `chart-shape-ssr.test.ts`;
compile-only guarantees live in `chart-shape.types.test-d.ts`.

The generators are ports of d3-shape 3 and d3-path 3 (ISC licensed) and emit
byte-identical `d` strings, including d3-shape's default 3-digit rounding. The
fixed reference vectors live in `shape-d3-vectors.ts`; there is no runtime or
test dependency on d3.

| #   | Input                               | Outcome                                                                        | Proven by                                                          |
| --- | ----------------------------------- | ------------------------------------------------------------------------------ | ------------------------------------------------------------------ |
| H1  | `linePath` for every curve          | identical to d3-shape `line().curve(...)`                                      | `line paths match d3-shape for every curve`                        |
| H2  | `areaPath` for every curve          | identical to d3-shape `area()` including the reversed baseline                 | `area paths close the baseline in reverse like d3-shape`           |
| H3  | `defined` gaps                      | lines and areas split into segments exactly like d3-shape                      | `undefined data split lines and areas into segments like d3-shape` |
| H4  | single point, pair, empty, `digits` | degenerate output and rounding identical to d3; invalid digits and radii throw | `degenerate inputs and rounding digits match d3-shape`             |
| H5  | `arcPath`, `arcCentroid`            | circles, annuli, padded and rounded sectors identical to d3-shape `arc()`      | `arcs, donuts, padding, and rounded corners match d3-shape`        |
| H6  | `pieLayout`                         | angles, value sorting, input order, padding, and reversed sweeps match d3      | `pie layouts match d3-shape angles, ordering, and padding`         |
| H7  | `stackLayout`                       | all 6 orders × 5 offsets match d3-shape                                        | `stack layouts match d3-shape for every order and offset`          |
| H8  | `barRects`                          | non-negative rectangles from a baseline, vertical and horizontal               | `bar rectangles grow from the baseline in both orientations`       |
| H9  | `roundedRectPath`                   | per-corner radii clamped to half the shorter side                              | `rounded rectangles clamp radii to the rectangle`                  |
| H10 | SSR and hydration                   | generated paths render byte-identically and hydrate without warnings           | `chart-shape-ssr.test.ts`                                          |

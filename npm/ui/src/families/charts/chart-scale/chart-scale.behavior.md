# Chart Scale Behavior Contract

Normative input -> outcome table for `@vizejs/ui/chart-scale`: linear, pow,
sqrt, log, time, band, point, and ordinal scales, tick generation, nice
domains, and `Intl`-based tick formatting. Every row is proven by the named test
in `chart-scale.test.ts` or `chart-scale-ssr.test.ts`; compile-only guarantees
live in `chart-scale.types.test-d.ts`.

The numeric algorithms are ports of d3-array 3, d3-scale 4, and d3-time 3 (ISC
licensed). Parity is asserted against fixed vectors in `scale-d3-vectors.ts`,
generated once by running the same inputs through d3 under `TZ=UTC`,
`TZ=America/New_York`, and `TZ=Asia/Tokyo`. There is no runtime or test
dependency on d3. Scales are immutable callable objects: `nice()` and `with()`
return copies.

| #   | Input                                     | Outcome                                                                                 | Proven by                                                                   |
| --- | ----------------------------------------- | --------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| K1  | `ticks`, `tickIncrement`, `tickStep`      | identical to d3-array, including reversed, tiny, huge, and degenerate extents           | `ticks, tickIncrement, and tickStep match d3-array exactly`                 |
| K2  | linear map / invert / clamp / round       | identical to d3 including piecewise domains, descending domains, and zero-width domains | `linear scales map, invert, clamp, round, and tick like d3`                 |
| K3  | `nice`                                    | iterates to a stable tick increment like d3; copies, never mutates                      | `nice domains iterate to a stable tick increment like d3`                   |
| K4  | number `tickFormat`                       | `Intl.NumberFormat` with d3 `precisionFixed` digits; locale and percent options         | `number tick formats match d3 ',f' precision through Intl`                  |
| K5  | pow / sqrt                                | signed power transform and linear ticks identical to d3                                 | `pow and sqrt scales follow d3's signed power transform`                    |
| K6  | log                                       | map, invert, ticks, nice, negative domains, and d3's minor-label blanking               | `log scales map, tick, nice, and blank minor labels like d3`                |
| K7  | band / point                              | step, bandwidth, padding, align, round, and reversed ranges identical to d3             | `band and point scales lay out categories like d3`                          |
| K8  | ordinal                                   | range cycling like d3, but unknown values never mutate the domain                       | `ordinal scales cycle the range without mutating the domain`                |
| K9  | UTC time ticks / nice / label granularity | identical to d3 `scaleUtc` from milliseconds to centuries                               | `UTC time ticks, nice domains, and label granularity match d3 scaleUtc`     |
| K10 | zoned time ticks                          | `timeZone` reproduces d3 local `scaleTime` under that `TZ`, including DST transitions   | `zoned time ticks match d3 local scaleTime run under that TZ`               |
| K11 | time map / invert / `tickFormat`          | `Intl.DateTimeFormat` labels per granularity, in the scale's zone and locale            | `time scales map dates, invert, and format ticks with Intl in their zone`   |
| K12 | `with`, `options`, frozen domains         | immutable copies that round-trip their options                                          | `scales are immutable and expose their options`                             |
| K13 | SSR on hosts in different time zones      | byte-identical markup, because ticks and labels never read the host zone                | `scale-driven markup is byte-identical across requests and host time zones` |
| K14 | hydration                                 | scale-driven markup hydrates without warnings                                           | `hydrates scale-driven markup without mismatch warnings`                    |

`Intl` renders negative numbers with an ASCII hyphen-minus where d3 uses `−`;
that is the only intentional difference in number labels.

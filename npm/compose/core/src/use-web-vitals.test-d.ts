/** Compile-only assertions for the `use-web-vitals` type contracts. */

import { WEB_VITALS_THRESHOLDS, rateMetric, useWebVitals } from "./use-web-vitals.ts";
import type {
  WebVitalMetric,
  WebVitalName,
  WebVitalRating,
  WebVitalsDocumentHost,
} from "./use-web-vitals.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const vitals = useWebVitals();
type _Keys = Expect<Equal<keyof typeof vitals.metrics.value, WebVitalName>>;
type _Metric = Expect<Equal<NonNullable<typeof vitals.metrics.value.LCP>, WebVitalMetric>>;
type _Rating = Expect<Equal<ReturnType<typeof rateMetric>, WebVitalRating>>;
type _Threshold = Expect<Equal<(typeof WEB_VITALS_THRESHOLDS)["CLS"], readonly [0.1, 0.25]>>;

vitals.onMetric((metric) => {
  type _Name = Expect<Equal<typeof metric.name, WebVitalName>>;
});

document satisfies WebVitalsDocumentHost;

// @ts-expect-error only the five metrics exist.
rateMetric("FID", 100);

// @ts-expect-error metrics are read-only.
vitals.metrics.value = {};

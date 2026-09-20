/**
 * The one type-check surface both engine-class suites measure against.
 *
 * Its medians are spread so every assertion is distinguishable: vue-tsc (the
 * published incumbent) is the slowest row, the same-engine native checkers sit
 * between it and Vize, and `vize-check-max` is the fastest. A ratio that picked
 * the wrong baseline therefore cannot coincide with the right one.
 */

const CHECK_VARIANTS = [
  { id: "vue-tsc", label: "vue-tsc", medianMs: 8000, throughput: "62.5 files/s", runs: [8000] },
  {
    id: "verter-tsc",
    label: "verter-tsc",
    medianMs: 1000,
    throughput: "500.0 files/s",
    runs: [1000],
  },
  {
    id: "golar-typecheck",
    label: "Golar typecheck",
    medianMs: 1500,
    throughput: "333.3 files/s",
    runs: [1500],
  },
  {
    id: "golar-default",
    label: "Golar (lint+check)",
    medianMs: 2500,
    throughput: "200.0 files/s",
    runs: [2500],
  },
  {
    id: "vize-check-1t",
    label: "Vize check (1T)",
    medianMs: 2000,
    throughput: "250.0 files/s",
    runs: [2000],
  },
  {
    id: "vize-check-max",
    label: "Vize check (max)",
    medianMs: 500,
    throughput: "1.0k files/s",
    runs: [500],
  },
];

const CHECK_ENGINE_CLASSES = {
  "golar-default": "tsgo-native",
  "golar-typecheck": "tsgo-native",
  "verter-tsc": "tsgo-native",
  "vue-tsc": "typescript-js",
  "vize-check-1t": "tsgo-native",
  "vize-check-max": "tsgo-native",
};

function checkSurfaceInput(overrides: Record<string, unknown> = {}) {
  return {
    id: "check",
    label: "Type check",
    files: 500,
    bytes: 1_000_000,
    baselineId: "vue-tsc",
    vizeSingleId: "vize-check-1t",
    vizeMaxId: "vize-check-max",
    engineClasses: CHECK_ENGINE_CLASSES,
    variants: CHECK_VARIANTS,
    ...overrides,
  };
}

export { CHECK_ENGINE_CLASSES, CHECK_VARIANTS, checkSurfaceInput };

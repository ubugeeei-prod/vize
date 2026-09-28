/** The product compile's opt-in, same-run stage feed. */
export const PRODUCT_CAPTURE_SCHEMA_VERSION = 2;

export type ProductTarget = "dom" | "ssr" | "vapor";
export type ProductOutcome = "accepted" | "legacy" | "unavailable" | "rejected";

export interface ProductCaptureFeed {
  schema_version: 2;
  command: string;
  source: {
    path: string | null;
    input_syntax: string;
    compiled_syntax: string;
    template_span: { start: number; end: number } | null;
  };
  target: ProductTarget;
  outcome: { kind: ProductOutcome; reason: string | null };
  observed: { timings: boolean; remarks: boolean };
  options: { name: string; value: string }[];
  pages: { level: string; step: string; text: string }[];
  timings: { level: string; step: string; nanos: number }[];
  remarks: {
    level: string;
    pass: string;
    kind: "applied" | "missed" | "analysis";
    name: string;
    span: { start: number; end: number };
    args: { name: string; value: string | number | boolean }[];
  }[];
}

export type ProductCaptureNegotiation =
  | { ok: true; feed: ProductCaptureFeed }
  | { ok: false; error: string };

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function span(value: unknown): value is { start: number; end: number } {
  return (
    record(value) &&
    Number.isSafeInteger(value.start) &&
    Number.isSafeInteger(value.end) &&
    (value.start as number) >= 0 &&
    (value.end as number) >= (value.start as number)
  );
}

function text(value: unknown): value is string {
  return typeof value === "string";
}

function page(value: unknown): boolean {
  return record(value) && text(value.level) && text(value.step) && text(value.text);
}

function timing(value: unknown): boolean {
  return (
    record(value) &&
    text(value.level) &&
    text(value.step) &&
    Number.isSafeInteger(value.nanos) &&
    (value.nanos as number) >= 0
  );
}

function remark(value: unknown): boolean {
  return (
    record(value) &&
    text(value.level) &&
    text(value.pass) &&
    ["applied", "missed", "analysis"].includes(String(value.kind)) &&
    text(value.name) &&
    span(value.span) &&
    Array.isArray(value.args) &&
    value.args.every(
      (arg: unknown) =>
        record(arg) &&
        text(arg.name) &&
        ["string", "number", "boolean"].includes(typeof arg.value),
    )
  );
}

/** Reject unsupported versions, target mixups, and partial result shapes. */
export function negotiateProductCapture(
  raw: unknown,
  target: ProductTarget,
): ProductCaptureNegotiation {
  if (!record(raw)) return { ok: false, error: `${target} compile returned no product capture.` };
  if (raw.schema_version !== PRODUCT_CAPTURE_SCHEMA_VERSION) {
    return {
      ok: false,
      error: `${target} product capture schema_version ${String(raw.schema_version)} is unsupported; expected ${PRODUCT_CAPTURE_SCHEMA_VERSION}.`,
    };
  }
  if (raw.target !== target) {
    return { ok: false, error: `${target} compile returned a ${String(raw.target)} capture.` };
  }
  const source = raw.source;
  const outcome = raw.outcome;
  const observed = raw.observed;
  if (
    !text(raw.command) ||
    !record(source) ||
    !(source.path === null || text(source.path)) ||
    !text(source.input_syntax) ||
    !text(source.compiled_syntax) ||
    !(source.template_span === null || span(source.template_span)) ||
    !record(outcome) ||
    !["accepted", "legacy", "unavailable", "rejected"].includes(String(outcome.kind)) ||
    !(outcome.reason === null || text(outcome.reason)) ||
    !record(observed) ||
    typeof observed.timings !== "boolean" ||
    typeof observed.remarks !== "boolean" ||
    !Array.isArray(raw.options) ||
    !raw.options.every((item) => record(item) && text(item.name) && text(item.value)) ||
    !Array.isArray(raw.pages) ||
    !raw.pages.every(page) ||
    !Array.isArray(raw.timings) ||
    !raw.timings.every(timing) ||
    !Array.isArray(raw.remarks) ||
    !raw.remarks.every(remark)
  ) {
    return { ok: false, error: `${target} product capture does not match schema v2.` };
  }
  if (
    outcome.kind !== "accepted" &&
    (raw.pages.length > 0 || raw.timings.length > 0 || raw.remarks.length > 0)
  ) {
    return { ok: false, error: `${target} fallback capture unexpectedly contains native stages.` };
  }
  return { ok: true, feed: raw as unknown as ProductCaptureFeed };
}

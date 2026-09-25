import type {
  SignatureDataUrlOptions,
  SignaturePadErrorCode,
  SignaturePadValueFormat,
  SignaturePoint,
  SignatureStroke,
  SignatureStrokeOptions,
  SignatureSvgOptions,
  SignatureValue,
} from "./signature-pad-types.ts";

/** Typed error thrown by signature helpers. `code` is stable across releases. */
export class SignaturePadError extends Error {
  /** Stable diagnostic code. */
  readonly code: SignaturePadErrorCode;

  constructor(code: SignaturePadErrorCode, message: string) {
    super(`${code}: ${message}`);
    this.name = "SignaturePadError";
    this.code = code;
  }
}

/** Resolved stroke geometry defaults. */
export const SIGNATURE_STROKE_DEFAULTS = Object.freeze({
  size: 3,
  thinning: 0.6,
  smoothing: 0.5,
});

interface Vector {
  readonly x: number;
  readonly y: number;
}

const MIN_DISTANCE = 0.01;

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function finiteOr(value: number | undefined, fallback: number): number {
  return value !== undefined && Number.isFinite(value) ? value : fallback;
}

/** Format one coordinate compactly and deterministically (two decimals, no `-0`). */
function formatNumber(value: number): string {
  const rounded = Math.round(value * 100) / 100;
  return Object.is(rounded, -0) ? "0" : String(rounded);
}

function point(vector: Vector): string {
  return `${formatNumber(vector.x)} ${formatNumber(vector.y)}`;
}

function midpoint(left: Vector, right: Vector): Vector {
  return { x: (left.x + right.x) / 2, y: (left.y + right.y) / 2 };
}

function lerp(from: Vector, to: Vector, amount: number): Vector {
  return { x: from.x + (to.x - from.x) * amount, y: from.y + (to.y - from.y) * amount };
}

/** Whether a signature contains no drawable point. */
export function isSignatureEmpty(value: SignatureValue): boolean {
  return value.every((stroke) => stroke.points.length === 0);
}

/**
 * Pressure for the next sample of a non-pen pointer, simulated from velocity:
 * fast movement thins the stroke and slow movement thickens it, eased against
 * the previous sample so width changes stay smooth.
 */
export function simulatePressure(
  previous: SignaturePoint | undefined,
  x: number,
  y: number,
  time: number,
): number {
  if (previous === undefined) return 0.5;
  const distance = Math.hypot(x - previous.x, y - previous.y);
  const elapsed = Math.max(1, time - previous.time);
  const velocity = distance / elapsed;
  const target = 1 - Math.min(1, velocity / 2) * 0.8;
  return clamp(previous.pressure * 0.7 + target * 0.3, 0, 1);
}

function dedupe(points: readonly SignaturePoint[]): SignaturePoint[] {
  const result: SignaturePoint[] = [];
  for (const sample of points) {
    if (!Number.isFinite(sample.x) || !Number.isFinite(sample.y)) continue;
    const last = result.at(-1);
    if (last && Math.hypot(sample.x - last.x, sample.y - last.y) < MIN_DISTANCE) continue;
    result.push(sample);
  }
  return result;
}

function smoothEdge(edge: readonly Vector[], smoothing: number): string {
  const first = edge[0];
  if (first === undefined) return "";
  let path = "";
  for (let index = 1; index < edge.length - 1; index += 1) {
    const control = edge[index];
    const next = edge[index + 1];
    if (control === undefined || next === undefined) continue;
    const end = lerp(control, midpoint(control, next), smoothing);
    path += ` Q ${point(control)} ${point(end)}`;
  }
  const last = edge.at(-1);
  if (last !== undefined && edge.length > 1) path += ` L ${point(last)}`;
  return path;
}

/**
 * Filled outline of one stroke as an SVG path `d` string.
 *
 * Each sample contributes a half-width of `size * (1 - thinning + thinning *
 * pressure) / 2` perpendicular to the local direction. Both edges are smoothed
 * with quadratic curves and joined with round caps; a single sample renders a
 * dot. Empty strokes produce `""`.
 */
export function strokeOutlinePath(
  stroke: SignatureStroke,
  options: SignatureStrokeOptions = {},
): string {
  const size = Math.max(0, finiteOr(options.size, SIGNATURE_STROKE_DEFAULTS.size));
  const thinning = clamp(finiteOr(options.thinning, SIGNATURE_STROKE_DEFAULTS.thinning), 0, 1);
  const smoothing = clamp(finiteOr(options.smoothing, SIGNATURE_STROKE_DEFAULTS.smoothing), 0, 1);
  const points = dedupe(stroke.points);
  const radius = (sample: SignaturePoint): number =>
    Math.max(0.01, (size * (1 - thinning + thinning * clamp(sample.pressure, 0, 1))) / 2);

  const first = points[0];
  if (first === undefined || size === 0) return "";
  if (points.length === 1) {
    const r = radius(first);
    return `M ${formatNumber(first.x - r)} ${formatNumber(first.y)} a ${formatNumber(r)} ${formatNumber(r)} 0 1 0 ${formatNumber(r * 2)} 0 a ${formatNumber(r)} ${formatNumber(r)} 0 1 0 ${formatNumber(-r * 2)} 0 Z`;
  }

  const left: Vector[] = [];
  const right: Vector[] = [];
  points.forEach((sample, index) => {
    const before = points[Math.max(0, index - 1)] ?? sample;
    const after = points[Math.min(points.length - 1, index + 1)] ?? sample;
    const dx = after.x - before.x;
    const dy = after.y - before.y;
    const length = Math.hypot(dx, dy) || 1;
    const r = radius(sample);
    const nx = (-dy / length) * r;
    const ny = (dx / length) * r;
    left.push({ x: sample.x + nx, y: sample.y + ny });
    right.push({ x: sample.x - nx, y: sample.y - ny });
  });

  const last = points.at(-1) ?? first;
  const endRadius = formatNumber(radius(last));
  const startRadius = formatNumber(radius(first));
  const rightReversed = [...right].reverse();
  return (
    `M ${point(left[0] ?? first)}` +
    smoothEdge(left, smoothing) +
    ` A ${endRadius} ${endRadius} 0 0 0 ${point(rightReversed[0] ?? last)}` +
    smoothEdge(rightReversed, smoothing) +
    ` A ${startRadius} ${startRadius} 0 0 0 ${point(left[0] ?? first)} Z`
  );
}

function escapeAttribute(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

/** Standalone SVG markup for a signature, with one filled path per stroke. */
export function signatureToSvg(value: SignatureValue, options: SignatureSvgOptions): string {
  const width = formatNumber(Math.max(0, options.width));
  const height = formatNumber(Math.max(0, options.height));
  const color = escapeAttribute(options.color ?? "#000");
  const background =
    options.background === undefined
      ? ""
      : `<rect width="100%" height="100%" fill="${escapeAttribute(options.background)}"/>`;
  const paths = value
    .map((stroke) => strokeOutlinePath(stroke, options))
    .filter((d) => d.length > 0)
    .map((d) => `<path d="${d}" fill="${color}"/>`)
    .join("");
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" width="${width}" height="${height}">${background}${paths}</svg>`;
}

/**
 * Export a signature as a data URL.
 *
 * `image/svg+xml` is produced from {@link signatureToSvg} without a canvas and
 * works on the server. Raster types draw with `Path2D` on a canvas and require
 * a browser; otherwise the promise rejects with
 * `VIZE_UI_SIGNATURE_PAD_CANVAS_UNAVAILABLE`.
 */
export function signatureToDataUrl(
  value: SignatureValue,
  options: SignatureDataUrlOptions,
): Promise<string> {
  const type = options.type ?? "image/png";
  if (type === "image/svg+xml") {
    return Promise.resolve(
      `data:image/svg+xml;charset=utf-8,${encodeURIComponent(signatureToSvg(value, options))}`,
    );
  }
  if (typeof document === "undefined" || typeof Path2D !== "function") {
    return Promise.reject(
      new SignaturePadError(
        "VIZE_UI_SIGNATURE_PAD_CANVAS_UNAVAILABLE",
        "raster export requires a browser canvas with Path2D",
      ),
    );
  }
  const scale = Math.max(0.01, finiteOr(options.scale, 1));
  const canvas = document.createElement("canvas");
  canvas.width = Math.max(1, Math.round(options.width * scale));
  canvas.height = Math.max(1, Math.round(options.height * scale));
  const context = canvas.getContext("2d");
  if (context === null) {
    return Promise.reject(
      new SignaturePadError(
        "VIZE_UI_SIGNATURE_PAD_CANVAS_UNAVAILABLE",
        "the canvas 2D context is unavailable",
      ),
    );
  }
  context.scale(scale, scale);
  if (options.background !== undefined) {
    context.fillStyle = options.background;
    context.fillRect(0, 0, options.width, options.height);
  }
  context.fillStyle = options.color ?? "#000";
  for (const stroke of value) {
    const d = strokeOutlinePath(stroke, options);
    if (d.length > 0) context.fill(new Path2D(d));
  }
  return Promise.resolve(canvas.toDataURL(type, options.quality));
}

function roundPoint(sample: SignaturePoint): SignaturePoint {
  return {
    x: Math.round(sample.x * 100) / 100,
    y: Math.round(sample.y * 100) / 100,
    pressure: Math.round(sample.pressure * 1000) / 1000,
    time: Math.round(sample.time),
  };
}

/**
 * Serialize a signature for form submission: compact JSON of rounded samples,
 * or standalone SVG markup. Empty signatures serialize to `""`.
 */
export function serializeSignature(
  value: SignatureValue,
  format: SignaturePadValueFormat,
  options: SignatureSvgOptions,
): string {
  if (isSignatureEmpty(value)) return "";
  if (format === "svg") return signatureToSvg(value, options);
  return JSON.stringify(value.map((stroke) => ({ points: stroke.points.map(roundPoint) })));
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function parsePoint(value: unknown): SignaturePoint | null {
  if (!isRecord(value)) return null;
  const { x, y, pressure, time } = value;
  if (
    typeof x !== "number" ||
    typeof y !== "number" ||
    typeof pressure !== "number" ||
    typeof time !== "number" ||
    ![x, y, pressure, time].every(Number.isFinite)
  ) {
    return null;
  }
  return Object.freeze({ x, y, pressure: clamp(pressure, 0, 1), time });
}

/**
 * Parse JSON produced by {@link serializeSignature}. `""` parses to an empty
 * signature.
 *
 * @throws {SignaturePadError} `VIZE_UI_SIGNATURE_PAD_INVALID_VALUE` for malformed input.
 */
export function parseSignature(text: string): SignatureValue {
  if (text.trim().length === 0) return Object.freeze([]);
  const invalid = (): SignaturePadError =>
    new SignaturePadError("VIZE_UI_SIGNATURE_PAD_INVALID_VALUE", "expected serialized strokes");
  let data: unknown;
  try {
    data = JSON.parse(text);
  } catch {
    throw invalid();
  }
  if (!Array.isArray(data)) throw invalid();
  const strokes: SignatureStroke[] = [];
  for (const entry of data) {
    if (!isRecord(entry) || !Array.isArray(entry.points)) throw invalid();
    const points: SignaturePoint[] = [];
    for (const candidate of entry.points) {
      const parsed = parsePoint(candidate);
      if (parsed === null) throw invalid();
      points.push(parsed);
    }
    strokes.push(Object.freeze({ points: Object.freeze(points) }));
  }
  return Object.freeze(strokes);
}

import type { AudioBarsOptions } from "./audio-visualizer-types.ts";

const DEFAULT_DECIBELS: readonly [number, number] = [-100, -30];

function clamp01(value: number): number {
  if (!Number.isFinite(value)) return 0;
  return value < 0 ? 0 : value > 1 ? 1 : value;
}

function isFloat(data: Float32Array | Uint8Array): data is Float32Array {
  return data instanceof Float32Array;
}

/** Normalize one time-domain sample to `-1..1`. */
function sample(data: Float32Array | Uint8Array, index: number): number {
  const value = data[index] ?? (isFloat(data) ? 0 : 128);
  return isFloat(data) ? value : (value - 128) / 128;
}

/** Root-mean-square level of time-domain data, `0..1`. Silence and empty data are `0`. */
export function computeLevel(waveform: Float32Array | Uint8Array): number {
  if (waveform.length === 0) return 0;
  let sum = 0;
  for (let index = 0; index < waveform.length; index++) {
    const value = sample(waveform, index);
    sum += value * value;
  }
  return clamp01(Math.sqrt(sum / waveform.length));
}

/** Absolute peak of time-domain data, `0..1`. */
export function computePeak(waveform: Float32Array | Uint8Array): number {
  let peak = 0;
  for (let index = 0; index < waveform.length; index++) {
    peak = Math.max(peak, Math.abs(sample(waveform, index)));
  }
  return clamp01(peak);
}

/** Normalize one frequency bin to `0..1` (byte `/255`, float decibels over the range). */
function level(
  data: Float32Array | Uint8Array,
  index: number,
  decibels: readonly [number, number],
) {
  const value = data[index] ?? 0;
  if (!isFloat(data)) return value / 255;
  const [min, max] = decibels;
  return clamp01((value - min) / (max - min));
}

/**
 * Bin boundaries for `count` bars over `binCount` bins. `log` spacing starts at
 * bin 1 (bin 0 is DC) and grows geometrically; every bar owns at least one bin
 * when there are enough bins. The result has `count + 1` ascending edges.
 */
export function barEdges(binCount: number, count: number, scale: "linear" | "log"): number[] {
  const bars = Math.max(0, Math.floor(count));
  const bins = Math.max(0, Math.floor(binCount));
  if (bars === 0 || bins === 0) return [];
  const edges = [scale === "log" && bins > 1 ? 1 : 0];
  for (let bar = 1; bar <= bars; bar++) {
    const fraction = bar / bars;
    const start = edges[0] ?? 0;
    const edge =
      scale === "log" ? start * Math.pow(bins / Math.max(start, 1), fraction) : bins * fraction;
    const previous = edges[bar - 1] ?? 0;
    edges.push(Math.min(bins, Math.max(Math.round(edge), previous + (previous < bins ? 1 : 0))));
  }
  edges[bars] = bins;
  return edges;
}

/**
 * Group frequency data into `count` bars normalized to `0..1`, averaging the
 * bins of each bar. Bars without bins (more bars than bins) repeat the previous
 * bar's value so the output always has `count` entries.
 */
export function toBars(
  data: Float32Array | Uint8Array,
  count: number,
  options: AudioBarsOptions = {},
): readonly number[] {
  const scale = options.scale ?? "log";
  const decibels = options.decibels ?? DEFAULT_DECIBELS;
  const edges = barEdges(data.length, count, scale);
  const bars: number[] = [];
  for (let bar = 0; bar + 1 < edges.length; bar++) {
    const start = edges[bar] ?? 0;
    const end = edges[bar + 1] ?? start;
    if (end <= start) {
      bars.push(bars.at(-1) ?? 0);
      continue;
    }
    let sum = 0;
    for (let index = start; index < end; index++) sum += level(data, index, decibels);
    bars.push(clamp01(sum / (end - start)));
  }
  return Object.freeze(bars);
}

function coordinate(value: number): string {
  const rounded = Math.round(value * 100) / 100;
  return Object.is(rounded, -0) ? "0" : String(rounded);
}

/**
 * SVG path (`M x,y L x,y …`) of time-domain data scaled into a `width` x
 * `height` box; silence is the horizontal center line. Coordinates are rounded
 * to two decimals so output is deterministic. Empty data yields `""`.
 */
export function toWaveformPath(
  data: Float32Array | Uint8Array,
  width: number,
  height: number,
): string {
  if (data.length === 0 || !(width > 0) || !(height > 0)) return "";
  const step = data.length > 1 ? width / (data.length - 1) : 0;
  const middle = height / 2;
  const commands: string[] = [];
  for (let index = 0; index < data.length; index++) {
    const x = step * index;
    const y = middle - Math.max(-1, Math.min(1, sample(data, index))) * middle;
    commands.push(`${index === 0 ? "M" : "L"}${coordinate(x)},${coordinate(y)}`);
  }
  return commands.join(" ");
}

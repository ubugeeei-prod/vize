/**
 * Pure, dependency-free color model for the ColorPicker family.
 *
 * Colors are stored as immutable HSB (HSV) values plus alpha because that
 * space keeps hue and saturation stable while a user drags a saturation ×
 * brightness area through achromatic regions. Every other representation is
 * derived on demand, and parsing/formatting follows CSS Color 4 syntax.
 */

/** Immutable color value used by every ColorPicker part. */
export interface ColorValue {
  /** Hue angle in degrees, `0` inclusive to `360` exclusive. */
  readonly hue: number;

  /** HSB saturation percentage from `0` to `100`. */
  readonly saturation: number;

  /** HSB brightness (value) percentage from `0` to `100`. */
  readonly brightness: number;

  /** Opacity from `0` (transparent) to `1` (opaque). */
  readonly alpha: number;
}

/** Red, green, blue channels from `0` to `255` plus alpha from `0` to `1`. */
export interface RgbaColor {
  /** Red channel from `0` to `255`. */
  readonly red: number;

  /** Green channel from `0` to `255`. */
  readonly green: number;

  /** Blue channel from `0` to `255`. */
  readonly blue: number;

  /** Opacity from `0` to `1`. */
  readonly alpha: number;
}

/** HSL representation: hue degrees, saturation and lightness percentages, alpha. */
export interface HslaColor {
  /** Hue angle in degrees, `0` inclusive to `360` exclusive. */
  readonly hue: number;

  /** HSL saturation percentage from `0` to `100`. */
  readonly saturation: number;

  /** HSL lightness percentage from `0` to `100`. */
  readonly lightness: number;

  /** Opacity from `0` to `1`. */
  readonly alpha: number;
}

/**
 * String serialization produced by {@link formatColor}.
 *
 * - `hex`: `#rrggbb`, or `#rrggbbaa` when the color is not fully opaque.
 * - `hex8`: always `#rrggbbaa`.
 * - `rgb`: CSS Color 4 `rgb(r g b)` / `rgb(r g b / a)`.
 * - `hsl`: CSS Color 4 `hsl(h s% l%)` / `hsl(h s% l% / a)`.
 * - `hsb`: non-CSS `hsb(h s% b%)` / `hsb(h s% b% / a)` for design-tool displays.
 */
export type ColorFormat = "hex" | "hex8" | "hsb" | "hsl" | "rgb";

/** Color space whose channels a slider or area edits. */
export type ColorSpace = "hsb" | "hsl" | "rgb";

/** Channel names owned by each {@link ColorSpace}. */
export interface ColorSpaceChannelMap {
  /** Hue, HSB saturation, brightness, and alpha. */
  readonly hsb: "alpha" | "brightness" | "hue" | "saturation";

  /** Hue, HSL saturation, lightness, and alpha. */
  readonly hsl: "alpha" | "hue" | "lightness" | "saturation";

  /** Red, green, blue, and alpha. */
  readonly rgb: "alpha" | "blue" | "green" | "red";
}

/** Channels available in one color space, inferred from the space literal. */
export type ColorSpaceChannel<Space extends ColorSpace = ColorSpace> = ColorSpaceChannelMap[Space];

/** Every editable color channel. */
export type ColorChannel = ColorSpaceChannel;

/** Numeric range and keyboard increments for one channel. */
export interface ColorChannelRange {
  /** Smallest channel value. */
  readonly min: number;

  /** Largest channel value. */
  readonly max: number;

  /** Arrow-key increment. */
  readonly step: number;

  /** Page Up / Page Down and Shift+Arrow increment. */
  readonly pageStep: number;
}

/** Fully transparent-free black used when no parseable color is supplied. */
export const DEFAULT_COLOR: ColorValue = Object.freeze({
  hue: 0,
  saturation: 0,
  brightness: 0,
  alpha: 1,
});

const COLOR_FORMATS: ReadonlySet<string> = new Set<ColorFormat>([
  "hex",
  "hex8",
  "hsb",
  "hsl",
  "rgb",
]);

const CHANNEL_RANGES: Readonly<Record<ColorChannel, ColorChannelRange>> = Object.freeze({
  hue: Object.freeze({ min: 0, max: 360, step: 1, pageStep: 15 }),
  saturation: Object.freeze({ min: 0, max: 100, step: 1, pageStep: 10 }),
  brightness: Object.freeze({ min: 0, max: 100, step: 1, pageStep: 10 }),
  lightness: Object.freeze({ min: 0, max: 100, step: 1, pageStep: 10 }),
  red: Object.freeze({ min: 0, max: 255, step: 1, pageStep: 16 }),
  green: Object.freeze({ min: 0, max: 255, step: 1, pageStep: 16 }),
  blue: Object.freeze({ min: 0, max: 255, step: 1, pageStep: 16 }),
  alpha: Object.freeze({ min: 0, max: 1, step: 0.01, pageStep: 0.1 }),
});

const CHANNEL_LABELS: Readonly<Record<ColorChannel, string>> = Object.freeze({
  hue: "Hue",
  saturation: "Saturation",
  brightness: "Brightness",
  lightness: "Lightness",
  red: "Red",
  green: "Green",
  blue: "Blue",
  alpha: "Alpha",
});

const NUMBER = /^[+-]?(?:\d+\.?\d*|\.\d+)(?:e[+-]?\d+)?$/i;
const HEX = /^#([\da-f]{3,4}|[\da-f]{6}|[\da-f]{8})$/i;
const FUNCTION = /^([a-z]+)\(\s*([^()]*?)\s*\)$/i;
const ANGLE = /^([+-]?(?:\d+\.?\d*|\.\d+)(?:e[+-]?\d+)?)(deg|grad|rad|turn)?$/i;

function clamp(value: number, min: number, max: number): number {
  if (Number.isNaN(value)) return min;
  return Math.min(max, Math.max(min, value));
}

function wrapHue(value: number): number {
  if (!Number.isFinite(value)) return 0;
  const wrapped = value % 360;
  return wrapped < 0 ? wrapped + 360 : wrapped === 0 ? 0 : wrapped;
}

function round(value: number, digits = 0): number {
  const factor = 10 ** digits;
  const rounded = Math.round(value * factor) / factor;
  return Object.is(rounded, -0) ? 0 : rounded;
}

/** Whether an unknown value names a supported {@link ColorFormat}. */
export function isColorFormat(value: unknown): value is ColorFormat {
  return typeof value === "string" && COLOR_FORMATS.has(value);
}

/** Whether an unknown value is a structurally valid {@link ColorValue}. */
export function isColorValue(value: unknown): value is ColorValue {
  if (typeof value !== "object" || value === null) return false;
  const hue: unknown = Reflect.get(value, "hue");
  const saturation: unknown = Reflect.get(value, "saturation");
  const brightness: unknown = Reflect.get(value, "brightness");
  const alpha: unknown = Reflect.get(value, "alpha");
  return (
    typeof hue === "number" &&
    typeof saturation === "number" &&
    typeof brightness === "number" &&
    typeof alpha === "number" &&
    [hue, saturation, brightness, alpha].every(Number.isFinite)
  );
}

/**
 * Create a frozen, normalized color. Hue wraps into `[0, 360)`, percentages
 * clamp to `[0, 100]`, and alpha clamps to `[0, 1]`. Non-finite inputs fall
 * back to the channel minimum (alpha falls back to `1`).
 */
export function createColor(input: {
  readonly hue: number;
  readonly saturation: number;
  readonly brightness: number;
  readonly alpha?: number;
}): ColorValue {
  const alpha = input.alpha ?? 1;
  return Object.freeze({
    hue: wrapHue(input.hue),
    saturation: clamp(Number.isFinite(input.saturation) ? input.saturation : 0, 0, 100),
    brightness: clamp(Number.isFinite(input.brightness) ? input.brightness : 0, 0, 100),
    alpha: clamp(Number.isFinite(alpha) ? alpha : 1, 0, 1),
  });
}

/** Convert a color to unrounded sRGB channels. */
export function toRgba(color: ColorValue): RgbaColor {
  const saturation = color.saturation / 100;
  const brightness = color.brightness / 100;
  const channel = (offset: number): number => {
    const k = (offset + color.hue / 60) % 6;
    return brightness - brightness * saturation * Math.max(0, Math.min(k, 4 - k, 1));
  };
  return Object.freeze({
    red: channel(5) * 255,
    green: channel(3) * 255,
    blue: channel(1) * 255,
    alpha: color.alpha,
  });
}

/**
 * Create a color from sRGB channels. Achromatic results keep `fallback`'s hue
 * (and black keeps its saturation) so editing never snaps hue back to red.
 */
export function fromRgba(input: RgbaColor, fallback: ColorValue = DEFAULT_COLOR): ColorValue {
  const red = clamp(input.red, 0, 255) / 255;
  const green = clamp(input.green, 0, 255) / 255;
  const blue = clamp(input.blue, 0, 255) / 255;
  const max = Math.max(red, green, blue);
  const delta = max - Math.min(red, green, blue);
  let hue = fallback.hue;
  if (delta > 0) {
    if (max === red) hue = 60 * (((green - blue) / delta) % 6);
    else if (max === green) hue = 60 * ((blue - red) / delta + 2);
    else hue = 60 * ((red - green) / delta + 4);
  }
  const saturation = max === 0 ? fallback.saturation : (delta / max) * 100;
  return createColor({ hue, saturation, brightness: max * 100, alpha: input.alpha });
}

/** Convert a color to unrounded HSL channels. */
export function toHsla(color: ColorValue): HslaColor {
  const saturation = color.saturation / 100;
  const brightness = color.brightness / 100;
  const lightness = brightness * (1 - saturation / 2);
  const divisor = Math.min(lightness, 1 - lightness);
  return Object.freeze({
    hue: color.hue,
    saturation: divisor === 0 ? 0 : ((brightness - lightness) / divisor) * 100,
    lightness: lightness * 100,
    alpha: color.alpha,
  });
}

/**
 * Create a color from HSL channels. Black keeps `fallback`'s HSB saturation so
 * a lightness drag through `0%` does not discard the chosen saturation.
 */
export function fromHsla(input: HslaColor, fallback: ColorValue = DEFAULT_COLOR): ColorValue {
  const saturation = clamp(input.saturation, 0, 100) / 100;
  const lightness = clamp(input.lightness, 0, 100) / 100;
  const brightness = lightness + saturation * Math.min(lightness, 1 - lightness);
  return createColor({
    hue: input.hue,
    saturation: brightness === 0 ? fallback.saturation : 2 * (1 - lightness / brightness) * 100,
    brightness: brightness * 100,
    alpha: input.alpha,
  });
}

function parseNumber(token: string): number | null {
  return NUMBER.test(token) ? Number(token) : null;
}

function parsePercentOrNumber(token: string, percentScale: number): number | null {
  if (token.endsWith("%")) {
    const value = parseNumber(token.slice(0, -1));
    return value === null ? null : (value / 100) * percentScale;
  }
  return parseNumber(token);
}

function parseAlpha(token: string | undefined): number | null {
  if (token === undefined) return 1;
  const value = parsePercentOrNumber(token, 1);
  return value === null ? null : clamp(value, 0, 1);
}

function parseHue(token: string): number | null {
  const match = ANGLE.exec(token);
  if (match === null) return null;
  const value = Number(match[1]);
  switch ((match[2] ?? "deg").toLowerCase()) {
    case "grad":
      return (value * 360) / 400;
    case "rad":
      return (value * 180) / Math.PI;
    case "turn":
      return value * 360;
    default:
      return value;
  }
}

/** Split function arguments into three channel tokens and an optional alpha token. */
function splitArguments(
  body: string,
): readonly [string, string, string, string | undefined] | null {
  if (body.includes(",")) {
    const parts = body.split(",").map((part) => part.trim());
    if (parts.some((part) => part.length === 0 || /\s/.test(part))) return null;
    if (parts.length === 3) return [parts[0] ?? "", parts[1] ?? "", parts[2] ?? "", undefined];
    if (parts.length === 4) return [parts[0] ?? "", parts[1] ?? "", parts[2] ?? "", parts[3]];
    return null;
  }
  const slash = body.split("/");
  if (slash.length > 2) return null;
  const channels = (slash[0] ?? "").trim().split(/\s+/);
  if (channels.length !== 3) return null;
  const alpha = slash[1]?.trim();
  if (alpha !== undefined && (alpha.length === 0 || /\s/.test(alpha))) return null;
  return [channels[0] ?? "", channels[1] ?? "", channels[2] ?? "", alpha];
}

function parseHex(digits: string): ColorValue {
  const expanded =
    digits.length <= 4
      ? digits.replaceAll(/[\da-f]/gi, "$&$&").toLowerCase()
      : digits.toLowerCase();
  const read = (index: number): number => Number.parseInt(expanded.slice(index, index + 2), 16);
  return fromRgba({
    red: read(0),
    green: read(2),
    blue: read(4),
    alpha: expanded.length === 8 ? read(6) / 255 : 1,
  });
}

/**
 * Parse a CSS color string into a {@link ColorValue}.
 *
 * Supports `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, `rgb()`/`rgba()` and
 * `hsl()`/`hsla()` in both legacy comma and modern space/slash syntax
 * (numbers or percentages, hue units `deg`/`grad`/`rad`/`turn`), the non-CSS
 * `hsb()`/`hsba()` notation, and the `transparent` keyword. Out-of-range
 * channels clamp exactly as CSS does. Returns `null` for anything else.
 */
export function parseColor(input: string): ColorValue | null {
  const source = input.trim();
  if (source.length === 0) return null;
  if (source.toLowerCase() === "transparent") return createColor({ ...DEFAULT_COLOR, alpha: 0 });
  const hex = HEX.exec(source);
  if (hex !== null) return parseHex(hex[1] ?? "");
  const call = FUNCTION.exec(source);
  if (call === null) return null;
  const name = (call[1] ?? "").toLowerCase();
  const args = splitArguments(call[2] ?? "");
  if (args === null) return null;
  const alpha = parseAlpha(args[3]);
  if (alpha === null) return null;

  if (name === "rgb" || name === "rgba") {
    const red = parsePercentOrNumber(args[0], 255);
    const green = parsePercentOrNumber(args[1], 255);
    const blue = parsePercentOrNumber(args[2], 255);
    if (red === null || green === null || blue === null) return null;
    return fromRgba({ red, green, blue, alpha });
  }
  if (name === "hsl" || name === "hsla" || name === "hsb" || name === "hsba") {
    const hue = parseHue(args[0]);
    const second = parsePercentOrNumber(args[1], 100);
    const third = parsePercentOrNumber(args[2], 100);
    if (hue === null || second === null || third === null) return null;
    if (name.startsWith("hsb")) {
      return createColor({ hue, saturation: second, brightness: third, alpha });
    }
    return fromHsla({
      hue: wrapHue(hue),
      saturation: clamp(second, 0, 100),
      lightness: clamp(third, 0, 100),
      alpha,
    });
  }
  return null;
}

function formatAlpha(alpha: number): string {
  return String(round(alpha, 3));
}

function hexPair(value: number): string {
  return Math.round(clamp(value, 0, 255))
    .toString(16)
    .padStart(2, "0");
}

/** Serialize a color into one {@link ColorFormat}. */
export function formatColor(color: ColorValue, format: ColorFormat = "hex"): string {
  switch (format) {
    case "hex":
    case "hex8": {
      const rgb = toRgba(color);
      const base = `#${hexPair(rgb.red)}${hexPair(rgb.green)}${hexPair(rgb.blue)}`;
      const alpha = Math.round(color.alpha * 255);
      return format === "hex8" || alpha < 255 ? `${base}${hexPair(alpha)}` : base;
    }
    case "rgb": {
      const rgb = toRgba(color);
      const body = `${Math.round(rgb.red)} ${Math.round(rgb.green)} ${Math.round(rgb.blue)}`;
      return color.alpha < 1 ? `rgb(${body} / ${formatAlpha(color.alpha)})` : `rgb(${body})`;
    }
    case "hsl": {
      const hsl = toHsla(color);
      const body = `${round(hsl.hue) % 360} ${round(hsl.saturation)}% ${round(hsl.lightness)}%`;
      return color.alpha < 1 ? `hsl(${body} / ${formatAlpha(color.alpha)})` : `hsl(${body})`;
    }
    case "hsb": {
      const body = `${round(color.hue) % 360} ${round(color.saturation)}% ${round(color.brightness)}%`;
      return color.alpha < 1 ? `hsb(${body} / ${formatAlpha(color.alpha)})` : `hsb(${body})`;
    }
  }
}

/** Serialize a color as a CSS `rgb()` string suitable for custom properties. */
export function toCssColor(color: ColorValue): string {
  return formatColor(color, "rgb");
}

/**
 * Whether two colors render identically at 8-bit sRGB precision, including
 * alpha quantized to 1/255 (the precision of `#rrggbbaa`).
 */
export function colorEquals(left: ColorValue, right: ColorValue): boolean {
  return formatColor(left, "hex8") === formatColor(right, "hex8");
}

/** Numeric range and keyboard increments for a channel. */
export function getColorChannelRange(channel: ColorChannel): ColorChannelRange {
  return CHANNEL_RANGES[channel];
}

/** English default label for a channel, used when no accessible name is supplied. */
export function getColorChannelLabel(channel: ColorChannel): string {
  return CHANNEL_LABELS[channel];
}

/**
 * Resolve the space a channel is read in. `lightness` always uses HSL, the RGB
 * channels always use RGB, and `hue`/`saturation`/`alpha` use `space` unless it
 * cannot own them (`saturation` in RGB falls back to HSB).
 */
export function resolveColorChannelSpace(
  channel: ColorChannel,
  space: ColorSpace = "hsb",
): ColorSpace {
  if (channel === "lightness") return "hsl";
  if (channel === "brightness") return "hsb";
  if (channel === "red" || channel === "green" || channel === "blue") return "rgb";
  if (channel === "saturation" && space === "rgb") return "hsb";
  return space;
}

/** Read one unrounded channel value in the given space. */
export function getColorChannelValue<Space extends ColorSpace = "hsb">(
  color: ColorValue,
  channel: ColorSpaceChannel<Space>,
  space?: Space,
): number {
  const key: ColorChannel = channel;
  const resolved = resolveColorChannelSpace(key, space);
  switch (key) {
    case "alpha":
      return color.alpha;
    case "hue":
      return color.hue;
    case "brightness":
      return color.brightness;
    case "saturation":
      return resolved === "hsl" ? toHsla(color).saturation : color.saturation;
    case "lightness":
      return toHsla(color).lightness;
    case "red":
      return toRgba(color).red;
    case "green":
      return toRgba(color).green;
    case "blue":
      return toRgba(color).blue;
  }
}

/** Return a new color with one channel replaced (clamped to its range). */
export function setColorChannelValue<Space extends ColorSpace = "hsb">(
  color: ColorValue,
  channel: ColorSpaceChannel<Space>,
  value: number,
  space?: Space,
): ColorValue {
  const key: ColorChannel = channel;
  const range = CHANNEL_RANGES[key];
  const next = clamp(Number.isFinite(value) ? value : range.min, range.min, range.max);
  const resolved = resolveColorChannelSpace(key, space);
  switch (key) {
    case "alpha":
      return createColor({ ...color, alpha: next });
    case "hue":
      return createColor({ ...color, hue: next === 360 ? 359.999 : next });
    case "brightness":
      return createColor({ ...color, brightness: next });
    case "saturation":
      return resolved === "hsl"
        ? fromHsla({ ...toHsla(color), saturation: next }, color)
        : createColor({ ...color, saturation: next });
    case "lightness":
      return fromHsla({ ...toHsla(color), lightness: next }, color);
    case "red":
      return fromRgba({ ...toRgba(color), red: next }, color);
    case "green":
      return fromRgba({ ...toRgba(color), green: next }, color);
    case "blue":
      return fromRgba({ ...toRgba(color), blue: next }, color);
  }
}

/** Snap a channel value to its step grid and range. */
export function snapColorChannelValue(channel: ColorChannel, value: number, step?: number): number {
  const range = CHANNEL_RANGES[channel];
  const size = step !== undefined && Number.isFinite(step) && step > 0 ? step : range.step;
  const snapped = Math.round((value - range.min) / size) * size + range.min;
  const decimals = (String(size).split(".")[1] ?? "").length;
  return round(clamp(snapped, range.min, range.max), decimals);
}

/**
 * Human-readable channel value for `aria-valuetext`, e.g. `"Hue 210°"`,
 * `"Saturation 40%"`, `"Red 128"`, or `"Alpha 50%"`.
 */
export function formatColorChannelValue(
  channel: ColorChannel,
  value: number,
  label: string = CHANNEL_LABELS[channel],
): string {
  switch (channel) {
    case "hue":
      return `${label} ${round(value)}°`;
    case "alpha":
      return `${label} ${round(value * 100)}%`;
    case "red":
    case "green":
    case "blue":
      return `${label} ${round(value)}`;
    default:
      return `${label} ${round(value)}%`;
  }
}

/**
 * Linear gradient that previews one channel's full range at the current
 * color, e.g. for a slider track background. Hue uses full saturation and
 * brightness; every other channel interpolates exactly along its range.
 */
export function getColorChannelGradient<Space extends ColorSpace = "hsb">(
  color: ColorValue,
  channel: ColorSpaceChannel<Space>,
  space?: Space,
  direction = "to right",
): string {
  const range = CHANNEL_RANGES[channel];
  let stops: readonly string[];
  if (channel === "hue") {
    stops = [0, 60, 120, 180, 240, 300, 359.999].map((hue) =>
      toCssColor(createColor({ hue, saturation: 100, brightness: 100 })),
    );
  } else if (channel === "lightness" || (channel === "saturation" && space === "hsl")) {
    const at = (fraction: number) =>
      toCssColor(
        setColorChannelValue(
          color,
          channel,
          range.min + (range.max - range.min) * fraction,
          resolveColorChannelSpace(channel, space),
        ),
      );
    stops = [at(0), at(0.5), at(1)];
  } else {
    stops = [range.min, range.max].map((value) =>
      toCssColor(
        setColorChannelValue(color, channel, value, resolveColorChannelSpace(channel, space)),
      ),
    );
  }
  return `linear-gradient(${direction}, ${stops.join(", ")})`;
}

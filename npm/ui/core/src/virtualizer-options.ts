/** Validation and reactive readers for the public virtualizer options. */

import { toValue } from "vue";

import type {
  VirtualizerOptions,
  VirtualizerOrientation,
  VirtualizerRect,
} from "./virtualizer-types.ts";

export const invalidOptionDiagnostic = "VIZE_UI_VIRTUALIZER_OPTION";
export const disposedDiagnostic = "VIZE_UI_VIRTUALIZER_DISPOSED";
export const setupDiagnostic = "VIZE_UI_VIRTUALIZER_SETUP";

const orientations = new Set<VirtualizerOrientation>(["horizontal", "vertical"]);

/** Live readers over validated virtualizer options. */
export interface VirtualizerOptionReaders {
  readonly readCount: () => number;
  readonly readOrientation: () => VirtualizerOrientation;
  readonly readGap: () => number;
  readonly readLanes: () => number;
  readonly readOverscan: () => number;
  readonly readStickyIndexes: () => readonly number[];
  readonly resolveBaseSize: (index: number) => number;
  readonly usesExactSizes: () => boolean;
  readonly getItemKey: (index: number) => string | number;
  readonly paddingStart: number;
  readonly paddingEnd: number;
  readonly anchorScroll: boolean;
  readonly initialRect: VirtualizerRect;
  readonly initialScrollOffset: number;
}

function fail(message: string): never {
  throw new TypeError(`${invalidOptionDiagnostic}: ${message}`);
}

function readNumber(
  value: unknown,
  name: string,
  {
    integer = false,
    minimum = 0,
    fallback,
  }: { integer?: boolean; minimum?: number; fallback: number },
): number {
  const resolved = toValue(value) ?? fallback;
  if (
    typeof resolved !== "number" ||
    !Number.isFinite(resolved) ||
    resolved < minimum ||
    (integer && !Number.isInteger(resolved))
  ) {
    fail(
      `${name} must resolve to a${integer ? "n integer" : " finite number"} of at least ${minimum}`,
    );
  }
  return resolved;
}

function readSizeSource(
  value: number | ((index: number) => number),
  name: string,
): (index: number) => number {
  if (typeof value === "function") return value;
  if (typeof value === "number" && Number.isFinite(value) && value >= 0) return () => value;
  return fail(`${name} must be a non-negative pixel size or a resolver function`);
}

function readStaticNumber(value: number | undefined, name: string, fallback: number): number {
  if (value === undefined) return fallback;
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) {
    fail(`${name} must be a finite non-negative number`);
  }
  return value;
}

/** Validate static options once and wrap reactive options in checked readers. */
export function resolveVirtualizerOptions(options: VirtualizerOptions): VirtualizerOptionReaders {
  for (const name of ["getItemKey", "onRangeChange"] as const) {
    const callback = options[name];
    if (callback !== undefined && typeof callback !== "function") {
      fail(`${name} must be a function`);
    }
  }
  if (options.anchorScroll !== undefined && typeof options.anchorScroll !== "boolean") {
    fail("anchorScroll must be a boolean");
  }
  if (options.itemSize === undefined && options.estimateItemSize === undefined) {
    fail("one of itemSize or estimateItemSize is required");
  }

  const usesExactSizes = options.itemSize !== undefined;
  const resolveBaseSize = usesExactSizes
    ? readSizeSource(options.itemSize as number | ((index: number) => number), "itemSize")
    : readSizeSource(
        options.estimateItemSize as number | ((index: number) => number),
        "estimateItemSize",
      );

  const initialRect = Object.freeze({
    width: readStaticNumber(options.initialRect?.width, "initialRect.width", 0),
    height: readStaticNumber(options.initialRect?.height, "initialRect.height", 0),
  });

  const readers: VirtualizerOptionReaders = Object.freeze({
    readCount: () => readNumber(options.count, "count", { integer: true, fallback: 0 }),
    readOrientation() {
      const resolved = toValue(options.orientation) ?? "vertical";
      if (!orientations.has(resolved)) {
        fail("orientation must resolve to horizontal or vertical");
      }
      return resolved;
    },
    readGap: () => readNumber(options.gap, "gap", { fallback: 0 }),
    readLanes: () => readNumber(options.lanes, "lanes", { integer: true, minimum: 1, fallback: 1 }),
    readOverscan: () => readNumber(options.overscan, "overscan", { integer: true, fallback: 2 }),
    readStickyIndexes() {
      const resolved = toValue(options.stickyIndexes) ?? [];
      if (!Array.isArray(resolved)) fail("stickyIndexes must resolve to an array of indexes");
      return resolved;
    },
    resolveBaseSize,
    usesExactSizes: () => usesExactSizes,
    getItemKey: options.getItemKey ?? ((index: number) => index),
    paddingStart: readStaticNumber(options.paddingStart, "paddingStart", 0),
    paddingEnd: readStaticNumber(options.paddingEnd, "paddingEnd", 0),
    anchorScroll: options.anchorScroll ?? true,
    initialRect,
    initialScrollOffset: readStaticNumber(options.initialScrollOffset, "initialScrollOffset", 0),
  });

  // Validate statically resolvable reactive options eagerly, like other families do.
  if (typeof options.count !== "function") readers.readCount();
  if (typeof options.orientation !== "function") readers.readOrientation();
  if (typeof options.gap !== "function") readers.readGap();
  if (typeof options.lanes !== "function") readers.readLanes();
  if (typeof options.overscan !== "function") readers.readOverscan();
  if (typeof options.stickyIndexes !== "function") readers.readStickyIndexes();

  return readers;
}

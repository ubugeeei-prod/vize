import type {
  ScrubberPreviewCue,
  ScrubberPreviewDirection,
  ScrubberPreviewFrame,
  ScrubberPreviewRegion,
  ScrubberPreviewSprite,
} from "./scrubber-preview-types.ts";

const TIMING =
  /^\s*((?:\d+:)?\d{1,2}:\d{2}\.\d{3})\s+-->\s+((?:\d+:)?\d{1,2}:\d{2}\.\d{3})(?:\s.*)?$/u;
const XYWH = /#xywh=(?:pixel:)?(\d+),(\d+),(\d+),(\d+)$/u;

function parseTimestamp(value: string): number {
  const parts = value.split(":").map(Number);
  const seconds = parts.pop() ?? 0;
  const minutes = parts.pop() ?? 0;
  const hours = parts.pop() ?? 0;
  return hours * 3600 + minutes * 60 + seconds;
}

function resolveUrl(url: string, baseUrl: string | undefined): string {
  if (baseUrl === undefined) return url;
  try {
    return new URL(url, baseUrl).href;
  } catch {
    return url;
  }
}

/**
 * Parse a WebVTT thumbnails track: each cue's first payload line is an image
 * URL, optionally with a `#xywh=x,y,w,h` (or `#xywh=pixel:…`) sprite region.
 * Relative URLs resolve against `baseUrl` when given. Malformed cues are
 * skipped; the result is sorted by start time and frozen.
 */
export function parseThumbnailVtt(text: string, baseUrl?: string): readonly ScrubberPreviewCue[] {
  const lines = text.replace(/\r\n?/gu, "\n").split("\n");
  const cues: ScrubberPreviewCue[] = [];
  for (let index = 0; index < lines.length; index++) {
    const timing = TIMING.exec(lines[index] ?? "");
    if (timing === null) continue;
    const start = parseTimestamp(timing[1] ?? "");
    const end = parseTimestamp(timing[2] ?? "");
    const payload = (lines[index + 1] ?? "").trim();
    if (payload.length === 0 || !(end > start)) continue;
    const match = XYWH.exec(payload);
    const url = match === null ? payload : payload.slice(0, match.index);
    const region: ScrubberPreviewRegion | null =
      match === null
        ? null
        : Object.freeze({
            x: Number(match[1]),
            y: Number(match[2]),
            width: Number(match[3]),
            height: Number(match[4]),
          });
    cues.push(Object.freeze({ start, end, src: resolveUrl(url, baseUrl), region }));
    index += 1;
  }
  cues.sort((left, right) => left.start - right.start);
  return Object.freeze(cues);
}

/** Cue covering `time` (start inclusive, end exclusive), found by binary search. */
export function findThumbnailCue(
  cues: readonly ScrubberPreviewCue[],
  time: number,
): ScrubberPreviewCue | undefined {
  let low = 0;
  let high = cues.length - 1;
  let found: ScrubberPreviewCue | undefined;
  while (low <= high) {
    const middle = (low + high) >>> 1;
    const cue = cues[middle];
    if (cue === undefined) break;
    if (cue.start <= time) {
      found = cue;
      low = middle + 1;
    } else {
      high = middle - 1;
    }
  }
  return found !== undefined && time < found.end ? found : undefined;
}

/**
 * Sprite frame for `time`: the frame index is `floor(time / interval)`,
 * clamped to `frameCount`; frames fill each sheet row by row.
 *
 * @throws {RangeError} When the sprite geometry is not positive and finite.
 */
export function spriteFrame(sprite: ScrubberPreviewSprite, time: number): ScrubberPreviewFrame {
  const { columns, rows, interval, width, height } = sprite;
  for (const value of [columns, rows, interval, width, height]) {
    if (!(value > 0) || !Number.isFinite(value)) {
      throw new RangeError(
        "VIZE_UI_SCRUBBER_PREVIEW_SPRITE: columns, rows, interval, width, and height must be positive",
      );
    }
  }
  const last = sprite.frameCount === undefined ? Number.POSITIVE_INFINITY : sprite.frameCount - 1;
  const index = Math.max(0, Math.min(last, Math.floor(Math.max(0, time) / interval)));
  const perSheet = Math.floor(columns) * Math.floor(rows);
  const sheet = Math.floor(index / perSheet);
  const within = index % perSheet;
  const column = within % Math.floor(columns);
  const row = Math.floor(within / Math.floor(columns));
  const src = typeof sprite.src === "function" ? sprite.src(sheet) : sprite.src;
  return Object.freeze({
    src,
    region: Object.freeze({ x: column * width, y: row * height, width, height }),
  });
}

/** Position `0..1` of `clientX` along a track box; RTL tracks run right to left. */
export function ratioFromPointer(
  clientX: number,
  rect: { readonly left: number; readonly width: number },
  dir: ScrubberPreviewDirection = "ltr",
): number {
  if (!(rect.width > 0)) return 0;
  const ratio = Math.min(1, Math.max(0, (clientX - rect.left) / rect.width));
  return dir === "rtl" ? 1 - ratio : ratio;
}

/** Snap a time down to a multiple of `interval` seconds (used as a capture cache key). */
export function quantizeTime(time: number, interval: number): number {
  if (!(interval > 0)) return Math.max(0, time);
  return Math.max(0, Math.floor(time / interval) * interval);
}

/** Format seconds as `m:ss`, or `h:mm:ss` from one hour (or when `forceHours`). */
export function formatScrubberTime(seconds: number, forceHours = false): string {
  const total = Number.isFinite(seconds) && seconds > 0 ? Math.floor(seconds) : 0;
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const rest = String(total % 60).padStart(2, "0");
  return hours > 0 || forceHours
    ? `${hours}:${String(minutes).padStart(2, "0")}:${rest}`
    : `${minutes}:${rest}`;
}

/** Least-recently-used cache of captured frame URLs. */
export interface FrameCache {
  /** Cached URL for a key, refreshing its recency. */
  readonly get: (key: number) => string | undefined;

  /** Store a URL, evicting the least recently used entry beyond the limit. */
  readonly set: (key: number, url: string) => void;

  /** Evict every entry. */
  readonly clear: () => void;

  /** Number of cached entries. */
  readonly size: () => number;
}

/** Create an LRU frame cache; `onEvict` receives each evicted URL (e.g. to revoke it). */
export function createFrameCache(limit: number, onEvict: (url: string) => void): FrameCache {
  const entries = new Map<number, string>();
  const capacity = Math.max(1, Math.floor(limit));
  return Object.freeze({
    get(key: number) {
      const url = entries.get(key);
      if (url === undefined) return undefined;
      entries.delete(key);
      entries.set(key, url);
      return url;
    },
    set(key: number, url: string) {
      const previous = entries.get(key);
      if (previous !== undefined && previous !== url) onEvict(previous);
      entries.delete(key);
      entries.set(key, url);
      while (entries.size > capacity) {
        const [oldestKey, oldestUrl] = entries.entries().next().value ?? [];
        if (oldestKey === undefined || oldestUrl === undefined) break;
        entries.delete(oldestKey);
        onEvict(oldestUrl);
      }
    },
    clear() {
      for (const url of entries.values()) onEvict(url);
      entries.clear();
    },
    size: () => entries.size,
  });
}

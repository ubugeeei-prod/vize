import type {
  CropArea,
  CropConstraints,
  CropPoint,
  CropSize,
  ImageCropperHandlePosition,
} from "./image-cropper-types.ts";

/**
 * Pure geometry for the image cropper.
 *
 * Crop areas live in the rotated image's bounding box, measured in natural
 * image pixels: rotating a 400x200 image by 90 degrees yields 200x400 bounds.
 * The display maps bounds to the viewport with `scale = fitScale * zoom` around
 * a view center that is clamped so zoomed content never exposes empty space.
 */

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function positive(value: number | undefined, fallback: number): number {
  return value !== undefined && Number.isFinite(value) && value > 0 ? value : fallback;
}

/** Normalize degrees into `[0, 360)`. Non-finite input normalizes to `0`. */
export function normalizeRotation(degrees: number): number {
  if (!Number.isFinite(degrees)) return 0;
  const normalized = ((degrees % 360) + 360) % 360;
  return Object.is(normalized, -0) ? 0 : normalized;
}

/** Bounding box of an image rotated by `degrees` around its center. */
export function rotatedBounds(size: CropSize, degrees: number): CropSize {
  const rotation = normalizeRotation(degrees);
  if (rotation % 180 === 0) return { width: size.width, height: size.height };
  if (rotation % 90 === 0) return { width: size.height, height: size.width };
  const radians = (rotation * Math.PI) / 180;
  const cos = Math.abs(Math.cos(radians));
  const sin = Math.abs(Math.sin(radians));
  return {
    width: size.width * cos + size.height * sin,
    height: size.width * sin + size.height * cos,
  };
}

interface Limits {
  readonly minWidth: number;
  readonly minHeight: number;
  readonly maxWidth: number;
  readonly maxHeight: number;
  readonly ratio: number | undefined;
}

function limitsOf(bounds: CropSize, constraints: CropConstraints): Limits {
  const ratio =
    constraints.aspectRatio !== undefined &&
    Number.isFinite(constraints.aspectRatio) &&
    constraints.aspectRatio > 0
      ? constraints.aspectRatio
      : undefined;
  const maxWidth = Math.min(positive(constraints.maxWidth, Infinity), bounds.width);
  const maxHeight = Math.min(positive(constraints.maxHeight, Infinity), bounds.height);
  return {
    maxHeight,
    maxWidth,
    minHeight: Math.min(positive(constraints.minHeight, 1), maxHeight),
    minWidth: Math.min(positive(constraints.minWidth, 1), maxWidth),
    ratio,
  };
}

/** Largest width allowed by every limit, for a locked aspect ratio. */
function lockedWidthRange(limits: Limits, ratio: number, availWidth: number, availHeight: number) {
  const max = Math.min(limits.maxWidth, availWidth, limits.maxHeight * ratio, availHeight * ratio);
  const min = Math.min(Math.max(limits.minWidth, limits.minHeight * ratio), max);
  return { min, max };
}

/**
 * Fit a crop inside `bounds`, honoring min/max size and the aspect ratio.
 *
 * Size is clamped first (bounds win over minimums), then the position is
 * clamped so the whole area stays inside the bounds.
 */
export function clampCrop(
  crop: CropArea,
  bounds: CropSize,
  constraints: CropConstraints = {},
): CropArea {
  const limits = limitsOf(bounds, constraints);
  let width: number;
  let height: number;
  if (limits.ratio === undefined) {
    width = clamp(crop.width, limits.minWidth, limits.maxWidth);
    height = clamp(crop.height, limits.minHeight, limits.maxHeight);
  } else {
    const range = lockedWidthRange(limits, limits.ratio, bounds.width, bounds.height);
    width = clamp(crop.width, range.min, range.max);
    height = width / limits.ratio;
  }
  return {
    x: clamp(Number.isFinite(crop.x) ? crop.x : 0, 0, Math.max(0, bounds.width - width)),
    y: clamp(Number.isFinite(crop.y) ? crop.y : 0, 0, Math.max(0, bounds.height - height)),
    width,
    height,
  };
}

/**
 * The largest centered crop covering `coverage` of the bounds on its limiting
 * axis, honoring the aspect ratio and size limits.
 */
export function centeredCrop(
  bounds: CropSize,
  constraints: CropConstraints = {},
  coverage = 1,
): CropArea {
  const fraction = clamp(Number.isFinite(coverage) ? coverage : 1, 0, 1);
  const limits = limitsOf(bounds, constraints);
  let width = bounds.width * fraction;
  let height = bounds.height * fraction;
  if (limits.ratio !== undefined) {
    if (width / height > limits.ratio) width = height * limits.ratio;
    else height = width / limits.ratio;
  }
  const sized = clampCrop({ x: 0, y: 0, width, height }, bounds, constraints);
  return {
    ...sized,
    x: (bounds.width - sized.width) / 2,
    y: (bounds.height - sized.height) / 2,
  };
}

/** Translate a crop by `(dx, dy)` bounds pixels, keeping it inside the bounds. */
export function moveCrop(crop: CropArea, dx: number, dy: number, bounds: CropSize): CropArea {
  return {
    ...crop,
    x: clamp(crop.x + dx, 0, Math.max(0, bounds.width - crop.width)),
    y: clamp(crop.y + dy, 0, Math.max(0, bounds.height - crop.height)),
  };
}

/**
 * Resize a crop by dragging one handle by `(dx, dy)` bounds pixels.
 *
 * The opposite edge or corner stays anchored. Edges never cross. With an aspect
 * ratio, corners follow the dominant drag axis and edge handles grow the other
 * axis symmetrically around the crop center, shifting only when a bound is hit.
 */
export function resizeCrop(
  crop: CropArea,
  handle: ImageCropperHandlePosition,
  dx: number,
  dy: number,
  bounds: CropSize,
  constraints: CropConstraints = {},
): CropArea {
  const limits = limitsOf(bounds, constraints);
  const west = handle.includes("w");
  const east = handle.includes("e");
  const north = handle.includes("n");
  const south = handle.includes("s");
  const left = crop.x;
  const top = crop.y;
  const right = crop.x + crop.width;
  const bottom = crop.y + crop.height;

  if (limits.ratio === undefined) {
    let nextLeft = left;
    let nextRight = right;
    let nextTop = top;
    let nextBottom = bottom;
    if (west)
      nextLeft = clamp(left + dx, Math.max(0, right - limits.maxWidth), right - limits.minWidth);
    if (east) {
      nextRight = clamp(
        right + dx,
        left + limits.minWidth,
        Math.min(bounds.width, left + limits.maxWidth),
      );
    }
    if (north)
      nextTop = clamp(top + dy, Math.max(0, bottom - limits.maxHeight), bottom - limits.minHeight);
    if (south) {
      nextBottom = clamp(
        bottom + dy,
        top + limits.minHeight,
        Math.min(bounds.height, top + limits.maxHeight),
      );
    }
    return { x: nextLeft, y: nextTop, width: nextRight - nextLeft, height: nextBottom - nextTop };
  }

  const ratio = limits.ratio;
  const horizontal = west || east;
  const vertical = north || south;
  const growX = west ? -dx : east ? dx : 0;
  const growY = north ? -dy : south ? dy : 0;

  if (horizontal && vertical) {
    const candidate = Math.max(crop.width + growX, (crop.height + growY) * ratio);
    const availWidth = west ? right : bounds.width - left;
    const availHeight = north ? bottom : bounds.height - top;
    const range = lockedWidthRange(limits, ratio, availWidth, availHeight);
    const width = clamp(candidate, range.min, range.max);
    const height = width / ratio;
    return {
      x: west ? right - width : left,
      y: north ? bottom - height : top,
      width,
      height,
    };
  }

  if (horizontal) {
    const availWidth = west ? right : bounds.width - left;
    const range = lockedWidthRange(limits, ratio, availWidth, bounds.height);
    const width = clamp(crop.width + growX, range.min, range.max);
    const height = width / ratio;
    const centerY = top + crop.height / 2;
    return {
      x: west ? right - width : left,
      y: clamp(centerY - height / 2, 0, bounds.height - height),
      width,
      height,
    };
  }

  const availHeight = north ? bottom : bounds.height - top;
  const range = lockedWidthRange(limits, ratio, bounds.width, availHeight);
  const width = clamp((crop.height + growY) * ratio, range.min, range.max);
  const height = width / ratio;
  const centerX = left + crop.width / 2;
  return {
    x: clamp(centerX - width / 2, 0, bounds.width - width),
    y: north ? bottom - height : top,
    width,
    height,
  };
}

/**
 * Re-fit a crop after the bounds change (e.g. a rotation): the crop center keeps
 * its relative position and the result is clamped into the new bounds.
 */
export function refitCrop(
  crop: CropArea,
  previous: CropSize,
  next: CropSize,
  constraints: CropConstraints = {},
): CropArea {
  if (previous.width <= 0 || previous.height <= 0) return centeredCrop(next, constraints);
  const centerX = ((crop.x + crop.width / 2) / previous.width) * next.width;
  const centerY = ((crop.y + crop.height / 2) / previous.height) * next.height;
  const sized = clampCrop(
    { x: 0, y: 0, width: crop.width, height: crop.height },
    next,
    constraints,
  );
  return clampCrop(
    { ...sized, x: centerX - sized.width / 2, y: centerY - sized.height / 2 },
    next,
    constraints,
  );
}

/** Scale that fits the whole bounds inside the viewport (`contain`). `0` when unmeasured. */
export function fitScale(viewport: CropSize, bounds: CropSize): number {
  if (viewport.width <= 0 || viewport.height <= 0 || bounds.width <= 0 || bounds.height <= 0) {
    return 0;
  }
  return Math.min(viewport.width / bounds.width, viewport.height / bounds.height);
}

/**
 * Clamp a view center (bounds pixels) so zoomed content covers the viewport.
 * Axes whose content fits the viewport stay centered.
 */
export function clampViewCenter(
  center: CropPoint,
  bounds: CropSize,
  viewport: CropSize,
  scale: number,
): CropPoint {
  const axis = (value: number, extent: number, visible: number): number => {
    if (scale <= 0 || extent * scale <= visible || !Number.isFinite(value)) return extent / 2;
    const half = visible / (2 * scale);
    return clamp(value, half, extent - half);
  };
  return {
    x: axis(center.x, bounds.width, viewport.width),
    y: axis(center.y, bounds.height, viewport.height),
  };
}

/**
 * View center after zooming from `scale` to `nextScale` so the bounds point
 * `anchor` stays under the same viewport position.
 */
export function zoomAroundPoint(
  center: CropPoint,
  anchor: CropPoint,
  scale: number,
  nextScale: number,
): CropPoint {
  if (scale <= 0 || nextScale <= 0) return center;
  const factor = scale / nextScale;
  return {
    x: anchor.x - (anchor.x - center.x) * factor,
    y: anchor.y - (anchor.y - center.y) * factor,
  };
}

/** Map a bounds point to viewport pixels. */
export function toViewport(
  point: CropPoint,
  center: CropPoint,
  viewport: CropSize,
  scale: number,
): CropPoint {
  return {
    x: (point.x - center.x) * scale + viewport.width / 2,
    y: (point.y - center.y) * scale + viewport.height / 2,
  };
}

/** Map a viewport pixel to bounds pixels. */
export function fromViewport(
  point: CropPoint,
  center: CropPoint,
  viewport: CropSize,
  scale: number,
): CropPoint {
  if (scale <= 0) return center;
  return {
    x: (point.x - viewport.width / 2) / scale + center.x,
    y: (point.y - viewport.height / 2) / scale + center.y,
  };
}

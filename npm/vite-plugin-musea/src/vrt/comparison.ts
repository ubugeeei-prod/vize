/**
 * Pixel comparison utilities for VRT.
 *
 * Provides PNG reading/writing, color delta calculation using YIQ color space,
 * and anti-aliasing detection for pixel-level image comparison.
 */

import fs from "node:fs";
import { PNG } from "pngjs";

/**
 * Read PNG file and return PNG object.
 */
export async function readPng(filepath: string): Promise<PNG> {
  return new Promise((resolve, reject) => {
    fs.createReadStream(filepath)
      .pipe(new PNG())
      .on("parsed", function (this: PNG) {
        resolve(this);
      })
      .on("error", reject);
  });
}

/**
 * Write PNG object to file.
 */
export async function writePng(png: PNG, filepath: string): Promise<void> {
  return new Promise((resolve, reject) => {
    png.pack().pipe(fs.createWriteStream(filepath)).on("finish", resolve).on("error", reject);
  });
}

/**
 * Calculate color delta using YIQ color space.
 */
export function colorDelta(
  r1: number,
  g1: number,
  b1: number,
  a1: number,
  r2: number,
  g2: number,
  b2: number,
  a2: number,
): number {
  if (a1 !== 255) {
    r1 = blend(r1, 255, a1 / 255);
    g1 = blend(g1, 255, a1 / 255);
    b1 = blend(b1, 255, a1 / 255);
  }
  if (a2 !== 255) {
    r2 = blend(r2, 255, a2 / 255);
    g2 = blend(g2, 255, a2 / 255);
    b2 = blend(b2, 255, a2 / 255);
  }

  const y1 = r1 * 0.29889531 + g1 * 0.58662247 + b1 * 0.11448223;
  const i1 = r1 * 0.59597799 - g1 * 0.2741761 - b1 * 0.32180189;
  const q1 = r1 * 0.21147017 - g1 * 0.52261711 + b1 * 0.31114694;

  const y2 = r2 * 0.29889531 + g2 * 0.58662247 + b2 * 0.11448223;
  const i2 = r2 * 0.59597799 - g2 * 0.2741761 - b2 * 0.32180189;
  const q2 = r2 * 0.21147017 - g2 * 0.52261711 + b2 * 0.31114694;

  const dy = y1 - y2;
  const di = i1 - i2;
  const dq = q1 - q2;

  return dy * dy * 0.5053 + di * di * 0.299 + dq * dq * 0.1957;
}

/**
 * Blend foreground with background using alpha.
 */
function blend(fg: number, bg: number, alpha: number): number {
  return bg + (fg - bg) * alpha;
}

/**
 * Check if file exists.
 */
export async function fileExists(filepath: string): Promise<boolean> {
  try {
    await fs.promises.access(filepath);
    return true;
  } catch {
    return false;
  }
}

/**
 * Simple anti-aliasing detection.
 * A pixel is likely anti-aliased if its neighbors have high contrast in opposite directions.
 */
export function isAntiAliased(
  img1: PNG,
  img2: PNG,
  x: number,
  y: number,
  width: number,
  height: number,
): boolean {
  const minX = Math.max(0, x - 1);
  const maxX = Math.min(width - 1, x + 1);
  const minY = Math.max(0, y - 1);
  const maxY = Math.min(height - 1, y + 1);

  let zeroes = 0;
  let positives = 0;
  let negatives = 0;

  for (let ny = minY; ny <= maxY; ny++) {
    for (let nx = minX; nx <= maxX; nx++) {
      if (nx === x && ny === y) continue;
      const idx = (ny * width + nx) * 4;

      const delta = colorDelta(
        img1.data[idx],
        img1.data[idx + 1],
        img1.data[idx + 2],
        img1.data[idx + 3],
        img2.data[idx],
        img2.data[idx + 1],
        img2.data[idx + 2],
        img2.data[idx + 3],
      );

      if (delta === 0) {
        zeroes++;
      } else if (delta > 0) {
        positives++;
      } else {
        negatives++;
      }
    }
  }

  // If neighbors are mixed (some match, some differ), it's likely AA
  return zeroes > 0 && (positives > 0 || negatives > 0) && positives + negatives < 4;
}

/**
 * Simple glob matching for pattern-based filtering.
 */
export function matchGlob(filepath: string, pattern: string): boolean {
  const regex = pattern
    .replace(/\./g, "\\.")
    .replace(/\*\*/g, ".*")
    .replace(/\*(?!\*)/g, "[^/]*");
  return new RegExp(`^${regex}$`).test(filepath);
}

/**
 * Escape HTML special characters.
 */
export function escapeHtml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#x27;");
}

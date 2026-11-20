import fs from "node:fs";
import path from "node:path";

import { resolveInside } from "../security.js";
import type {
  ArtFileInfo,
  MuseaExternalArtStatus,
  MuseaExternalStatusValue,
} from "../types/index.js";

const EXTERNAL_STATUS_NAMES = new Set(["success", "warning", "error", "pending"]);

export interface ResolvedExternalStatuses {
  filePath?: string;
  values: ReadonlyMap<string, MuseaExternalArtStatus>;
}

export async function resolveExternalStatuses(
  root: string,
  source: string | Record<string, MuseaExternalStatusValue> | undefined,
): Promise<ResolvedExternalStatuses> {
  if (!source) return { values: new Map() };

  const filePath = typeof source === "string" ? resolveInside(root, source, "statuses") : undefined;
  const input = filePath
    ? (JSON.parse(await fs.promises.readFile(filePath, "utf8")) as unknown)
    : source;
  const record = statusRecord(input);
  const values = new Map<string, MuseaExternalArtStatus>();

  for (const [key, value] of Object.entries(record)) {
    values.set(normalizeKey(key), normalizeStatus(key, value));
  }
  return { filePath, values };
}

export function externalStatusForArt(
  art: ArtFileInfo,
  root: string,
  statuses: ReadonlyMap<string, MuseaExternalArtStatus>,
): MuseaExternalArtStatus | undefined {
  const relativePath = normalizeKey(path.relative(root, art.path));
  return (
    statuses.get(normalizeKey(art.path)) ??
    statuses.get(relativePath) ??
    statuses.get(`./${relativePath}`) ??
    statuses.get(art.metadata.title)
  );
}

function statusRecord(input: unknown): Record<string, unknown> {
  if (!input || typeof input !== "object" || Array.isArray(input)) {
    throw new Error("[musea] statuses must be a JSON object");
  }
  const record = input as Record<string, unknown>;
  const nested = record.statuses;
  if (nested && typeof nested === "object" && !Array.isArray(nested)) {
    return nested as Record<string, unknown>;
  }
  return record;
}

function normalizeStatus(key: string, value: unknown): MuseaExternalArtStatus {
  if (typeof value === "string" && EXTERNAL_STATUS_NAMES.has(value)) {
    return { status: value as MuseaExternalArtStatus["status"] };
  }
  if (value && typeof value === "object" && !Array.isArray(value)) {
    const { status, label } = value as Record<string, unknown>;
    if (typeof status === "string" && EXTERNAL_STATUS_NAMES.has(status)) {
      return {
        status: status as MuseaExternalArtStatus["status"],
        ...(typeof label === "string" ? { label } : {}),
      };
    }
  }
  throw new Error(
    `[musea] invalid external status for ${JSON.stringify(key)} (expected success | warning | error | pending)`,
  );
}

function normalizeKey(value: string): string {
  return value.replaceAll("\\", "/").split(path.sep).join("/");
}

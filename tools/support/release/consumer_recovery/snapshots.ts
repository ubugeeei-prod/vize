import assert from "node:assert/strict";
import { object, same, type Json } from "./policy.ts";

// GitHub embeds current repository clocks in historical run responses. Only
// these values are mutable here; presence, type and every other field remain exact.
// Comparison views never rewrite the retained raw evidence or signed receipt.
function repositoryView(value: unknown): Json {
  const view = { ...object(value) };
  for (const field of ["pushed_at", "updated_at"]) {
    if (!Object.hasOwn(view, field)) continue;
    assert.ok(
      typeof view[field] === "string" || view[field] === null,
      `repository ${field} must be a string or null`,
    );
    if (typeof view[field] === "string") view[field] = "";
  }
  return view;
}

function runView(value: unknown): Json {
  const view = { ...object(value) };
  for (const field of ["repository", "head_repository"])
    if (Object.hasOwn(view, field)) view[field] = repositoryView(view[field]);
  return view;
}

export function sameRunSnapshot(actual: unknown, expected: unknown, message: string) {
  same(runView(actual), runView(expected), message);
}

export function sameOriginalSnapshot(actual: Json, expected: Json, phase: string) {
  for (const key of [
    "policy",
    "run",
    "jobs",
    "failed",
    "log",
    "logIdentity",
    "blobs",
    "artifacts",
    "pin",
    "sourceReceipt",
  ]) {
    const compare = key === "run" ? sameRunSnapshot : same;
    compare(actual[key], expected[key], `original ${key} changed ${phase}`);
  }
}

#!/usr/bin/env -S vp node
/** Make the existing metric readbacks available to their host consumer. */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export function exposeReadbacks(source: string): string {
  const signatures = ["span_snapshot", "counter_snapshot"].map((name) => ({
    old: `pub(super) fn ${name}(&self)`,
    next: `pub fn ${name}(&self)`,
  }));
  const states = signatures.map(({ old, next }) => {
    const oldCount = source.split(old).length - 1;
    const nextCount = source.split(next).length - 1;
    if (oldCount + nextCount !== 1) throw new Error("ambiguous metric readback signature");
    return oldCount === 1 ? "old" : "next";
  });
  if (states[0] !== states[1]) throw new Error("partial metric readback exposure");
  return signatures
    .reduce((text, { old, next }) => text.replace(old, next), source)
    .replace(
      "/// keys — for the machine-readable export.",
      "/// keys. The returned metrics are owned readbacks of the existing stores.",
    )
    .replace(
      "/// Snapshot every counter for the machine-readable export.",
      "/// Snapshot every counter as an owned readback of the existing stores.",
    );
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
  const file = path.join(root, "davinci/vize_l0/src/profiler/snapshot.rs");
  const source = fs.readFileSync(file, "utf8");
  const next = exposeReadbacks(source);
  const mode = process.argv[2] ?? "integrate";
  if (mode !== "integrate" && mode !== "check") throw new Error(`unknown mode: ${mode}`);
  if (mode === "check" && source !== next) throw new Error("metric readbacks remain private");
  if (mode === "integrate" && source !== next) fs.writeFileSync(file, next);
}

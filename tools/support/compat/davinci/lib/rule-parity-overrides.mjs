// The overrides sidecar: hand-corrections to the heuristic classification.
// Parsed strictly (a typo is a hard failure, not a silently ignored row) so a
// correction either applies or the generator stops.

import { existsSync, readFileSync } from "node:fs";

import { CLASSIFICATIONS, OVERRIDES, OVERRIDES_REL } from "./rule-parity-paths.mjs";

export function parseOverrides() {
  if (!existsSync(OVERRIDES)) {
    throw new Error(`${OVERRIDES_REL} is missing; commit it (empty is fine)`);
  }
  const overrides = new Map(); // rule name -> { classification, reason }
  let current = null;
  const lines = readFileSync(OVERRIDES, "utf8").split("\n");
  for (const [idx, rawLine] of lines.entries()) {
    const line = rawLine.replace(/(^|\s)#.*$/, "").trim();
    if (line === "") continue;
    const header = /^\[overrides\."([^"]+)"\]$/.exec(line);
    if (header) {
      current = { classification: null, reason: null };
      if (overrides.has(header[1])) {
        throw new Error(`${OVERRIDES_REL}:${idx + 1}: duplicate override for ${header[1]}`);
      }
      overrides.set(header[1], current);
      continue;
    }
    const kv = /^([a-z_]+)\s*=\s*"([^"]*)"$/.exec(line);
    if (!kv || current === null) {
      throw new Error(
        `${OVERRIDES_REL}:${idx + 1}: unrecognized line (schema: [overrides."rule"], classification = "…", reason = "…")`,
      );
    }
    if (kv[1] === "classification") {
      if (!CLASSIFICATIONS.includes(kv[2])) {
        throw new Error(`${OVERRIDES_REL}:${idx + 1}: invalid classification "${kv[2]}"`);
      }
      current.classification = kv[2];
    } else if (kv[1] === "reason") {
      current.reason = kv[2];
    } else {
      throw new Error(`${OVERRIDES_REL}:${idx + 1}: unknown key "${kv[1]}"`);
    }
  }
  for (const [name, o] of overrides) {
    if (!o.classification || !o.reason) {
      throw new Error(`${OVERRIDES_REL}: override for ${name} needs classification and reason`);
    }
  }
  return overrides;
}

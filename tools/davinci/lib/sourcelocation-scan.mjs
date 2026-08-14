// SourceLocation scanning helpers (Davinci P0-9).
//
// Split out of sourcelocation-inventory.mjs under the 350-line source-length
// budget: this module owns workspace scanning, member matching, and citation
// resolution; the entry file owns rendering and the CLI.

import { existsSync, readFileSync } from "node:fs";
import path from "node:path";

import { discoverCrates, walkRustFiles } from "./crates.mjs";
import { repoRoot } from "./paths.mjs";
import { stripRust } from "./rust-source.mjs";

export const MEMBERS = ["source", "start.line", "start.column", "end.line", "end.column"];
export const OFFSET_MEMBERS = ["span.start", "span.end"];

export function isLocShaped(ident) {
  return (
    ident === "loc" || ident === "location" || ident.endsWith("_loc") || ident.endsWith("_location")
  );
}

/**
 * Scan stripped Rust text for reads of the given member paths through
 * loc-shaped receivers. Returns [{ index, member }].
 *
 * One regex pass finds `<ident>[()].<member>` shapes, capturing whether the
 * receiver is itself preceded by `.` (a field/method in a chain, e.g.
 * `node.loc.source`) or stands bare (a local, e.g. `loc.source`); the
 * receiver identifier is then filtered to loc-shaped names. Bare receivers
 * additionally require a non-identifier, non-`.` character before them so a
 * rejected chain segment is never recounted as a bare local.
 */
export function scanMembers(stripped, members) {
  const memberAlt = members.map((m) => m.replaceAll(".", "\\.")).join("|");
  const re = new RegExp(
    `(\\.)?([A-Za-z_][A-Za-z0-9_]*)(\\(\\))?\\.(${memberAlt})(?![A-Za-z0-9_])`,
    "g",
  );
  const sites = [];
  let m;
  while ((m = re.exec(stripped)) !== null) {
    const [, chained, receiver, , member] = m;
    if (!isLocShaped(receiver)) continue;
    if (!chained) {
      const before = m.index === 0 ? "" : stripped[m.index - 1];
      if (/[A-Za-z0-9_.]/.test(before)) continue;
    }
    sites.push({ index: m.index, member });
  }
  return sites;
}

export function lineOfIndex(text, index) {
  let line = 1;
  for (let i = 0; i < index; i++) if (text[i] === "\n") line++;
  return line;
}

/**
 * Is the site at `index` test code? True when the file itself is a test
 * module by name, or when the site sits at or after the file's first
 * `#[cfg(test)]` attribute (the repo convention keeps test modules at the
 * end of the file).
 */
export function isTestSite(relPath, stripped, index) {
  const base = path.basename(relPath);
  if (base === "tests.rs" || base.endsWith("_tests.rs")) return true;
  if (relPath.includes("/tests/")) return true;
  const cfgTest = stripped.indexOf("#[cfg(test)]");
  return cfgTest !== -1 && index >= cfgTest;
}

/** Scan the workspace once; returns per-crate stats and the flat site list. */
export function scanWorkspace() {
  const crates = [];
  const allSites = [];
  let offsetReadTotal = 0;
  const offsetReadCrates = new Set();
  for (const crate of discoverCrates()) {
    const counts = Object.fromEntries(MEMBERS.map((member) => [member, 0]));
    let testSites = 0;
    let total = 0;
    for (const file of walkRustFiles(crate.srcDir)) {
      const relPath = path.relative(repoRoot, file).split(path.sep).join("/");
      const stripped = stripRust(readFileSync(file, "utf8"));
      for (const site of scanMembers(stripped, MEMBERS)) {
        counts[site.member] += 1;
        total += 1;
        const test = isTestSite(relPath, stripped, site.index);
        if (test) testSites += 1;
        allSites.push({
          crate: crate.name,
          relPath,
          line: lineOfIndex(stripped, site.index),
          member: site.member,
          test,
        });
      }
      const offsetReads = scanMembers(stripped, OFFSET_MEMBERS).length;
      if (offsetReads > 0) {
        offsetReadTotal += offsetReads;
        offsetReadCrates.add(crate.name);
      }
    }
    if (total > 0) crates.push({ name: crate.name, counts, total, testSites });
  }
  return { crates, allSites, offsetReadTotal, offsetReadCrateCount: offsetReadCrates.size };
}

/** `path:line` of the first inventoried site in the file, or throw. */
export function citeSite(allSites, relPath, member = null) {
  const site = allSites.find(
    (s) => s.relPath === relPath && (member === null || s.member === member),
  );
  if (!site) {
    throw new Error(
      `citation lost: no ${member ?? "inventoried"} read found in ${relPath}; ` +
        `re-classify this consumer in tools/davinci/sourcelocation-inventory.mjs`,
    );
  }
  return `${site.relPath}:${site.line}`;
}

/** `path:line` of the first line matching `anchor` in the file, or throw. */
export function citeAnchor(relPath, anchor) {
  const file = path.join(repoRoot, relPath);
  if (!existsSync(file)) {
    throw new Error(
      `citation lost: ${relPath} does not exist; ` +
        `re-classify this consumer in tools/davinci/sourcelocation-inventory.mjs`,
    );
  }
  const lines = readFileSync(file, "utf8").split("\n");
  const idx = lines.findIndex((line) => anchor.test(line));
  if (idx === -1) {
    throw new Error(
      `citation lost: ${anchor} not found in ${relPath}; ` +
        `re-classify this consumer in tools/davinci/sourcelocation-inventory.mjs`,
    );
  }
  return `${relPath}:${idx + 1}`;
}

/** References to `name` outside its defining module chain (stripped text). */
export function countExternalReferences(name, excludeRelPaths) {
  let count = 0;
  for (const crate of discoverCrates()) {
    for (const file of walkRustFiles(crate.srcDir)) {
      const relPath = path.relative(repoRoot, file).split(path.sep).join("/");
      if (excludeRelPaths.includes(relPath)) continue;
      const stripped = stripRust(readFileSync(file, "utf8"));
      const re = new RegExp(`(^|[^A-Za-z0-9_])${name}([^A-Za-z0-9_]|$)`, "g");
      count += (stripped.match(re) ?? []).length;
    }
  }
  return count;
}

/**
 * Pin a deletion: fail regeneration when `anchor` matches in `relPath` again
 * (or the file itself reappears after `relPath` was deleted whole). The
 * inverse of `citeAnchor`, for migration groups whose consumers stop
 * existing rather than move.
 */
export function assertAnchorAbsent(relPath, anchor, description) {
  const file = path.join(repoRoot, relPath);
  if (!existsSync(file)) return;
  const lines = readFileSync(file, "utf8").split("\n");
  const idx = lines.findIndex((line) => anchor.test(line));
  if (idx !== -1) {
    throw new Error(
      `deletion regressed: ${description} reappeared at ${relPath}:${idx + 1}; ` +
        `it was deleted by Davinci P1-4 — re-classify it in ` +
        `tools/davinci/sourcelocation-inventory.mjs if it is back by design`,
    );
  }
}

/** Pin a workspace-wide deletion of `name` (no references anywhere). */
export function assertSymbolAbsent(name, description) {
  const count = countExternalReferences(name, []);
  if (count !== 0) {
    throw new Error(
      `deletion regressed: ${count} reference(s) to ${name} reappeared (${description}); ` +
        `it was deleted by Davinci P1-4 — re-classify it in ` +
        `tools/davinci/sourcelocation-inventory.mjs if it is back by design`,
    );
  }
}

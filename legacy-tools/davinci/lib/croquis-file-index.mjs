// Per-file symbol index for the consuming crates: stripped text with `use`
// declarations masked out of the counting surface, plus the alias tables that
// map local names to `vize_croquis` items. `pub use` re-export chains are
// followed to a fixpoint (name-level approximation, stated in the artifact).

import { readFileSync } from "node:fs";
import path from "node:path";

import { walkRustFiles } from "./crates.mjs";
import { CROQUIS_CRATE_NAME, repoRoot } from "./paths.mjs";
import { expandUseTree, findUseDecls, maskKeepNewlines, stripRust } from "./rust-source.mjs";

const MAX_ROUNDS = 12;

/**
 * @returns {Map<string, object[]>} crate name -> per-file index entries
 *   ({ rel, raw, masked, entries, itemAliases, moduleAliases, pubUseSites,
 *   globImports })
 */
export function buildFileIndex(crates, crateNames) {
  // Parse every file once.
  const parsed = new Map();
  for (const crate of crates) {
    const files = [];
    for (const abs of walkRustFiles(crate.srcDir)) {
      const raw = readFileSync(abs, "utf8");
      const stripped = stripRust(raw);
      const decls = findUseDecls(stripped);
      // Mask use decls out of the reference-counting text.
      let masked = stripped;
      for (const d of decls) {
        masked =
          masked.slice(0, d.start) +
          maskKeepNewlines(stripped.slice(d.start, d.end)) +
          masked.slice(d.end);
      }
      const entries = [];
      for (const d of decls) {
        for (const entry of expandUseTree(d.body)) {
          entries.push({ ...entry, isPub: d.isPub });
        }
      }
      files.push({
        rel: path.relative(repoRoot, abs).split(path.sep).join("/"),
        raw,
        masked,
        entries,
      });
    }
    parsed.set(crate.name, files);
  }

  // Fixpoint resolution of use entries to vize_croquis items across
  // `pub use` re-export chains (name-level approximation).
  /** crate -> Map(exportedName -> itemName) */
  const crateExports = new Map(crates.map((c) => [c.name, new Map()]));
  /** per file: resolved item aliases / module aliases / counted pub-use sites */
  for (const crate of crates) {
    for (const file of parsed.get(crate.name)) {
      file.itemAliases = new Map(); // localName -> { item, via }
      file.moduleAliases = new Map(); // localName -> module path under vize_croquis
      file.pubUseSites = []; // itemName[]
      file.globImports = [];
    }
  }
  let changed = true;
  let rounds = 0;
  while (changed && rounds < MAX_ROUNDS) {
    changed = false;
    rounds++;
    for (const crate of crates) {
      for (const file of parsed.get(crate.name)) {
        for (const entry of file.entries) {
          if (entry.resolvedDone) continue;
          const root = entry.segments[0];
          let itemName = null;
          let via = null;
          let modulePath = null;
          if (root === CROQUIS_CRATE_NAME) {
            if (entry.glob) {
              file.globImports.push(entry.segments.join("::") + "::*");
              entry.resolvedDone = true;
              changed = true;
              continue;
            }
            const last = entry.segments[entry.segments.length - 1];
            if (entry.self) {
              // `use vize_croquis::foo::{self}` binds module `foo`.
              modulePath = entry.segments.slice(1).join("::");
            } else if (entry.segments.length === 1) {
              // `use vize_croquis;` / `use vize_croquis as vc;`
              modulePath = "";
            } else {
              itemName = last;
              via = entry.segments.join("::");
              // A lowercase final segment may be a module (see moduleAliases
              // handling below) or a fn/const item; both are tracked.
              modulePath = entry.segments.slice(1).join("::");
            }
          } else if (crateNames.has(root) || root === "crate") {
            const exports =
              root === "crate" ? crateExports.get(crate.name) : crateExports.get(root);
            const target = exports?.get(entry.segments[entry.segments.length - 1]);
            if (target) {
              itemName = target;
              via = entry.segments.join("::") + " -> " + target;
            } else {
              continue; // may resolve in a later round
            }
          } else {
            entry.resolvedDone = true;
            changed = true;
            continue;
          }
          const local = entry.alias ?? entry.segments[entry.segments.length - 1];
          if (itemName !== null) {
            file.itemAliases.set(local, { item: itemName, via });
            if (/^[a-z_]/.test(itemName) && modulePath !== null) {
              // Could also be a module path; allow `local::Member` refs.
              file.moduleAliases.set(local, modulePath);
            }
            if (entry.isPub) {
              crateExports.get(crate.name).set(local, itemName);
              file.pubUseSites.push(itemName);
            }
          } else if (modulePath !== null) {
            file.moduleAliases.set(local, modulePath);
          }
          entry.resolvedDone = true;
          changed = true;
        }
      }
    }
  }

  return parsed;
}

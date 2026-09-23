// Consumer resolution: turns the per-file symbol index into product ×
// consuming-crate site counts, plus the naive grep lane used as a cross-check.

import { discoverCrates } from "./crates.mjs";
import { buildFileIndex } from "./croquis-file-index.mjs";
import { collectCroquisProducers } from "./croquis-producers.mjs";
import { byKey } from "./ordering.mjs";
import { CROQUIS_CRATE_NAME } from "./paths.mjs";

export function analyzeConsumers(products) {
  const allCrates = discoverCrates();
  const crates = allCrates.filter((c) => c.name !== CROQUIS_CRATE_NAME);
  const crateNames = new Set(crates.map((c) => c.name));
  const producers = collectCroquisProducers(allCrates);
  const parsed = buildFileIndex(crates, crateNames);

  // Reference + field-access counting.
  const rows = new Map(); // key `${product}\0${crate}` -> { sites, files:Set }
  const nonProduct = new Map(); // same keying for non-product croquis items
  const grepRows = new Map(); // naive lane
  const globFiles = [];

  const bump = (map, product, crate, file, count) => {
    if (count <= 0) return;
    const key = product + "\u0000" + crate;
    if (!map.has(key)) map.set(key, { product, crate, sites: 0, files: new Set() });
    const row = map.get(key);
    row.sites += count;
    row.files.add(file);
  };

  const isProduct = (name) => products.typeProducts.has(name);
  const fieldNames = [...products.fieldProducts.keys()].sort(byKey);
  const fieldAlt = fieldNames.join("|");

  for (const crate of crates) {
    for (const file of parsed.get(crate.name)) {
      const attribute = (name, count) => {
        if (isProduct(name)) bump(rows, name, crate.name, file.rel, count);
        else bump(nonProduct, name, crate.name, file.rel, count);
      };

      for (const g of file.globImports) globFiles.push(`${file.rel}: use ${g}`);
      for (const item of file.pubUseSites) attribute(item, 1);

      // (1) Fully qualified paths in code (masked text has use decls removed).
      let text = file.masked;
      const fqRe = /\bvize_croquis(?:::[a-z_][a-z0-9_]*)*::([A-Za-z_][A-Za-z0-9_]*)/g;
      const fqCounts = new Map();
      text = text.replace(fqRe, (whole, item) => {
        fqCounts.set(item, (fqCounts.get(item) ?? 0) + 1);
        return " ".repeat(whole.length);
      });
      for (const [item, count] of fqCounts) attribute(item, count);

      // (2) Module-alias qualified references: `alias::Member`.
      const memberHits = new Map();
      for (const [local] of file.moduleAliases) {
        const re = new RegExp(`(?<![A-Za-z0-9_:.])${local}::([A-Za-z_][A-Za-z0-9_]*)`, "g");
        text = text.replace(re, (whole, member) => {
          memberHits.set(member, (memberHits.get(member) ?? 0) + 1);
          return " ".repeat(whole.length);
        });
      }
      for (const [member, count] of memberHits) attribute(member, count);

      // (3) Item-alias bare references.
      for (const [local, { item }] of file.itemAliases) {
        const re = new RegExp(`(?<![A-Za-z0-9_:.'])${local}(?![A-Za-z0-9_])`, "g");
        const count = (text.match(re) ?? []).length;
        attribute(item, count);
      }

      // (4) Field accesses on values provably typed `Croquis` in this file.
      if (fieldAlt) {
        const croquisTypeNames = [];
        for (const [local, { item }] of file.itemAliases) {
          if (item === "Croquis") croquisTypeNames.push(local);
        }
        croquisTypeNames.push("vize_croquis::Croquis", "vize_croquis::croquis::Croquis");
        const receivers = new Set();
        const croquisMethods = new Set();
        for (const typeName of croquisTypeNames) {
          const t = typeName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
          // parameter / struct field / let annotations, incl. &, &'a, &mut,
          // Option<&…>, Box/Rc/Arc wrappers
          const annRe = new RegExp(
            `\\b([a-z_][a-z0-9_]*)\\s*:\\s*(?:&(?:'[a-z_][A-Za-z0-9_]*)?\\s*(?:mut\\s+)?)?` +
              `(?:(?:Option|Box|Rc|Arc)\\s*<\\s*&?(?:'[a-z_][A-Za-z0-9_]*)?\\s*)*${t}\\b`,
            "g",
          );
          for (const m of file.masked.matchAll(annRe)) receivers.add(m[1]);
          // methods/functions returning Croquis (same file)
          const fnRe = new RegExp(
            `\\bfn\\s+([a-z_][a-z0-9_]*)\\s*(?:<[^>]*>)?\\s*\\([^)]*\\)\\s*->\\s*[^;{]*\\b${t}\\b`,
            "g",
          );
          for (const m of file.masked.matchAll(fnRe)) croquisMethods.add(m[1]);
        }
        // Receivers bound from workspace-wide croquis producers:
        //   let croquis = drawer.finish();      (pub fn … -> Croquis)
        //   let Some(a) = ctx.analysis() else…  (pub fn … -> Option<&Croquis>)
        //   let summary = result.croquis;       (pub field …: Croquis)
        if (producers.fns.size > 0 || producers.fields.size > 0) {
          const fnAlt = [...producers.fns].sort(byKey).join("|");
          const fieldAltP = [...producers.fields].sort(byKey).join("|");
          const letRe = new RegExp(
            `\\blet\\s+(?:mut\\s+)?(?:Some\\s*\\(\\s*)?(?:mut\\s+)?(?:&\\s*)?([a-z_][a-z0-9_]*)\\s*\\)?\\s*=\\s*([^;]{0,200})`,
            "g",
          );
          const ctorAlt = croquisTypeNames
            .map((t) => t.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
            .join("|");
          for (const m of file.masked.matchAll(letRe)) {
            const rhs = m[2];
            let bound = false;
            if (fnAlt && new RegExp(`[.:](?:${fnAlt})\\s*\\(`).test(rhs)) bound = true;
            if (!bound && fieldAltP && new RegExp(`\\.(?:${fieldAltP})\\b(?!\\s*\\()`).test(rhs))
              bound = true;
            // Constructor / associated calls: `let c = Croquis::default();`
            if (
              !bound &&
              new RegExp(`(?<![A-Za-z0-9_])(?:${ctorAlt})\\s*::\\s*[a-z_][a-z0-9_]*\\s*\\(`).test(
                rhs,
              )
            ) {
              bound = true;
            }
            if (bound) receivers.add(m[1]);
          }
        }
        receivers.delete("self");
        const fieldHits = new Map();
        const seenPositions = new Set();
        const collect = (re) => {
          for (const m of file.masked.matchAll(re)) {
            const field = m[1];
            const pos = m.index + m[0].length;
            if (seenPositions.has(pos)) continue;
            seenPositions.add(pos);
            fieldHits.set(field, (fieldHits.get(field) ?? 0) + 1);
          }
        };
        if (receivers.size > 0) {
          const rAlt = [...receivers].sort(byKey).join("|");
          collect(
            new RegExp(
              `(?<![A-Za-z0-9_])(?:${rAlt})\\s*(?:\\.\\s*[a-z_][a-z0-9_]*\\(\\)\\s*)*\\.\\s*(${fieldAlt})(?![A-Za-z0-9_(])`,
              "g",
            ),
          );
        }
        const chainMethods = new Set([...croquisMethods, ...producers.fns]);
        if (chainMethods.size > 0) {
          const mAlt = [...chainMethods].sort(byKey).join("|");
          collect(
            new RegExp(`\\.\\s*(?:${mAlt})\\(\\)\\s*\\.\\s*(${fieldAlt})(?![A-Za-z0-9_(])`, "g"),
          );
        }
        if (producers.fields.size > 0) {
          // Inline chains through a croquis-typed field: `entry.analysis.race_conditions`.
          const pfAlt = [...producers.fields].sort(byKey).join("|");
          collect(new RegExp(`\\.\\s*(?:${pfAlt})\\s*\\.\\s*(${fieldAlt})(?![A-Za-z0-9_(])`, "g"));
        }
        for (const [field, count] of fieldHits) {
          bump(rows, "Croquis." + field, crate.name, file.rel, count);
        }
      }

      // (5) Naive grep lane over the RAW text (comments/strings included).
      for (const name of products.typeProducts.keys()) {
        const re = new RegExp(`(?<![A-Za-z0-9_])${name}(?![A-Za-z0-9_])`, "g");
        bump(grepRows, name, crate.name, file.rel, (file.raw.match(re) ?? []).length);
      }
      for (const field of fieldNames) {
        const re = new RegExp(`\\.${field}(?![A-Za-z0-9_])`, "g");
        bump(grepRows, "Croquis." + field, crate.name, file.rel, (file.raw.match(re) ?? []).length);
      }
    }
  }

  globFiles.sort(byKey);
  return { crates, rows, nonProduct, grepRows, globFiles };
}

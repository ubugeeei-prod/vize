// Artifact rendering. Ordering is fully determined by lexical sorts so the
// staleness check can byte-compare the committed matrix.

import { formatTable } from "./markdown.mjs";
import { byKey } from "./ordering.mjs";
import { REGEN_COMMAND } from "./paths.mjs";

export function renderArtifact(products, analysis) {
  const { typeProducts, fieldProducts, passthroughs } = products;
  const { rows, nonProduct, grepRows, globFiles } = analysis;

  const productIds = [];
  for (const name of [...typeProducts.keys()].sort(byKey)) {
    productIds.push({ id: name, kind: "type", module: typeProducts.get(name).module });
  }
  for (const name of [...fieldProducts.keys()].sort(byKey)) {
    productIds.push({
      id: "Croquis." + name,
      kind: "field",
      module: "croquis",
      typeText: fieldProducts.get(name).typeText,
    });
  }

  const rowsByProduct = new Map();
  for (const row of rows.values()) {
    if (!rowsByProduct.has(row.product)) rowsByProduct.set(row.product, []);
    rowsByProduct.get(row.product).push(row);
  }
  for (const list of rowsByProduct.values()) list.sort((a, b) => byKey(a.crate, b.crate));

  const consumed = productIds.filter((p) => rowsByProduct.has(p.id));
  const unconsumed = productIds.filter((p) => !rowsByProduct.has(p.id));

  const lines = [];
  lines.push("<!-- GENERATED FILE — do not edit by hand.");
  lines.push(`     Regenerate: ${REGEN_COMMAND}`);
  lines.push("     Verify:     rust-script tools/commands/davinci/croquis-consumers.rs --check");
  lines.push("     Generator:  tools/davinci/croquis-consumers.mjs -->");
  lines.push("");
  lines.push("# Croquis consumption matrix");
  lines.push("");
  lines.push(
    "Which workspace crates consume the public analysis products of" +
      " `crates/vize_croquis`. Mechanizes the 2026-08-13 hand audit in" +
      " [semantic-engine.md](../semantic-engine.md#the-problem-measured) (Davinci P0-7).",
  );
  lines.push("");
  lines.push("## Resolution method (and its limits)");
  lines.push("");
  lines.push("**Product enumeration** — parsed from source, not hardcoded:");
  lines.push("");
  lines.push(
    "- `pub` fields of the `Croquis` struct in `crates/vize_croquis/src/croquis.rs`" +
      " (rows named `Croquis.<field>`), plus the tracker/product types those fields" +
      " reference, resolved through croquis.rs's own `use crate::…` declarations.",
  );
  lines.push(
    "- Types re-exported by croquis.rs (`pub use bindings::…`, `snapshot::…`, …) and" +
      " the crate-root `pub use` groups in `crates/vize_croquis/src/lib.rs` whose" +
      " source is a local module (this is what brings in the `effect_graph`," +
      " `scope`, `symbol`, `analyzer`, `drawer`, and `reactivity_overlay` families).",
  );
  lines.push(
    "- Crate-root passthrough re-exports of foreign items are **excluded** from the" +
      " product set: " +
      passthroughs.map((p) => "`" + p + "`").join(", ") +
      ".",
  );
  lines.push("");
  lines.push("**Consumer resolution** — symbol-aware, per `crates/*/src/**/*.rs`:");
  lines.push("");
  lines.push(
    "- Rust `use` declarations are parsed (brace groups, `as` aliases, `pub use`)" +
      " into per-file alias tables mapping local names to `vize_croquis` items;" +
      " `pub use` re-export chains across crates are followed to a fixpoint" +
      " (name-level, see limits). Comments and string literals are stripped before" +
      " any counting, and the `use` declarations themselves are not counted as" +
      " reference sites (a `pub use` re-export counts as one site).",
  );
  lines.push(
    "- **type rows** — sites are references to a resolved local alias, a module-" +
      "qualified member (`reactivity::ReactiveKind`), or a fully qualified" +
      " `vize_croquis::…` path.",
  );
  lines.push(
    "- **`Croquis.<field>` rows** — sites are field accesses (`summary.bindings`)" +
      " counted only on receivers resolved to `Croquis` values: idents with a" +
      " `Croquis` type annotation (params, struct fields, `let`," +
      " `&`/`&'a`/`&mut`/`Option<&…>`/`Box`/`Rc`/`Arc` wrappers), calls to" +
      " same-file functions returning `Croquis`, and `let`-bindings whose" +
      " right-hand side calls a workspace `pub fn` returning `Croquis`" +
      " (`drawer.finish()`, `ctx.analysis()`), reads a workspace `pub` field" +
      " typed `Croquis` (`result.croquis`), or calls an associated function on" +
      " the `Croquis` type itself (`Croquis::default()`) — producer tables" +
      " parsed from `crates/*/src`, matched by name. Inline chains through" +
      " those producers" +
      " (`entry.analysis.race_conditions`, `ctx.croquis().bindings`) are counted" +
      " too.",
  );
  lines.push("");
  lines.push("**Known limits** (undercounts are possible; the grep lane below bounds them):");
  lines.push("");
  lines.push(
    "- Re-export chains resolve by item **name**, not full module path; same-named" +
      " items reached through different facade modules would be conflated.",
  );
  lines.push(
    "- No type inference: field accesses through closure params, iterator" +
      " chains, destructuring patterns, or re-borrowed locals (`let b = &a;`)" +
      " are not counted; the producer tables match croquis-returning method" +
      " **names** without owner types, so a same-named method on an unrelated" +
      " type can mark a false receiver (only matters if that value also has a" +
      " product-named field).",
  );
  lines.push("- Macro-generated code is invisible to source parsing.");
  lines.push(
    "- `#[cfg(test)]` code inside `src/` is included; `tests/`, `benches/`," +
      " `examples/` directories are not scanned. `vize_croquis` itself is excluded" +
      " (internal use is not consumption). Note that `vize_croquis_cf` is a" +
      " separate crate and therefore counted as an external consumer, even though" +
      " it is part of the same semantic layer.",
  );
  if (globFiles.length > 0) {
    lines.push("- Glob imports of `vize_croquis` (cannot be alias-resolved):");
    for (const g of globFiles) lines.push(`  - ${g}`);
  } else {
    lines.push("- No glob imports (`use vize_croquis::…::*`) exist in the workspace today.");
  }
  lines.push("");

  lines.push("## Products with external consumers");
  lines.push("");
  const productRows = [];
  for (const p of consumed) {
    for (const row of rowsByProduct.get(p.id)) {
      productRows.push([
        `\`${p.id}\``,
        p.kind,
        `\`${p.module}\``,
        `\`${row.crate}\``,
        String(row.files.size),
        String(row.sites),
      ]);
    }
  }
  lines.push(
    formatTable(
      ["product", "kind", "module", "consuming crate", "files", "sites"],
      ["left", "left", "left", "left", "right", "right"],
      productRows,
    ).trimEnd(),
  );
  lines.push("");

  lines.push("## Products with no external consumers");
  lines.push("");
  lines.push(
    "Computed (or exported) by `vize_croquis`, referenced by no other workspace" +
      " crate under the resolution above.",
  );
  lines.push("");
  const unconsumedByModule = new Map();
  for (const p of unconsumed) {
    if (!unconsumedByModule.has(p.module)) unconsumedByModule.set(p.module, []);
    unconsumedByModule.get(p.module).push(p);
  }
  for (const module of [...unconsumedByModule.keys()].sort(byKey)) {
    const items = unconsumedByModule
      .get(module)
      .map((p) => "`" + p.id + "`")
      .join(", ");
    lines.push(`- \`${module}\`: ${items}`);
  }
  lines.push("");

  lines.push("## Non-product `vize_croquis` imports observed");
  lines.push("");
  lines.push(
    "Items consumers import from `vize_croquis` that are outside the product set" +
      " above (module-path items never re-exported at the crate root nor referenced" +
      " by `croquis.rs`). Kept visible so nothing resolved is silently dropped.",
  );
  lines.push("");
  const nonProductRows = [...nonProduct.values()].sort(
    (a, b) => byKey(a.product, b.product) || byKey(a.crate, b.crate),
  );
  lines.push(
    formatTable(
      ["item", "consuming crate", "files", "sites"],
      ["left", "left", "right", "right"],
      nonProductRows.map((row) => [
        `\`${row.product}\``,
        `\`${row.crate}\``,
        String(row.files.size),
        String(row.sites),
      ]),
    ).trimEnd(),
  );
  lines.push("");

  lines.push("## Cross-check: symbol-resolved vs naive grep");
  lines.push("");
  lines.push(
    "The naive lane counts raw word-boundary text matches per product name" +
      " (`\\.field` matches for field rows) over the same files — comments," +
      " strings, doc text, and same-named unrelated symbols included, imports" +
      " included. Disagreements are listed, **not** reconciled: `grep > resolved`" +
      " usually means comments/unrelated same-named symbols (for field rows: field" +
      " accesses on non-`Croquis` receivers); `grep < resolved` would indicate a" +
      " resolver bug and must be investigated.",
  );
  lines.push("");
  const crossCheckRows = [];
  const addTo = (map, key, crate, sites) => {
    if (!map.has(key)) map.set(key, new Map());
    map.get(key).set(crate, sites);
  };
  const resolvedPer = new Map();
  const grepPer = new Map();
  for (const row of rows.values()) addTo(resolvedPer, row.product, row.crate, row.sites);
  for (const row of grepRows.values()) addTo(grepPer, row.product, row.crate, row.sites);
  const allProducts = productIds.map((p) => p.id);
  for (const id of allProducts) {
    const r = resolvedPer.get(id) ?? new Map();
    const g = grepPer.get(id) ?? new Map();
    const crates = [...new Set([...r.keys(), ...g.keys()])].sort(byKey);
    const diffs = crates.filter((c) => (r.get(c) ?? 0) !== (g.get(c) ?? 0));
    if (diffs.length === 0) continue;
    const rTotal = [...r.values()].reduce((a, b) => a + b, 0);
    const gTotal = [...g.values()].reduce((a, b) => a + b, 0);
    const detail = diffs.map((c) => `\`${c}\` (${r.get(c) ?? 0}/${g.get(c) ?? 0})`).join(", ");
    crossCheckRows.push([`\`${id}\``, String(rTotal), String(gTotal), detail]);
  }
  lines.push(
    formatTable(
      ["product", "resolved", "grep", "disagreeing crates (resolved/grep)"],
      ["left", "right", "right", "left"],
      crossCheckRows,
    ).trimEnd(),
  );
  lines.push("");
  return lines.join("\n");
}

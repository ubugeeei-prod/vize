// Shared rendering pieces of the croquis consumption matrix: product
// identities, the resolution-method prose, per-crate grouping, and the
// on-demand `--summary` view (cross-crate aggregates, never committed — they
// would change on every PR and make every other PR conflict). Ordering is
// fully determined by lexical sorts so the staleness check can byte-compare.

import { gateOf, gatedProductIds } from "./croquis-gates.mjs";
import { formatTable } from "./markdown.mjs";
import { byKey } from "./ordering.mjs";

export const SUMMARY_COMMAND = "rust-script tools/commands/davinci/croquis-consumers.rs --summary";

/** Every product in artifact order: type products by name, then `Croquis.<field>` rows. */
export function productIdsOf(products) {
  const { typeProducts, fieldProducts } = products;
  const ids = [];
  for (const name of [...typeProducts.keys()].sort(byKey)) {
    ids.push({ id: name, kind: "type", module: typeProducts.get(name).module });
  }
  for (const name of [...fieldProducts.keys()].sort(byKey)) {
    ids.push({ id: "Croquis." + name, kind: "field", module: "croquis" });
  }
  return ids;
}

/** Per consuming crate: resolved rows, non-product rows, and naive-grep sites by product. */
export function groupByCrate(analysis) {
  const byCrate = new Map();
  const entry = (crate) => {
    if (!byCrate.has(crate)) {
      byCrate.set(crate, { resolved: new Map(), nonProduct: new Map(), grep: new Map() });
    }
    return byCrate.get(crate);
  };
  for (const row of analysis.rows.values()) entry(row.crate).resolved.set(row.product, row);
  for (const row of analysis.nonProduct.values()) entry(row.crate).nonProduct.set(row.product, row);
  for (const row of analysis.grepRows.values()) entry(row.crate).grep.set(row.product, row.sites);
  return byCrate;
}

/** Products whose resolved and naive-grep counts differ in one crate's group. */
export function disagreements(productIds, group) {
  return productIds
    .map((p) => ({
      id: p.id,
      resolved: group.resolved.get(p.id)?.sites ?? 0,
      grep: group.grep.get(p.id) ?? 0,
    }))
    .filter((d) => d.resolved !== d.grep);
}

export function methodLines(products, analysis) {
  const { passthroughs } = products;
  const { globFiles } = analysis;
  const lines = [];
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
  lines.push(
    "- **naive grep lane** (cross-check) — raw word-boundary text matches per" +
      " product name (`\\.field` matches for field rows) over the same files —" +
      " comments, strings, doc text, and same-named unrelated symbols included," +
      " imports included. Disagreements are listed per crate, **not** reconciled:" +
      " `grep > resolved` usually means comments/unrelated same-named symbols (for" +
      " field rows: field accesses on non-`Croquis` receivers); `grep < resolved`" +
      " would indicate a resolver bug and must be investigated.",
  );
  lines.push("");
  lines.push("**Known limits** (undercounts are possible; the naive grep lane bounds them):");
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
  return lines;
}

function sum(values) {
  return values.reduce((a, b) => a + b, 0);
}

// One row per id with at least one consuming crate: lead cells, then the
// crate count and the file/site sums over that id's per-crate rows.
function totalsTable(leadHeaders, ids, rowsById) {
  const rows = [];
  for (const { id, cells } of ids) {
    const list = rowsById.get(id) ?? [];
    if (list.length === 0) continue;
    rows.push([
      ...cells,
      String(list.length),
      String(sum(list.map((row) => row.files.size))),
      String(sum(list.map((row) => row.sites))),
    ]);
  }
  return formatTable(
    [...leadHeaders, "crates", "files", "sites"],
    [...leadHeaders.map(() => "left"), "right", "right", "right"],
    rows,
  ).trimEnd();
}

function rowsBy(map) {
  const out = new Map();
  for (const row of map.values()) {
    if (!out.has(row.product)) out.set(row.product, []);
    out.get(row.product).push(row);
  }
  return out;
}

function orphanProducts(productIds, resolvedBy) {
  return productIds
    .filter((product) => !resolvedBy.has(product.id))
    .sort((left, right) => byKey(left.id, right.id));
}

/** Orphans missing a gate, and gates whose product is no longer an orphan. */
export function gateViolations(products, analysis) {
  const productIds = productIdsOf(products);
  const resolvedBy = rowsBy(analysis.rows);
  const orphans = new Set(orphanProducts(productIds, resolvedBy).map((product) => product.id));
  const missing = [...orphans].filter((id) => gateOf(id) === null).sort(byKey);
  const stale = gatedProductIds()
    .filter((id) => !orphans.has(id))
    .sort(byKey);
  return { missing, stale };
}

/** The on-demand cross-crate view printed by `--summary`. Never committed. */
export function renderSummary(products, analysis) {
  const productIds = productIdsOf(products);
  const resolvedBy = rowsBy(analysis.rows);
  const byCrate = groupByCrate(analysis);
  const crates = [...byCrate.keys()].sort(byKey);
  const lines = [];
  lines.push("# Croquis consumption summary (computed, not committed)");
  lines.push("");
  lines.push(
    "Cross-crate aggregates of the sharded matrix in" +
      " `docs/davinci/plan/croquis-consumption/`. Printed by `" +
      SUMMARY_COMMAND +
      "`; totals are sums over the per-crate shards.",
  );
  lines.push("");
  lines.push("## Products with external consumers");
  lines.push("");
  const productCells = productIds.map((p) => ({
    id: p.id,
    cells: [`\`${p.id}\``, p.kind, `\`${p.module}\``],
  }));
  lines.push(totalsTable(["product", "kind", "module"], productCells, resolvedBy));
  lines.push("");
  lines.push("## Products with no external consumers");
  lines.push("");
  lines.push(
    "Every row is demand-gated. The gate is the switch that keeps the product off the hot path until a consumer demands it.",
  );
  lines.push("");
  for (const product of orphanProducts(productIds, resolvedBy)) {
    lines.push(`- \`${product.id}\` — gate: \`${gateOf(product.id)}\``);
  }
  lines.push("");
  lines.push("## Non-product `vize_croquis` imports observed");
  lines.push("");
  const nonProductBy = rowsBy(analysis.nonProduct);
  const itemCells = [...nonProductBy.keys()]
    .sort(byKey)
    .map((id) => ({ id, cells: [`\`${id}\``] }));
  lines.push(totalsTable(["item"], itemCells, nonProductBy));
  lines.push("");
  lines.push("## Cross-check: symbol-resolved vs naive grep");
  lines.push("");
  const crossRows = [];
  for (const p of productIds) {
    const perCrate = crates
      .map((crate) => ({ crate, d: disagreements([p], byCrate.get(crate))[0] }))
      .filter((x) => x.d);
    if (perCrate.length === 0) continue;
    let resolved = 0;
    let grep = 0;
    for (const group of byCrate.values()) {
      resolved += group.resolved.get(p.id)?.sites ?? 0;
      grep += group.grep.get(p.id) ?? 0;
    }
    const detail = perCrate.map((x) => `\`${x.crate}\` (${x.d.resolved}/${x.d.grep})`).join(", ");
    crossRows.push([`\`${p.id}\``, String(resolved), String(grep), detail]);
  }
  lines.push(
    formatTable(
      ["product", "resolved", "grep", "disagreeing crates (resolved/grep)"],
      ["left", "right", "right", "left"],
      crossRows,
    ).trimEnd(),
  );
  lines.push("");
  return lines.join("\n");
}

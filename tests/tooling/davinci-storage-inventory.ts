export type VecCategory = "contract" | "analysis" | "lower" | "pass" | "emit";
export type VecMeasurement = { directPaths: number; boundUses: number };
export type VecPolicy = VecMeasurement & { category: VecCategory };
export type CategorySummary = VecMeasurement & { files: number };
export type InventorySummary = VecMeasurement & {
  files: number;
  categories: Record<VecCategory, CategorySummary>;
};

export const categoryReasons: Record<VecCategory, string> = {
  contract: "variable-length owned Folio/S2 contract data",
  analysis: "unbounded diagnostics, lookup storage, and traversal results",
  lower: "source-sized lowering worklists and owned results",
  pass: "source-sized pass facts, provenance, and traversal worklists",
  emit: "ordered emitter buffers whose size follows the document",
};

function policy(category: VecCategory, directPaths = 1, boundUses = 0): VecPolicy {
  return { category, directPaths, boundUses };
}

// Exact reviewed state. Reductions must update this ledger so they cannot regress later.
export const retainedAllocVec = new Map<string, VecPolicy>([
  ["crates/vize_davinci/src/diagnostic.rs", policy("analysis", 1, 2)],
  ["crates/vize_davinci/src/folio/croquis.rs", policy("contract", 1, 16)],
  ["crates/vize_davinci/src/folio/croquis/parse.rs", policy("contract", 1, 3)],
  ["crates/vize_davinci/src/folio/croquis/parse/entry.rs", policy("contract", 1, 4)],
  ["crates/vize_davinci/src/folio/croquis/print.rs", policy("contract", 1, 2)],
  ["crates/vize_davinci/src/folio/dump.rs", policy("contract", 1, 2)],
  ["crates/vize_davinci/src/folio/feed.rs", policy("contract", 1, 2)],
  ["crates/vize_davinci/src/folio/page.rs", policy("contract", 1, 2)],
  ["crates/vize_davinci/src/pass/pipeline.rs", policy("analysis", 1, 4)],
  ["crates/vize_davinci/src/side_table.rs", policy("analysis", 2, 5)],
  ["crates/vize_disegno/src/expr/filter.rs", policy("analysis", 2)],
  ["crates/vize_disegno/src/folio.rs", policy("contract", 1, 1)],
  ["crates/vize_disegno/src/folio/owned.rs", policy("contract", 1, 13)],
  ["crates/vize_disegno/src/folio/owned/binding.rs", policy("contract", 1, 7)],
  ["crates/vize_disegno/src/folio/parse.rs", policy("contract", 1, 4)],
  ["crates/vize_disegno/src/folio/parse/binding_line.rs", policy("contract", 1, 8)],
  ["crates/vize_disegno/src/folio/parse/line.rs", policy("contract", 12)],
  ["crates/vize_disegno/src/scope.rs", policy("analysis", 1, 1)],
  ["crates/vize_disegno/src/verify.rs", policy("analysis", 1, 4)],
  ["crates/vize_disegno/src/verify/walk.rs", policy("analysis", 1, 5)],
  ["crates/vize_ricalco/src/emit.rs", policy("emit", 1, 2)],
  ["crates/vize_ricalco/src/emit/buf.rs", policy("emit", 1, 8)],
  ["crates/vize_ricalco/src/emit/component.rs", policy("emit", 1, 4)],
  ["crates/vize_ricalco/src/emit/create_slots.rs", policy("emit", 1, 5)],
  ["crates/vize_ricalco/src/emit/directive.rs", policy("emit", 1, 5)],
  ["crates/vize_ricalco/src/emit/hoist.rs", policy("emit", 1, 4)],
  ["crates/vize_ricalco/src/emit/merge.rs", policy("emit", 1, 8)],
  ["crates/vize_ricalco/src/emit/model.rs", policy("emit", 1, 6)],
  ["crates/vize_ricalco/src/emit/on.rs", policy("emit", 1, 6)],
  ["crates/vize_ricalco/src/emit/props.rs", policy("emit", 1, 2)],
  ["crates/vize_ricalco/src/emit/props_object.rs", policy("emit", 1, 5)],
  ["crates/vize_ricalco/src/emit/slots.rs", policy("emit", 1, 5)],
  ["crates/vize_ricalco/src/lower.rs", policy("lower", 1, 2)],
  ["crates/vize_ricalco/src/lower/binding.rs", policy("lower", 1, 1)],
  ["crates/vize_ricalco/src/lower/cx.rs", policy("lower", 1, 4)],
  ["crates/vize_ricalco/src/lower/directive.rs", policy("lower", 1, 6)],
  ["crates/vize_ricalco/src/lower/element.rs", policy("lower", 1, 2)],
  ["crates/vize_ricalco/src/lower/forop.rs", policy("lower", 1, 2)],
  ["crates/vize_ricalco/src/lower/structural.rs", policy("lower", 1, 10)],
  ["crates/vize_ricalco/src/lower/structural/wrapper.rs", policy("lower", 1, 2)],
  ["crates/vize_ricalco/src/lower/sugar.rs", policy("lower", 1, 1)],
  ["crates/vize_ricalco/src/lower/text.rs", policy("lower", 1, 4)],
  ["crates/vize_ricalco/src/lower/text/condense.rs", policy("lower", 1, 5)],
  ["crates/vize_ricalco/src/lower/vfor.rs", policy("lower", 5)],
  ["crates/vize_ricalco/src/pass/hoist/lattice.rs", policy("pass", 1, 2)],
  ["crates/vize_ricalco/src/pass/legacy/ids.rs", policy("pass", 1, 5)],
  ["crates/vize_ricalco/src/pass/text.rs", policy("pass", 1, 3)],
  ["crates/vize_ricalco/src/pass/vfor.rs", policy("pass", 1, 5)],
  ["crates/vize_ricalco/src/pass/vif.rs", policy("pass", 1, 5)],
  ["crates/vize_ricalco/src/pass/vmodel.rs", policy("pass", 1, 9)],
  ["crates/vize_ricalco/src/pass/vmodel/check.rs", policy("pass", 1, 1)],
  ["crates/vize_ricalco/src/pass/vslot.rs", policy("pass", 1, 2)],
  ["crates/vize_ricalco/src/pass/vslot/consume.rs", policy("pass", 1, 8)],
  ["crates/vize_ricalco/src/pass/vslot/group.rs", policy("pass", 1, 6)],
  ["crates/vize_ricalco/src/pass/vslot/spell.rs", policy("pass", 1, 1)],
]);

export const expectedInventorySummary: InventorySummary = {
  files: 55,
  directPaths: 72,
  boundUses: 231,
  categories: {
    contract: { files: 13, directPaths: 24, boundUses: 64 },
    analysis: { files: 7, directPaths: 9, boundUses: 21 },
    lower: { files: 12, directPaths: 16, boundUses: 39 },
    pass: { files: 11, directPaths: 11, boundUses: 47 },
    emit: { files: 12, directPaths: 12, boundUses: 60 },
  },
};

export function summarizeInventory(
  measured: ReadonlyMap<string, VecMeasurement>,
): InventorySummary {
  const categories: Record<VecCategory, CategorySummary> = {
    contract: { files: 0, directPaths: 0, boundUses: 0 },
    analysis: { files: 0, directPaths: 0, boundUses: 0 },
    lower: { files: 0, directPaths: 0, boundUses: 0 },
    pass: { files: 0, directPaths: 0, boundUses: 0 },
    emit: { files: 0, directPaths: 0, boundUses: 0 },
  };
  const summary: InventorySummary = {
    files: measured.size,
    directPaths: 0,
    boundUses: 0,
    categories,
  };
  for (const [relative, actual] of measured) {
    const entry = retainedAllocVec.get(relative);
    if (!entry) throw new Error(`unclassified alloc Vec file: ${relative}`);
    summary.directPaths += actual.directPaths;
    summary.boundUses += actual.boundUses;
    const category = categories[entry.category];
    category.files += 1;
    category.directPaths += actual.directPaths;
    category.boundUses += actual.boundUses;
  }
  return summary;
}

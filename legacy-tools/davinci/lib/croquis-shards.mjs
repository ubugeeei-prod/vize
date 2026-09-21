// Committed form of the croquis consumption matrix: an index that depends only
// on `vize_croquis` itself (method + product set) and one shard per consuming
// crate holding every per-crate fact. Nothing committed aggregates across
// crates, so two PRs touching different crates never edit the same file; the
// cross-crate totals live in the on-demand `--summary` view instead.

import { formatTable } from "./markdown.mjs";
import { byKey } from "./ordering.mjs";
import {
  SUMMARY_COMMAND,
  disagreements,
  groupByCrate,
  methodLines,
  productIdsOf,
} from "./croquis-render.mjs";
import { ARTIFACT_REL, REGEN_COMMAND, SHARD_DIR_REL } from "./paths.mjs";

const CHECK_COMMAND = "rust-script tools/commands/davinci/croquis-consumers.rs --check";

function header(extra = []) {
  const lines = [
    "<!-- GENERATED FILE — do not edit by hand.",
    `     Regenerate: ${REGEN_COMMAND}`,
    `     Verify:     ${CHECK_COMMAND}`,
    ...extra,
  ];
  lines[lines.length - 1] += " -->";
  return lines;
}

function tableOrNone(headers, aligns, rows) {
  return rows.length === 0 ? "_None._" : formatTable(headers, aligns, rows).trimEnd();
}

function renderIndex(products, analysis, productIds) {
  const lines = header([
    `     Totals:     ${SUMMARY_COMMAND}`,
    "     Generator:  legacy-tools/davinci/croquis-consumers.mjs",
  ]);
  lines.push("");
  lines.push("# Croquis consumption matrix");
  lines.push("");
  lines.push(
    "Which workspace crates consume the public analysis products of" +
      " `crates/vize_croquis`. Mechanizes the 2026-08-13 hand audit in" +
      " [semantic-engine.md](../semantic-engine.md#the-problem-measured) (Davinci P0-7).",
  );
  lines.push("");
  lines.push("## Layout");
  lines.push("");
  lines.push(
    "This page holds only what depends on `vize_croquis` itself: the resolution" +
      " method and the product set. Consumption facts are sharded one file per" +
      ` consuming crate, \`${SHARD_DIR_REL}/<crate>.md\`: that crate's resolved` +
      " product sites, its non-product `vize_croquis` imports, and its naive-grep" +
      " disagreements. A crate with none of those has no shard.",
  );
  lines.push("");
  lines.push(
    "Cross-crate aggregates — per-product totals, the products with no external" +
      " consumer, and the resolved/grep totals — are deliberately **not committed**:" +
      " they changed with every PR and made every open PR conflict. They are pure" +
      ` sums over the shards; print them with \`${SUMMARY_COMMAND}\`.` +
      " The staleness check (TS-12) byte-compares this page, every shard, and the" +
      " shard set itself (a leftover shard is stale).",
  );
  lines.push("");
  lines.push(...methodLines(products, analysis));
  lines.push("");
  lines.push("## Product set");
  lines.push("");
  lines.push(
    tableOrNone(
      ["product", "kind", "module"],
      ["left", "left", "left"],
      productIds.map((p) => [`\`${p.id}\``, p.kind, `\`${p.module}\``]),
    ),
  );
  lines.push("");
  return lines.join("\n");
}

function renderShard(crate, group, productIds) {
  const lines = header();
  lines.push("");
  lines.push(`# Croquis consumption: \`${crate}\``);
  lines.push("");
  lines.push(
    `One shard of the [Croquis consumption matrix](../${ARTIFACT_REL.split("/").pop()}):` +
      ` every fact the matrix records about \`crates/${crate}/src\`. The method and` +
      " the product set live on that page.",
  );
  lines.push("");
  lines.push("## Resolved product sites");
  lines.push("");
  lines.push(
    tableOrNone(
      ["product", "kind", "module", "files", "sites"],
      ["left", "left", "left", "right", "right"],
      productIds
        .filter((p) => group.resolved.has(p.id))
        .map((p) => {
          const row = group.resolved.get(p.id);
          return [
            `\`${p.id}\``,
            p.kind,
            `\`${p.module}\``,
            String(row.files.size),
            String(row.sites),
          ];
        }),
    ),
  );
  lines.push("");
  lines.push("## Non-product `vize_croquis` imports");
  lines.push("");
  lines.push(
    tableOrNone(
      ["item", "files", "sites"],
      ["left", "right", "right"],
      [...group.nonProduct.keys()].sort(byKey).map((item) => {
        const row = group.nonProduct.get(item);
        return [`\`${item}\``, String(row.files.size), String(row.sites)];
      }),
    ),
  );
  lines.push("");
  lines.push("## Naive grep disagreements (resolved/grep)");
  lines.push("");
  lines.push(
    tableOrNone(
      ["product", "resolved", "grep"],
      ["left", "right", "right"],
      disagreements(productIds, group).map((d) => [
        `\`${d.id}\``,
        String(d.resolved),
        String(d.grep),
      ]),
    ),
  );
  lines.push("");
  return lines.join("\n");
}

/** Every committed file of the matrix: the index plus one shard per consuming crate. */
export function renderCroquisArtifacts(products, analysis) {
  const productIds = productIdsOf(products);
  const files = [{ relPath: ARTIFACT_REL, text: renderIndex(products, analysis, productIds) }];
  const byCrate = groupByCrate(analysis);
  for (const crate of [...byCrate.keys()].sort(byKey)) {
    const group = byCrate.get(crate);
    const hasFacts =
      group.resolved.size > 0 ||
      group.nonProduct.size > 0 ||
      disagreements(productIds, group).length > 0;
    if (!hasFacts) continue;
    files.push({
      relPath: `${SHARD_DIR_REL}/${crate}.md`,
      text: renderShard(crate, group, productIds),
    });
  }
  return files;
}

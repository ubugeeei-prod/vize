import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import {
  gateViolations,
  renderSummary,
} from "../../tools/support/compat/davinci/lib/croquis-render.mjs";
import { renderCroquisArtifacts } from "../../tools/support/compat/davinci/lib/croquis-shards.mjs";

const products = {
  typeProducts: new Map([["Span", { module: "scope" }]]),
  fieldProducts: new Map(),
  passthroughs: [],
};

function analysis(grep, resolved = 0, nonProduct = 0) {
  const row = (product, sites) => ({
    crate: "vize_l2",
    product,
    sites,
    files: new Set(["davinci/vize_l2/src/file.rs"]),
  });
  return {
    rows: new Map(resolved ? [["resolved", row("Span", resolved)]] : []),
    nonProduct: new Map(nonProduct ? [["non-product", row("Other", nonProduct)]] : []),
    grepRows: new Map([["grep", row("Span", grep)]]),
    globFiles: [],
  };
}

void test("unrelated native grep additions retain exact committed consumption and fresh diagnostics", () => {
  const before = analysis(2, 1);
  const after = analysis(3, 1);
  assert.deepEqual(
    renderCroquisArtifacts(products, after),
    renderCroquisArtifacts(products, before),
  );
  assert.match(renderSummary(products, before), /`vize_l2` \(1\/2\)/u);
  assert.match(renderSummary(products, after), /`vize_l2` \(1\/3\)/u);
});

void test("grep-only native crates remain visible without generating a conflicting committed count", () => {
  const result = analysis(4);
  assert.equal(renderCroquisArtifacts(products, result).length, 1);
  assert.match(renderSummary(products, result), /`vize_l2` \(0\/4\)/u);
  const ungated = {
    ...products,
    typeProducts: new Map([["UngatedNewProduct", { module: "scope" }]]),
  };
  result.grepRows.get("grep").product = "UngatedNewProduct";
  assert.equal(gateViolations(ungated, result).missing.includes("UngatedNewProduct"), true);
});

void test("resolved consumption and non-product changes still change byte-compared artifacts", () => {
  const original = renderCroquisArtifacts(products, analysis(4, 1, 1));
  assert.notDeepEqual(renderCroquisArtifacts(products, analysis(4, 2, 1)), original);
  assert.notDeepEqual(renderCroquisArtifacts(products, analysis(4, 1, 2)), original);
});

void test("the actual check producer prints source-qualified per-crate diagnostics", () => {
  const generator = fileURLToPath(
    new URL("../../tools/support/compat/davinci/croquis-consumers.mjs", import.meta.url),
  );
  const result = spawnSync(process.execPath, [generator, "--check"], { encoding: "utf8" });
  assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
  assert.match(result.stdout, /^Analyzed working tree at HEAD `[0-9a-f]{40}`\./u);
  assert.match(result.stdout, /## Cross-check: symbol-resolved vs naive grep/u);
  assert.match(result.stdout, /`vize_l2` \(\d+\/\d+\)/u);
  assert.match(result.stdout, /croquis consumption matrix is up to date/u);
});

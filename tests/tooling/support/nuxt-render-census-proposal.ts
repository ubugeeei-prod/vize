import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { analyzeConsumers } from "../../../tools/support/compat/davinci/lib/croquis-analysis.ts";
import { enumerateProducts } from "../../../tools/support/compat/davinci/lib/croquis-products.ts";
import { gateViolations } from "../../../tools/support/compat/davinci/lib/croquis-render.ts";
import { renderCroquisArtifacts } from "../../../tools/support/compat/davinci/lib/croquis-shards.ts";
import { ARTIFACT_REL, repoRoot } from "../../../tools/support/compat/davinci/lib/paths.ts";

const output = process.argv[2];
assert(output, "Provide an output file outside the checkout");
function hash(value: string | Uint8Array): string {
  return createHash("sha256").update(value).digest("hex");
}
function git(...args: string[]): string {
  return execFileSync("git", args, { cwd: repoRoot, encoding: "utf8" }).trim();
}
const revision = git("rev-parse", "HEAD");
assert.equal(revision, process.env.GITHUB_SHA);
assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
const products = enumerateProducts();
const analysis = analyzeConsumers(products);
const artifacts = renderCroquisArtifacts(products, analysis);
const index = artifacts.find((file) => file.relPath === ARTIFACT_REL);
assert(index);
const violations = gateViolations(products, analysis);
const generators = [
  "croquis-consumers.ts",
  "lib/croquis-products.ts",
  "lib/croquis-analysis.ts",
  "lib/croquis-gates.ts",
  "lib/croquis-render.ts",
  "lib/croquis-shards.ts",
].map((name) => {
  const relative = "tools/support/compat/davinci/" + name;
  return { path: relative, sha256: hash(fs.readFileSync(path.join(repoRoot, relative))) };
});
const report = {
  schema: "vize.nuxt-render.source-census-proposal.v1",
  role: "source-qualified generated index proposal; no checkout rewrite or runtime qualification",
  qualified: false,
  source: {
    revision,
    tree: git("rev-parse", "HEAD^{tree}"),
    authoredHead: process.env.VIZE_NUXT_SCOPE_AUTHORED_HEAD,
  },
  generators,
  products: {
    types: [...products.typeProducts.keys()].sort(),
    fields: [...products.fieldProducts.keys()].sort(),
  },
  violations,
  consumers: [...analysis.rows.values()]
    .filter((row) => row.product === "Croquis.component_usages")
    .map((row) => ({
      product: row.product,
      crate: row.crate,
      sites: row.sites,
      files: [...row.files].sort(),
    }))
    .sort((left, right) => left.crate.localeCompare(right.crate)),
  previous: {
    path: ARTIFACT_REL,
    blob: git("rev-parse", "HEAD:" + ARTIFACT_REL),
    sha256: hash(fs.readFileSync(path.join(repoRoot, ARTIFACT_REL))),
  },
  proposed: { path: index.relPath, sha256: hash(index.text), text: index.text },
};
assert(!path.resolve(output).startsWith(repoRoot + path.sep));
fs.mkdirSync(path.dirname(output), { recursive: true });
fs.writeFileSync(output, JSON.stringify(report, null, 2) + "\n");
assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
assert.deepEqual(violations, { missing: [], stale: [] });
console.log(
  JSON.stringify({
    source: revision,
    path: ARTIFACT_REL,
    sha256: report.proposed.sha256,
    violations,
  }),
);

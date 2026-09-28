import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { appendFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { changedPaths } from "./plan-source-checks.mjs";

const manifestPath = (product) => `tests/_fixtures/differential/${product}/manifest.json`;

export function legacyProducts(paths) {
  const products = new Set();
  for (const path of paths) {
    if (!/^crates\/[^/]+\/src\//.test(path)) continue;
    if (path.startsWith("crates/vize_glyph/")) products.add("formatter");
    else if (path.startsWith("crates/vize_patina/")) products.add("linter");
    else if (path.startsWith("crates/vize_canon/")) products.add("typechecker");
    else if (/^crates\/vize_(maestro|resident)\//.test(path)) products.add("lsp");
    else if (/^crates\/vize_(armature|relief|atelier_[^/]+)\//.test(path)) {
      products.add("compiler");
    }
  }
  return [...products].sort((left, right) => left.localeCompare(right));
}

export function analyzeLegacyFix({ title, paths, readFile }) {
  if (!/^fix(?:\([^)]+\))?!?:\s\S/.test(title)) {
    return { state: "not-a-fix-pr", products: [] };
  }
  const products = legacyProducts(paths);
  if (products.length === 0) return { state: "no-known-legacy-source", products: [] };
  return {
    state: "report-only",
    products: products.map((product) => {
      const file = manifestPath(product);
      const previous = readFile("base", file);
      const current = readFile("head", file);
      if (current === null) return { product, state: "adapter-or-manifest-missing", cases: [] };
      try {
        const manifest = JSON.parse(current.toString("utf8"));
        const before = previous === null ? null : JSON.parse(previous.toString("utf8"));
        if (
          manifest.schema !== "vize.differential.manifest" ||
          manifest.version !== 1 ||
          manifest.product !== product ||
          !Array.isArray(manifest.cases) ||
          (before !== null &&
            (before.schema !== manifest.schema ||
              before.version !== manifest.version ||
              !Array.isArray(before.cases) ||
              before.product !== product))
        ) {
          throw new Error("invalid product manifest envelope");
        }
        const allIds = manifest.cases.map((entry) => entry.id);
        if (new Set(allIds).size !== allIds.length) throw new Error("duplicate case ID");
        const oldIds = new Set(before?.cases.map((entry) => entry.id) ?? []);
        const added = manifest.cases.filter((entry) => !oldIds.has(entry.id));
        const cases = added.map((entry) => {
          if (
            entry.state !== "active" ||
            !entry.id?.startsWith(`${product}/`) ||
            !/^[-a-z0-9/]+$/.test(entry.id)
          ) {
            throw new Error(`invalid active case: ${entry.id}`);
          }
          if (
            !Array.isArray(entry.targets) ||
            entry.targets.length === 0 ||
            new Set(entry.targets).size !== entry.targets.length ||
            entry.targets.some((target) => !/^[a-z][a-z0-9_-]*$/.test(target))
          ) {
            throw new Error(`invalid case targets: ${entry.id}`);
          }
          const files = entry.inputs?.files;
          if (!Array.isArray(files) || files.length === 0 || !entry.inputs?.root) {
            throw new Error(`missing input files: ${entry.id}`);
          }
          for (const input of files) {
            if (
              !/^[a-zA-Z0-9_.-]+$/.test(input.path ?? "") ||
              !/^[a-zA-Z0-9_.-]+$/.test(entry.inputs.root) ||
              !/^[a-f0-9]{64}$/.test(input.sha256 ?? "")
            ) {
              throw new Error(`invalid pinned input: ${entry.id}`);
            }
            const path = `tests/_fixtures/differential/${product}/${entry.inputs.root}/${input.path}`;
            const bytes = readFile("head", path);
            if (
              bytes === null ||
              createHash("sha256").update(bytes).digest("hex") !== input.sha256
            ) {
              throw new Error(`missing or mismatched pinned input: ${path}`);
            }
          }
          return entry.id;
        });
        return {
          product,
          state: cases.length ? "new-inputs-registered" : "new-input-missing",
          cases,
        };
      } catch (error) {
        return { product, state: "invalid-manifest-or-input", cases: [], reason: String(error) };
      }
    }),
  };
}

export function formatSummary(report) {
  const lines = [
    "### Legacy fix differential input audit (#6852)",
    "",
    "Report only. Product adapters and native comparison are incomplete; this audit grants no parity or native acceptance.",
    "",
  ];
  if (report.products.length === 0) {
    lines.push(`Result: ${report.state}.`);
  } else {
    lines.push("| Product | Registered new input | Case IDs |", "| --- | --- | --- |");
    for (const product of report.products) {
      lines.push(`| ${product.product} | ${product.state} | ${product.cases.join(", ") || "—"} |`);
      if (product.reason) lines.push(`\n${product.product}: ${product.reason}`);
    }
  }
  lines.push("");
  return `${lines.join("\n")}\n`;
}

function gitFile(revision, path) {
  const result = spawnSync("git", ["show", `${revision}:${path}`], { encoding: "buffer" });
  if (result.status === 0) return result.stdout;
  if (result.stderr.toString().includes("does not exist")) return null;
  throw new Error(`git show failed for ${revision}:${path}: ${result.stderr.toString()}`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [base, head] = process.argv.slice(2);
  if (![base, head].every((sha) => /^[a-f0-9]{40}$/.test(sha ?? ""))) {
    throw new Error("expected full comparison base and head SHAs");
  }
  for (const revision of [base, head]) {
    execFileSync("git", ["cat-file", "-e", `${revision}^{commit}`]);
  }
  const paths = changedPaths(base, head);
  const report = analyzeLegacyFix({
    title: process.env.PR_TITLE ?? "",
    paths,
    readFile: (side, path) => gitFile(side === "base" ? base : head, path),
  });
  const summary = formatSummary(report);
  process.stdout.write(summary);
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, summary);
}

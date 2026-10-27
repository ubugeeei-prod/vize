// #6852: a `fix(...)` PR that changes legacy product source must register a
// new pinned differential input for each affected product. Native parity is not
// judged here; it stays a report until native product adapters exist.
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { appendFileSync } from "node:fs";

import { changedPaths } from "./plan-source-checks.ts";
import { newLanguageProjectCases } from "./language-project-fixtures.ts";
import {
  compilerCssApiSources,
  newCompilerCssApiCases,
  onlyCompilerCssChanges,
} from "../../../../tests/differential/compiler-css-api.ts";
import {
  nativeShutdownProtocolSource,
  newNativeShutdownProtocolCases,
} from "./native-shutdown-protocol-fixtures.ts";

import { knownCompilerCssSource } from "../../../../tests/differential/compiler-css-api-registry.ts";
import { newNativeRuntimeCases } from "./native-runtime-fixtures.ts";
import { newVaporStyleApiCases } from "./vapor-style-api-fixtures.ts";

/** PRs opened before this (just after the gate merged) stay report-only. */
export const GATE_SINCE = "2026-10-20T06:00:00Z";

export type Product = "compiler" | "formatter" | "linter" | "lsp" | "typechecker";
type Side = "base" | "head";
type ReadFile = (side: Side, path: string) => Buffer | null;

export type ProductState =
  | "adapter-or-manifest-missing"
  | "invalid-manifest-or-input"
  | "new-input-missing"
  | "new-inputs-registered";

export interface ProductReport {
  product: Product;
  state: ProductState;
  cases: string[];
  reason?: string;
}

export interface LegacyFixReport {
  state: "not-a-fix-pr" | "no-known-legacy-source" | "fixture-required";
  products: ProductReport[];
}

interface ManifestCase {
  id?: string;
  state?: string;
  targets?: unknown;
  inputs?: { root?: string; files?: PinnedInput[] } | PinnedInput[];
  adapters?: { legacy?: unknown; native?: unknown; reasons?: { native?: unknown } };
  comparison?: { contract?: unknown; requiredFacets?: unknown };
  reference?: { path?: string; sha256?: string; source?: unknown };
  dialectCoverage?: DialectCoverage;
}

interface DialectCoverage {
  state?: string;
  evidence?: { dialects?: unknown }[];
}

interface PinnedInput {
  path?: string;
  role?: string;
  sha256?: string;
}

interface Manifest {
  schema?: unknown;
  version?: unknown;
  product?: unknown;
  cases?: ManifestCase[];
  protocolCases?: unknown;
  languageProjectCases?: unknown;
  runtimeCases?: unknown;
  cssApiCases?: unknown;
  vaporStyleApiCases?: unknown;
}

const manifestPath = (product: Product) => `tests/_fixtures/differential/${product}/manifest.json`;
const COMPILER_FACETS = ["code", "css", "errors", "warnings", "macro_artifacts"];

export function legacyProducts(paths: readonly string[]): Product[] {
  const products = new Set<Product>();
  for (const path of paths) {
    if (knownCompilerCssSource(path)) {
      products.add("compiler");
      continue;
    }
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

const COVERAGE_RANK: Record<string, number> = { partial: 1, "source-inspected": 1, verified: 2 };

// Lower readiness, or fewer dialects at the same readiness.
function coverageLost(before?: DialectCoverage, after?: DialectCoverage): boolean {
  const rank = (coverage?: DialectCoverage) => COVERAGE_RANK[coverage?.state ?? ""] ?? 0;
  const dialects = (coverage?: DialectCoverage) =>
    new Set(
      (coverage?.evidence ?? []).flatMap((item) =>
        Array.isArray(item.dialects) ? (item.dialects as unknown[]) : [],
      ),
    );
  if (rank(before) !== rank(after)) return rank(after) < rank(before);
  const kept = dialects(after);
  return [...dialects(before)].some((dialect) => !kept.has(dialect));
}

function sha256(bytes: Buffer): string {
  return createHash("sha256").update(bytes).digest("hex");
}

function requirePinned(readFile: ReadFile, path: string, expected: string, label: string): void {
  const bytes = readFile("head", path);
  if (bytes === null || sha256(bytes) !== expected) {
    throw new Error(`missing or mismatched pinned ${label}: ${path}`);
  }
}

// Compiler cases compare whole SSR modules, so they also pin the expected module.
function checkCompilerCase(
  entry: ManifestCase,
  root: string,
  files: PinnedInput[],
  readFile: ReadFile,
): void {
  const reference = entry.reference;
  const expectedPath = `${root}/module.expected.txt`;
  if (
    entry.id !== `compiler/sfc/${root}` ||
    JSON.stringify(entry.targets) !== JSON.stringify(["ssr"]) ||
    files.length !== 1 ||
    files[0]?.path !== "Layout.vue.txt" ||
    files[0]?.role !== "entry" ||
    entry.adapters?.legacy !== "compiler-cli-ssr-v1" ||
    entry.adapters?.native !== null ||
    typeof entry.adapters?.reasons?.native !== "string" ||
    entry.comparison?.contract !== "compiler-ssr-module-bytes-v1" ||
    JSON.stringify(entry.comparison?.requiredFacets) !== JSON.stringify(COMPILER_FACETS) ||
    reference?.path !== expectedPath ||
    !/^[a-f0-9]{64}$/.test(reference.sha256 ?? "") ||
    typeof reference.source !== "string" ||
    reference.source.length === 0
  ) {
    throw new Error(`invalid pinned compiler reference: ${entry.id}`);
  }
  requirePinned(
    readFile,
    `tests/_fixtures/differential/compiler/${expectedPath}`,
    reference.sha256 ?? "",
    "compiler reference",
  );
}

function checkProduct(
  product: Product,
  paths: readonly string[],
  readFile: ReadFile,
): ProductReport {
  const previous = readFile("base", manifestPath(product));
  const current = readFile("head", manifestPath(product));
  if (current === null) return { product, state: "adapter-or-manifest-missing", cases: [] };
  try {
    const manifest = JSON.parse(current.toString("utf8")) as Manifest;
    const before = previous === null ? null : (JSON.parse(previous.toString("utf8")) as Manifest);
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
    const oldIds = new Set(before?.cases?.map((entry) => entry.id) ?? []);
    // Unit tests only keep baseline floors, so a fix must not drop or
    // downgrade a case that is already planned.
    const planned = new Map(manifest.cases.map((entry) => [entry.id, entry]));
    for (const entry of before?.cases ?? []) {
      const kept = planned.get(entry.id);
      if (kept === undefined) throw new Error(`drops planned case: ${entry.id}`);
      if (coverageLost(entry.dialectCoverage, kept.dialectCoverage)) {
        throw new Error(`downgrades dialect coverage: ${entry.id}`);
      }
    }
    const cases = manifest.cases
      .filter((entry) => !oldIds.has(entry.id))
      .map((entry) => {
        const id = entry.id ?? "";
        if (
          entry.state !== "active" ||
          !id.startsWith(`${product}/`) ||
          !/^[-a-z0-9/]+$/.test(id)
        ) {
          throw new Error(`invalid active case: ${id}`);
        }
        const targets = entry.targets;
        if (
          !Array.isArray(targets) ||
          targets.length === 0 ||
          new Set(targets).size !== targets.length ||
          targets.some((target) => typeof target !== "string" || !/^[a-z][a-z0-9_-]*$/.test(target))
        ) {
          throw new Error(`invalid case targets: ${id}`);
        }
        const legacyInputs = entry.inputs;
        const files = Array.isArray(legacyInputs) ? legacyInputs : legacyInputs?.files;
        const root = Array.isArray(legacyInputs) ? undefined : legacyInputs?.root;
        if (!Array.isArray(files) || files.length === 0) {
          throw new Error(`missing input files: ${id}`);
        }
        for (const input of files) {
          const relativePath = root ? `${root}/${input.path}` : input.path;
          if (
            !/^(?:[a-zA-Z0-9_.-]+\/)*[a-zA-Z0-9_.-]+$/.test(relativePath ?? "") ||
            (root !== undefined && !/^[a-zA-Z0-9_.-]+$/.test(root)) ||
            !/^[a-f0-9]{64}$/.test(input.sha256 ?? "")
          ) {
            throw new Error(`invalid pinned input: ${id}`);
          }
          const path = `tests/_fixtures/differential/${product}/${relativePath}`;
          requirePinned(readFile, path, input.sha256 ?? "", "input");
        }
        if (product === "compiler") {
          if (root === undefined) throw new Error(`missing compiler input root: ${id}`);
          checkCompilerCase(entry, root, files, readFile);
        }
        return id;
      });
    const canonPaths = paths.filter((path) => path.startsWith("crates/vize_canon/src/"));
    const protocolRequired =
      product === "typechecker" && canonPaths.some(nativeShutdownProtocolSource);
    const languageProject =
      product === "typechecker"
        ? newLanguageProjectCases(
            before?.languageProjectCases,
            manifest.languageProjectCases,
            readFile,
          )
        : { ids: [] as string[], sources: new Set<string>() };
    const runtime = newNativeRuntimeCases(
      product,
      before?.runtimeCases,
      manifest.runtimeCases,
      readFile,
    );
    const productPaths = paths.filter((path) =>
      path.startsWith(product === "linter" ? "crates/vize_patina/src/" : "crates/vize_canon/src/"),
    );
    const batchRequired =
      product === "linter" || product === "typechecker"
        ? productPaths.some(
            (path) =>
              !runtime.sources.has(path) &&
              !languageProject.sources.has(path) &&
              !(product === "typechecker" && nativeShutdownProtocolSource(path)),
          )
        : true;
    const protocols =
      product === "typechecker"
        ? newNativeShutdownProtocolCases(before?.protocolCases, manifest.protocolCases, readFile)
        : [];
    const cssApi =
      product === "compiler"
        ? newCompilerCssApiCases(before?.cssApiCases, manifest.cssApiCases, readFile)
        : [];
    const cssOnly =
      product === "compiler" &&
      cssApi.length > 0 &&
      onlyCompilerCssChanges(paths, compilerCssApiSources(cssApi));
    const vaporStyle =
      product === "compiler"
        ? newVaporStyleApiCases(before?.vaporStyleApiCases, manifest.vaporStyleApiCases, readFile)
        : { ids: [] as string[], sources: new Set<string>() };
    const compilerPaths = paths.filter((path) => legacyProducts([path]).includes("compiler"));
    const vaporIds = compilerPaths.some((path) => vaporStyle.sources.has(path))
      ? vaporStyle.ids
      : [];
    const vaporOnly =
      product === "compiler" &&
      vaporIds.length > 0 &&
      compilerPaths.length > 0 &&
      compilerPaths.every((path) => vaporStyle.sources.has(path));
    const registered =
      (cssOnly || vaporOnly || !batchRequired || cases.length > 0) &&
      (!protocolRequired || protocols.length > 0);
    return {
      product,
      state: registered ? "new-inputs-registered" : "new-input-missing",
      cases: [
        ...cases,
        ...protocols,
        ...runtime.ids,
        ...cssApi,
        ...vaporIds,
        ...languageProject.ids,
      ],
    };
  } catch (error) {
    return { product, state: "invalid-manifest-or-input", cases: [], reason: String(error) };
  }
}

export function analyzeLegacyFix({
  title,
  paths,
  readFile,
}: {
  title: string;
  paths: readonly string[];
  readFile: ReadFile;
}): LegacyFixReport {
  if (!/^fix(?:\([^)]+\))?!?:\s\S/.test(title)) return { state: "not-a-fix-pr", products: [] };
  const products = legacyProducts(paths);
  if (products.length === 0) return { state: "no-known-legacy-source", products: [] };
  return {
    state: "fixture-required",
    products: products.map((product) => checkProduct(product, paths, readFile)),
  };
}

/** Whether the requirement applies to a PR opened at `createdAt`; unknown means yes. */
export function enforced(createdAt: string | undefined): boolean {
  return !createdAt || Date.parse(createdAt) >= Date.parse(GATE_SINCE);
}

export function failingProducts(report: LegacyFixReport): Product[] {
  return report.products
    .filter((product) => product.state !== "new-inputs-registered")
    .map((product) => product.product);
}

export function formatSummary(report: LegacyFixReport, enforce: boolean): string {
  const lines = [
    "### Legacy fix differential inputs (#6852)",
    "",
    enforce
      ? "Required: each legacy product this fix changes must register a new active case with pinned inputs. Native parity is not checked here."
      : `Report only: this PR was opened before the requirement started (${GATE_SINCE}). Native parity is not checked here.`,
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
    if (enforce && failingProducts(report).length > 0) {
      lines.push(
        "",
        "Add the fixed input under `tests/_fixtures/differential/<product>/<case>/` and register it at its sorted position in that product's `manifest.json` with its SHA-256.",
      );
    }
  }
  lines.push("");
  return `${lines.join("\n")}\n`;
}

export function gitFixtureFile(revision: string, path: string, cwd = process.cwd()): Buffer | null {
  const result = spawnSync("git", ["show", `${revision}:${path}`], { cwd, encoding: "buffer" });
  if (result.status === 0) return result.stdout;
  // New files can exist in the checkout while being absent from the baseline.
  // Inspect the actual tree, rather than matching localized Git stderr. An
  // unreadable revision/tree/blob remains an error instead of becoming absence.
  const tree = spawnSync("git", ["ls-tree", "-z", revision, "--", path], {
    cwd,
    encoding: "buffer",
  });
  if (tree.status === 0 && tree.stdout.length === 0) return null;
  throw new Error(`git show failed for ${revision}:${path}: ${result.stderr.toString()}`);
}

if (process.argv[1]?.endsWith("/report-legacy-fix-fixtures.ts")) {
  const [base = "", head = ""] = process.argv.slice(2);
  if (![base, head].every((sha) => /^[a-f0-9]{40}$/.test(sha))) {
    throw new Error("expected full comparison base and head SHAs");
  }
  for (const revision of [base, head]) {
    execFileSync("git", ["cat-file", "-e", `${revision}^{commit}`]);
  }
  const report = analyzeLegacyFix({
    title: process.env.PR_TITLE ?? "",
    paths: changedPaths(base, head),
    readFile: (side, path) => gitFixtureFile(side === "base" ? base : head, path),
  });
  const enforce = enforced(process.env.PR_CREATED_AT);
  const summary = formatSummary(report, enforce);
  process.stdout.write(summary);
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, summary);
  const failing = failingProducts(report);
  if (enforce && failing.length > 0) {
    process.stderr.write(`::error::missing pinned differential input for ${failing.join(", ")}\n`);
    process.exitCode = 1;
  }
}

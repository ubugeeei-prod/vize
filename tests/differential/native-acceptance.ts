// One native-only acceptance census and zero-fallback verdict per product
// (#6853, #6854) over every differential bundle entry: the in-repo fixture
// corpora (T1) and the real-project corpus (T2).
//
// A coordinate is fallback-free only when its product adapter credited it as
// native-handled and its comparator verified equivalence. Everything else is
// counted, never dropped: legacy-backed and unsupported rows are legacy
// fallbacks, unverified rows could not be classified, and native rows without
// verified equivalence are non-equivalent. A policy target or dialect with no
// planned rows is unmeasured, which blocks the verdict like a fallback does.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import { bundleTiers, copyBundle, readBundles, type BundleEntry, type Tier } from "./bundle.ts";
import { sha256 } from "./sha256.ts";

type Json = Record<string, any>;
const countKeys = [
  "planned",
  "nativeHandled",
  "nativeEquivalent",
  "unsupported",
  "legacyBacked",
  "unverified",
] as const;
type Counts = Record<(typeof countKeys)[number], number>;
type ByTier = Record<Tier | "total", Counts>;

const emptyCounts = (): Counts =>
  Object.fromEntries(countKeys.map((key) => [key, 0])) as unknown as Counts;
const emptyByTier = (): ByTier => ({ T1: emptyCounts(), T2: emptyCounts(), total: emptyCounts() });

function add(into: ByTier, tier: Tier, counts: Counts) {
  for (const bucket of [into[tier], into.total]) {
    for (const key of countKeys) bucket[key] += counts[key];
  }
}

export function loadPolicy(file: string) {
  const bytes = fs.readFileSync(file);
  const policy = JSON.parse(bytes.toString("utf8"));
  assert.equal(policy.schema, "vize.deletion-readiness.policy");
  assert.equal(policy.version, 1);
  assert(Array.isArray(policy.dialects) && policy.dialects.length > 0, "policy dialects");
  assert(policy.targets && typeof policy.targets === "object", "policy targets");
  return { policy, sha256: sha256(bytes) };
}

export function buildCensus(entries: BundleEntry[], policy: Json, policySha256: string) {
  const revisions: Partial<Record<Tier, string>> = {};
  const packs: Json[] = [];
  const products: Json = {};
  for (const [product, targets] of Object.entries<string[]>(policy.targets)) {
    products[product] = {
      targets: Object.fromEntries(targets.map((target) => [target, emptyByTier()])),
      dialects: { verified: {}, provisional: {} },
      unknownReadiness: emptyByTier(),
    };
  }
  for (const entry of entries) {
    const context = `${entry.tier}/${entry.product}/${entry.pack}`;
    const known = revisions[entry.tier];
    assert(
      known === undefined || known === entry.sourceRevision,
      `${context}: ${entry.tier} mixes source revisions ${known} and ${entry.sourceRevision}`,
    );
    revisions[entry.tier] = entry.sourceRevision;
    const product = products[entry.product];
    assert(product, `${context}: product is not in the deletion-readiness policy`);
    packs.push({
      tier: entry.tier,
      product: entry.product,
      pack: entry.pack,
      manifestSha256: entry.loaded.manifestSha256,
      scope: entry.acceptance.scope,
      planned: entry.acceptance.total.planned,
    });
    for (const [target, counts] of Object.entries<Counts>(entry.acceptance.targets)) {
      assert(product.targets[target], `${context}: target ${target} is not in the policy`);
      add(product.targets[target], entry.tier, counts);
    }
    add(product.unknownReadiness, entry.tier, entry.acceptance.dialectCoverage.unknown);
    for (const evidence of ["verified", "provisional"]) {
      for (const [dialect, counts] of Object.entries<Counts>(
        entry.acceptance.dialectCoverage[evidence],
      )) {
        product.dialects[evidence][dialect] ??= emptyByTier();
        add(product.dialects[evidence][dialect], entry.tier, counts);
      }
    }
  }
  for (const product of Object.values<Json>(products)) {
    for (const evidence of ["verified", "provisional"]) {
      product.dialects[evidence] = Object.fromEntries(
        Object.entries(product.dialects[evidence]).sort(([a], [b]) => a.localeCompare(b)),
      );
    }
  }
  return {
    schema: "vize.differential.native-acceptance-census",
    version: 1,
    policySha256,
    sourceRevisions: revisions,
    packs,
    products,
  };
}

function fallbackCounts(counts: Counts) {
  return {
    legacyFallbacks: counts.legacyBacked + counts.unsupported,
    unverified: counts.unverified,
    nonEquivalent: counts.nativeHandled - counts.nativeEquivalent,
  };
}

export function verifyZeroFallback(census: Json, policy: Json) {
  const blockers: Json[] = [];
  const products: Json = {};
  for (const [name, product] of Object.entries<Json>(census.products)) {
    const total = emptyCounts();
    const unmeasuredTargets: string[] = [];
    for (const [target, byTier] of Object.entries<ByTier>(product.targets)) {
      for (const key of countKeys) total[key] += byTier.total[key];
      for (const tier of bundleTiers) {
        if (byTier[tier].planned === 0) unmeasuredTargets.push(`${tier}/${target}`);
      }
    }
    const unmeasuredDialects = policy.dialects.filter(
      (dialect: string) => !(product.dialects.verified[dialect]?.total.planned > 0),
    );
    const counts = fallbackCounts(total);
    for (const [code, count] of Object.entries(counts)) {
      if (count > 0) blockers.push({ product: name, code, detail: `${count} coordinate(s)` });
    }
    for (const coordinate of unmeasuredTargets) {
      blockers.push({ product: name, code: "unmeasured-target", detail: coordinate });
    }
    for (const dialect of unmeasuredDialects) {
      blockers.push({ product: name, code: "unmeasured-dialect", detail: dialect });
    }
    products[name] = {
      planned: total.planned,
      nativeEquivalent: total.nativeEquivalent,
      ...counts,
      unmeasuredTargets,
      unmeasuredDialects,
      policyDialects: policy.dialects.length,
      holds:
        Object.values(counts).every((count) => count === 0) &&
        unmeasuredTargets.length === 0 &&
        unmeasuredDialects.length === 0,
    };
  }
  return {
    schema: "vize.differential.zero-fallback",
    version: 1,
    policySha256: census.policySha256,
    sourceRevisions: census.sourceRevisions,
    holds: blockers.length === 0,
    products,
    blockers,
  };
}

const rate = (counts: Counts) =>
  counts.planned === 0
    ? "unmeasured"
    : `${counts.nativeHandled}/${counts.planned} (${((100 * counts.nativeHandled) / counts.planned).toFixed(1)}%)`;

export function renderCensusMarkdown(census: Json, verdict: Json) {
  const revisions = bundleTiers
    .map((tier) => `${tier} \`${census.sourceRevisions[tier]?.slice(0, 12) ?? "absent"}\``)
    .join(", ");
  const lines = [
    "### Native-only acceptance per product",
    "",
    `Sources: ${revisions}. T1 is the in-repo fixture corpus, T2 the real-project corpus.`,
    "",
    "Product | Target | T1 native-only | T2 native-only | Equivalent | Legacy-backed | Unsupported | Unverified",
    "--- | --- | ---: | ---: | ---: | ---: | ---: | ---:",
  ];
  for (const [name, product] of Object.entries<Json>(census.products)) {
    for (const [target, byTier] of Object.entries<ByTier>(product.targets)) {
      const total = byTier.total;
      lines.push(
        `${name} | ${target} | ${rate(byTier.T1)} | ${rate(byTier.T2)} | ${total.nativeEquivalent}/${total.planned} | ${total.legacyBacked} | ${total.unsupported} | ${total.unverified}`,
      );
    }
  }
  lines.push(
    "",
    "Product | Dialect | Verified rows | Verified native-only | Provisional rows | Provisional native-only",
    "--- | --- | ---: | ---: | ---: | ---:",
  );
  for (const [name, product] of Object.entries<Json>(census.products)) {
    const dialects = new Set([
      ...Object.keys(product.dialects.verified),
      ...Object.keys(product.dialects.provisional),
    ]);
    for (const dialect of [...dialects].sort()) {
      const verified = product.dialects.verified[dialect]?.total ?? emptyCounts();
      const provisional = product.dialects.provisional[dialect]?.total ?? emptyCounts();
      lines.push(
        `${name} | ${dialect} | ${verified.planned} | ${verified.nativeHandled} | ${provisional.planned} | ${provisional.nativeHandled}`,
      );
    }
  }
  lines.push(
    "",
    `### Zero-fallback verdict: ${verdict.holds ? "holds" : "does not hold"}`,
    "",
    "Product | Planned | Fallback-free | Legacy fallbacks | Unverified | Non-equivalent | Unmeasured targets | Unmeasured dialects",
    "--- | ---: | ---: | ---: | ---: | ---: | --- | ---",
  );
  for (const [name, product] of Object.entries<Json>(verdict.products)) {
    lines.push(
      `${name} | ${product.planned} | ${product.nativeEquivalent} | ${product.legacyFallbacks} | ${product.unverified} | ${product.nonEquivalent} | ${product.unmeasuredTargets.join(", ") || "none"} | ${product.unmeasuredDialects.length}/${product.policyDialects}`,
    );
  }
  lines.push(
    "",
    "Fallback-free requires native handling with verified equivalence. Legacy fallbacks are legacy-backed plus unsupported coordinates. A dialect is measured only by verified fixture evidence; provisional labels do not count. Unmeasured coordinates block the verdict.",
    "",
  );
  return lines.join("\n");
}

function main() {
  const { values, positionals } = parseArgs({
    allowPositionals: true,
    options: {
      out: { type: "string" },
      policy: { type: "string" },
      "bundle-out": { type: "string" },
      enforce: { type: "boolean", default: false },
    },
  });
  assert(values.out && positionals.length > 0, "usage: native-acceptance.ts --out DIR BUNDLE...");
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
  const { policy, sha256: policySha256 } = loadPolicy(
    values.policy ?? path.join(root, "docs/davinci/plan/deletion-readiness-policy.json"),
  );
  const entries = readBundles(positionals);
  assert(entries.length > 0, "no differential bundle entries were found");
  if (values["bundle-out"]) copyBundle(entries, path.resolve(values["bundle-out"]));
  const census = buildCensus(entries, policy, policySha256);
  const verdict = verifyZeroFallback(census, policy);
  const markdown = renderCensusMarkdown(census, verdict);
  const out = path.resolve(values.out);
  fs.mkdirSync(out, { recursive: true });
  fs.writeFileSync(
    path.join(out, "native-acceptance.json"),
    `${JSON.stringify(census, null, 2)}\n`,
  );
  fs.writeFileSync(path.join(out, "zero-fallback.json"), `${JSON.stringify(verdict, null, 2)}\n`);
  fs.writeFileSync(path.join(out, "summary.md"), markdown);
  if (process.env.GITHUB_STEP_SUMMARY) fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, markdown);
  process.stdout.write(markdown);
  if (values.enforce && !verdict.holds) {
    process.stderr.write(`zero-fallback does not hold: ${verdict.blockers.length} blocker(s)\n`);
    process.exitCode = 1;
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();

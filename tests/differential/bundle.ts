// The shared differential evidence bundle (#6891, #6853).
//
// Every product runner writes the planned coordinates it loaded, its
// `vize.differential.result` report and its acceptance summary to one layout:
//
//   <bundle>/<tier>/<product>/<pack>/{manifest,result,acceptance}.json
//
// T1 packs are the in-repo fixture corpora; T2 is the real-project corpus,
// which `real-project.ts` writes one level up (`<bundle>/T2/<product>/`).
// Readers re-validate the result envelope against the planned coordinates and
// cross-check the acceptance counts against the rows, so a bundle cannot
// publish a rate its own rows contradict.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { validateResultEnvelope } from "./harness.mjs";
import { sha256 } from "./sha256.ts";

export const bundleTiers = ["T1", "T2"] as const;
export type Tier = (typeof bundleTiers)[number];
const productPattern = /^[a-z][a-z0-9-]*$/;
const packPattern = /^[a-z0-9][a-z0-9-]*$/;
export const realProjectPack = "real-projects";

type Json = Record<string, any>;
interface Loaded {
  manifest: Json;
  cases: Json[];
  manifestSha256: string;
}
export interface BundleEntry {
  tier: Tier;
  product: string;
  pack: string;
  dir: string;
  sourceRevision: string;
  loaded: Loaded;
  report: Json;
  acceptance: Json;
}

export function defaultBundleDir(repoRoot: string): string {
  return path.resolve(
    repoRoot,
    process.env.VIZE_DIFFERENTIAL_BUNDLE_DIR ?? "target/differential/bundle",
  );
}

// Planned coordinates exactly as the product adapter loaded them. Adapters may
// expand packs or attach pinned inputs, so the committed manifest bytes are
// bound by digest rather than copied.
function plannedManifest(loaded: Loaded, manifestPath: string, repoRoot: string): Json {
  const relative = path.relative(repoRoot, manifestPath).split(path.sep).join("/");
  assert(relative && !relative.startsWith(".."), "manifest must be inside the repository");
  assert.equal(sha256(fs.readFileSync(manifestPath)), loaded.manifestSha256, "manifest digest");
  return {
    schema: "vize.differential.manifest",
    version: 1,
    product: loaded.manifest.product,
    baseRevision: loaded.manifest.baseRevision,
    sourceManifest: { path: relative, sha256: loaded.manifestSha256 },
    cases: loaded.cases.map((fixture) => ({
      id: fixture.id,
      state: fixture.state,
      targets: [...fixture.targets],
      dialectCoverage: fixture.dialectCoverage,
      ...(fixture.projectId === undefined ? {} : { projectId: fixture.projectId }),
    })),
  };
}

export function writeBundleEntry(options: {
  repoRoot: string;
  bundleDir?: string;
  tier: Tier;
  pack: string;
  manifestPath: string;
  loaded: Loaded;
  report: Json;
  acceptance: Json;
}): string {
  const { tier, pack, loaded, report, acceptance } = options;
  assert(bundleTiers.includes(tier), `unknown tier: ${tier}`);
  assert.match(pack, packPattern, `invalid pack name: ${pack}`);
  const product = loaded.manifest.product;
  assert.match(product, productPattern);
  validateResultEnvelope(loaded, report, report.sourceRevision);
  assert.equal(acceptance.product, product, "acceptance product");
  assert.equal(acceptance.manifestSha256, loaded.manifestSha256, "acceptance manifest");
  assert.equal(acceptance.sourceRevision, report.sourceRevision, "acceptance revision");
  const dir = path.join(
    options.bundleDir ?? defaultBundleDir(options.repoRoot),
    tier,
    product,
    pack,
  );
  fs.mkdirSync(dir, { recursive: true });
  const manifest = plannedManifest(loaded, path.resolve(options.manifestPath), options.repoRoot);
  for (const [name, value] of Object.entries({ manifest, result: report, acceptance })) {
    fs.writeFileSync(path.join(dir, `${name}.json`), `${JSON.stringify(value, null, 2)}\n`);
  }
  return dir;
}

function readJson(file: string): Json {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

// Non-completed native states map one-to-one onto acceptance buckets; a
// completed row may still be demoted by the adapter's provenance verifier, but
// never promoted beyond the completed rows.
export function assertAcceptanceMatchesRows(entry: Omit<BundleEntry, "sourceRevision" | "dir">) {
  const { loaded, report, acceptance } = entry;
  const targetsOf = new Map(loaded.cases.map((fixture) => [fixture.id, fixture.targets]));
  const observed = new Map<string, Json>();
  for (const row of report.rows) {
    const target = row.target ?? targetsOf.get(row.id)?.[0];
    const counts = observed.get(target) ?? { planned: 0, completed: 0, unsupported: 0, legacy: 0 };
    counts.planned += 1;
    if (row.native?.state === "completed") counts.completed += 1;
    else if (row.native?.state === "unsupported") counts.unsupported += 1;
    else if (row.native?.state === "legacy-backed") counts.legacy += 1;
    observed.set(target, counts);
  }
  const targets = acceptance.targets ?? {};
  assert.deepEqual(Object.keys(targets).sort(), [...observed.keys()].sort(), "acceptance targets");
  for (const [target, rows] of observed) {
    const counts = targets[target];
    const context = `${entry.tier}/${entry.product}/${entry.pack}/${target}`;
    assert.equal(counts.planned, rows.planned, `${context}: planned`);
    assert.equal(counts.unsupported, rows.unsupported, `${context}: unsupported`);
    assert(counts.legacyBacked >= rows.legacy, `${context}: legacy-backed`);
    assert(counts.nativeHandled <= rows.completed, `${context}: native credit without a row`);
    assert(counts.nativeEquivalent <= counts.nativeHandled, `${context}: equivalent`);
    assert.equal(
      counts.nativeHandled + counts.legacyBacked + counts.unsupported + counts.unverified,
      counts.planned,
      `${context}: buckets`,
    );
  }
}

function readEntry(dir: string, tier: Tier, product: string, pack: string): BundleEntry {
  const manifestBytes = fs.readFileSync(path.join(dir, "manifest.json"));
  const manifest = JSON.parse(manifestBytes.toString("utf8"));
  assert.equal(manifest.schema, "vize.differential.manifest", `${dir}: manifest schema`);
  assert.equal(manifest.product, product, `${dir}: manifest product`);
  const loaded = {
    manifest,
    cases: manifest.cases,
    manifestSha256: manifest.sourceManifest?.sha256 ?? sha256(manifestBytes),
  };
  const report = readJson(path.join(dir, "result.json"));
  const acceptance = readJson(path.join(dir, "acceptance.json"));
  assert.match(report.sourceRevision ?? "", /^[a-f0-9]{40}$/, `${dir}: source revision`);
  validateResultEnvelope(loaded, report, report.sourceRevision);
  assert.equal(acceptance.schema, "vize.differential.acceptance", `${dir}: acceptance schema`);
  assert.equal(acceptance.product, product, `${dir}: acceptance product`);
  assert.equal(acceptance.manifestSha256, loaded.manifestSha256, `${dir}: acceptance manifest`);
  assert.equal(acceptance.sourceRevision, report.sourceRevision, `${dir}: acceptance revision`);
  const entry = { tier, product, pack, sourceRevision: report.sourceRevision, loaded, report };
  assertAcceptanceMatchesRows({ ...entry, acceptance });
  return { ...entry, dir, acceptance };
}

function readBundleRoot(root: string, entries: Map<string, BundleEntry>) {
  for (const tier of bundleTiers) {
    const tierDir = path.join(root, tier);
    if (!fs.existsSync(tierDir)) continue;
    for (const product of fs.readdirSync(tierDir).sort()) {
      const productDir = path.join(tierDir, product);
      if (!productPattern.test(product) || !fs.statSync(productDir).isDirectory()) continue;
      const packs = fs.existsSync(path.join(productDir, "manifest.json"))
        ? [[realProjectPack, productDir]]
        : fs
            .readdirSync(productDir)
            .sort()
            .filter((pack) => packPattern.test(pack))
            .map((pack) => [pack, path.join(productDir, pack)]);
      for (const [pack, dir] of packs) {
        if (!fs.existsSync(path.join(dir, "acceptance.json"))) continue;
        const entry = readEntry(dir, tier, product, pack);
        const key = `${tier}/${product}/${pack}`;
        const previous = entries.get(key);
        if (previous) {
          // A pack observed by two workers must agree exactly.
          assert.deepEqual(entry.report, previous.report, `${key}: conflicting duplicate pack`);
          continue;
        }
        entries.set(key, entry);
      }
    }
  }
}

export const bundleFiles = ["manifest.json", "result.json", "acceptance.json"] as const;

// Copies every entry's files byte for byte into one merged bundle root.
export function copyBundle(entries: BundleEntry[], out: string) {
  for (const entry of entries) {
    const dir = path.join(out, entry.tier, entry.product, entry.pack);
    fs.mkdirSync(dir, { recursive: true });
    for (const file of bundleFiles)
      fs.copyFileSync(path.join(entry.dir, file), path.join(dir, file));
  }
}

// Each argument is a bundle root, or a directory searched for `bundle/` roots
// (for example a download of several Actions artifacts).
export function readBundles(dirs: string[]): BundleEntry[] {
  const entries = new Map<string, BundleEntry>();
  const visit = (dir: string, depth: number) => {
    if (bundleTiers.some((tier) => fs.existsSync(path.join(dir, tier)))) {
      readBundleRoot(dir, entries);
    }
    if (depth === 0) return;
    for (const child of fs.readdirSync(dir).sort()) {
      const next = path.join(dir, child);
      if (fs.statSync(next).isDirectory() && !bundleTiers.includes(child as Tier)) {
        visit(next, depth - 1);
      }
    }
  };
  for (const dir of dirs) visit(path.resolve(dir), 4);
  return [...entries.values()].sort((a, b) =>
    `${a.tier}/${a.product}/${a.pack}`.localeCompare(`${b.tier}/${b.product}/${b.pack}`),
  );
}

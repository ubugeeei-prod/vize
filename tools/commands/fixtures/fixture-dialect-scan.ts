// Record pinned-package dialect evidence for every registered real project (#6892).
//
// For each fixture, read the Git tree at its pinned revision, select the
// package.json files that own its registered source globs, and record one
// `pinned-package` claim per dialect a dependency proves: the Vue major
// (vue2 / vue2.7 / vue3), petite-vue, Pug and the JSX dialect. A pnpm
// `catalog:` range resolves through pnpm-workspace.yaml and records that blob.
// Existing registry-note and test-oracle claims stay. Quirks have no package
// signal; that dialect stays covered by the synthetic corpus.
//
//   GITHUB_TOKEN=... node tools/commands/fixtures/fixture-dialect-scan.ts --write
//
// The ledger tests validate every recorded claim offline against the registry.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  compatibilityLedgerPath,
  createCompatibilityContext,
  readCompatibilityLedger,
  validateCompatibilityLedger,
} from "../../support/compat/fixtures/fixture-compatibility-ledger.ts";
import {
  packageDialect,
  type RegistryProject,
} from "../../support/compat/fixtures/fixture-dialect-coverage.ts";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const fields = ["dependencies", "devDependencies", "peerDependencies"] as const;

interface TreeEntry {
  readonly path: string;
  readonly type: string;
  readonly sha: string;
}
interface Claim {
  kind: "pinned-package";
  dialects: string[];
  repository: string;
  revision: string;
  file: string;
  blobSha: string;
  field: string;
  value: string;
  catalog?: { file: string; blobSha: string; value: string };
}

async function github<T>(route: string): Promise<T> {
  const token = process.env.GITHUB_TOKEN;
  assert.ok(token, "GITHUB_TOKEN is required to read pinned trees");
  for (let attempt = 1; ; attempt += 1) {
    const response = await fetch(`https://api.github.com/${route}`, {
      headers: { authorization: `Bearer ${token}`, accept: "application/vnd.github+json" },
    });
    if (response.ok) return (await response.json()) as T;
    if (attempt === 3) throw new Error(`${route}: HTTP ${response.status}`);
  }
}

async function blobText(slug: string, sha: string): Promise<string> {
  const blob = await github<{ content: string }>(`repos/${slug}/git/blobs/${sha}`);
  return Buffer.from(blob.content, "base64").toString("utf8");
}

// The static directory prefix of a source glob, e.g. `packages/ui/src`.
const globPrefix = (glob: string) =>
  glob
    .split("/")
    .filter((_, index, parts) => !parts.slice(0, index + 1).some((part) => /[*?{[]/.test(part)))
    .join("/");

// A package.json owns sources when its directory is the root, an ancestor of
// a glob prefix, or inside one; an unprefixed glob covers the whole tree.
function ownsSources(file: string, prefixes: readonly string[]): boolean {
  if (file.split("/").includes("node_modules")) return false;
  const directory = path.posix.dirname(file);
  if (directory === ".") return true;
  return prefixes.some(
    (prefix) =>
      prefix === "" ||
      prefix === directory ||
      prefix.startsWith(`${directory}/`) ||
      directory.startsWith(`${prefix}/`),
  );
}

const indentOf = (line: string) => line.length - line.trimStart().length;
const unquote = (text: string) => text.trim().replace(/^(['"])(.*)\1$/u, "$2");

// `catalog:` and `catalog:<name>` ranges resolve through pnpm-workspace.yaml.
function catalogRange(workspace: string, value: string, name: string): string | undefined {
  const catalog = value.slice("catalog:".length);
  const lines = workspace.split(/\r?\n/u);
  let block = -1;
  if (catalog === "" || catalog === "default") {
    block = lines.findIndex((line) => line.trimEnd() === "catalog:");
  } else {
    const catalogs = lines.findIndex((line) => line.trimEnd() === "catalogs:");
    block = lines.findIndex(
      (line, index) =>
        catalogs >= 0 &&
        index > catalogs &&
        indentOf(line) > 0 &&
        unquote(line.trim().replace(/:$/u, "")) === catalog,
    );
  }
  if (block < 0) return undefined;
  const indent = indentOf(lines[block]!);
  for (const line of lines.slice(block + 1)) {
    if (line.trim() === "" || line.trim().startsWith("#")) continue;
    if (indentOf(line) <= indent) break;
    const separator = line.indexOf(":");
    if (separator < 0) continue;
    if (unquote(line.slice(0, separator)) === name) {
      return unquote(line.slice(separator + 1).replace(/\s#.*$/u, ""));
    }
  }
  return undefined;
}

async function scanProject(project: RegistryProject): Promise<Claim[]> {
  const repository = String(project.repository);
  const revision = String(project.revision);
  const slug = repository.replace(/^https:\/\/github\.com\//u, "").replace(/\.git$/u, "");
  const tree = await github<{ tree: TreeEntry[]; truncated: boolean }>(
    `repos/${slug}/git/trees/${revision}?recursive=1`,
  );
  assert.ok(!tree.truncated, `${slug}: tree listing is truncated`);
  const globs = [...(project.vueGlobs ?? []), ...((project.petiteVueGlobs as string[]) ?? [])];
  const prefixes = globs.map(globPrefix);
  const manifests = tree.tree
    .filter((entry) => entry.type === "blob" && /(?:^|\/)package\.json$/u.test(entry.path))
    .filter((entry) => ownsSources(entry.path, prefixes))
    .sort(
      (left, right) =>
        left.path.split("/").length - right.path.split("/").length ||
        (left.path < right.path ? -1 : 1),
    );
  const workspaceEntry = tree.tree.find((entry) => entry.path === "pnpm-workspace.yaml");
  const workspace = workspaceEntry ? await blobText(slug, workspaceEntry.sha) : "";
  const claims = new Map<string, Claim>();
  for (const entry of manifests) {
    const manifest = JSON.parse(await blobText(slug, entry.sha)) as Record<
      string,
      Record<string, unknown> | undefined
    >;
    for (const field of fields) {
      for (const [name, value] of Object.entries(manifest[field] ?? {})) {
        if (typeof value !== "string") continue;
        const resolved =
          workspaceEntry && value.startsWith("catalog:")
            ? catalogRange(workspace, value, name)
            : undefined;
        const dialect = packageDialect(name, resolved ?? value);
        if (dialect === undefined || claims.has(dialect)) continue;
        const claim: Claim = {
          kind: "pinned-package",
          dialects: [dialect],
          repository,
          revision,
          file: entry.path,
          blobSha: entry.sha,
          field: `${field}.${name}`,
          value,
        };
        if (workspaceEntry && resolved !== undefined) {
          claim.catalog = {
            file: "pnpm-workspace.yaml",
            blobSha: workspaceEntry.sha,
            value: resolved,
          };
        }
        claims.set(dialect, claim);
      }
    }
  }
  return [...claims.values()];
}

async function main() {
  assert.equal(process.argv[2], "--write", "usage: --write");
  const context = createCompatibilityContext(root);
  const ledger = readCompatibilityLedger();
  const projects = new Map(
    context.registry.projects.map((project) => [project.fixturePath, project]),
  );
  for (const fixture of ledger.fixtures) {
    const project = projects.get(fixture.fixturePath);
    if (!project?.vueGlobs?.length) continue;
    const coverage = fixture.dialectCoverage as { state: string; evidence: Claim[] };
    const kept = coverage.evidence.filter((item) => item.kind !== "pinned-package");
    const claimed = new Set(kept.flatMap((item) => item.dialects));
    const scanned = (await scanProject(project)).filter(
      (claim) => !claimed.has(claim.dialects[0]!),
    );
    coverage.evidence = [...kept, ...scanned];
    coverage.state = coverage.evidence.length ? "partial" : "unknown";
    const dialects = coverage.evidence.flatMap((item) => item.dialects).join(", ");
    process.stderr.write(`${fixture.fixturePath}: ${dialects || "-"}\n`);
  }
  validateCompatibilityLedger(ledger, context);
  // The repository formatter owns the ledger layout.
  fs.writeFileSync(compatibilityLedgerPath, JSON.stringify(ledger));
  execFileSync(path.join(root, "node_modules/.bin/oxfmt"), [compatibilityLedgerPath], {
    stdio: "ignore",
  });
}

await main();

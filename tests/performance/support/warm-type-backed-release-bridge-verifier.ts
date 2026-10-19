// Read-only complete-tree audit of an official release candidate and its source cut.
// Never invokes Cargo, vp, installs, a provider, or a release. The output is an
// attestation of a specific tree relation, not publication proof.
// usage: node warm-type-backed-release-bridge-verifier.ts REPO SOURCE_CUT RELEASE_HEAD
//   OLD_VERSION NEW_VERSION RECIPE OUTPUT [--pr N] [--offline-historical]
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { parseArgs } from "node:util";
import { parse as parseToml } from "@iarna/toml";

const schema = "vize.release.source-cut-version-bridge";
const guestLocks = [
  "davinci/vize_extension_host/tests/guests/expression-echo/Cargo.lock",
  "davinci/vize_extension_host/tests/guests/output-echo/Cargo.lock",
  "davinci/vize_extension_host/tests/guests/typed-expression-echo/Cargo.lock",
  "examples/volt-target/Cargo.lock",
];

type Json = null | boolean | number | string | readonly Json[] | { readonly [key: string]: Json };
type JsonObject = { readonly [key: string]: Json };
interface TreeEntry {
  readonly mode: string;
  readonly type: string;
  readonly oid: string;
}
interface RecipeSource {
  readonly path: string;
  readonly sha256: string;
}

function recipeSources(value: unknown): readonly RecipeSource[] {
  assert.ok(Array.isArray(value), "recipe sources");
  return value.map((row: unknown) => {
    const { path, sha256 } = object(row, "recipe source");
    assert.ok(typeof path === "string" && typeof sha256 === "string", "recipe source");
    return { path, sha256 };
  });
}

const sha = (data: string | Uint8Array): string => createHash("sha256").update(data).digest("hex");
const compact = (value: unknown): string => JSON.stringify(value);

function object(value: unknown, label: string): JsonObject {
  assert.ok(typeof value === "object" && value !== null && !Array.isArray(value), label);
  return value as JsonObject;
}

function field(value: unknown, ...keys: readonly string[]): unknown {
  let current = value;
  for (const key of keys) current = object(current, keys.join("."))[key];
  return current;
}

function replaceFirst(line: string, old: string, next: string): string {
  const at = line.indexOf(old);
  return at === -1 ? line : line.slice(0, at) + next + line.slice(at + old.length);
}

export function firstVersion(
  content: string,
  old: string,
  next: string,
  jsonField = false,
): string {
  const lines = content.split("\n");
  const index = lines.findIndex((line) => {
    const text = line.trim();
    const match = jsonField
      ? text.startsWith('"version"')
      : text.startsWith('version = "') && text.endsWith('"');
    return match && line.includes(old);
  });
  if (index !== -1) lines[index] = replaceFirst(lines[index] ?? "", old, next);
  return lines.join("\n");
}

function workspaceManifest(
  content: string,
  old: string,
  next: string,
  ownNames: ReadonlySet<string>,
): string {
  let section = "";
  return content
    .split("\n")
    .map((line) => {
      const text = line.trim();
      if (text.startsWith("[") && text.endsWith("]")) section = text;
      if (section === "[workspace.package]" && text === `version = "${old}"`)
        return replaceFirst(line, old, next);
      if (
        section === "[workspace.dependencies]" &&
        (text.startsWith("vize_") || line.includes('package = "vize_')) &&
        line.includes(`version = "=${old}"`)
      ) {
        const [entry] = Object.entries(parseToml(line));
        assert.ok(entry);
        const [key, dependency] = entry;
        const name = field(dependency, "package") ?? key;
        assert.ok(
          typeof name === "string" && ownNames.has(name) && "path" in object(dependency, key),
          `not an owned path pin: ${JSON.stringify(name)}`,
        );
        return line.replaceAll(`version = "=${old}"`, `version = "=${next}"`);
      }
      return line;
    })
    .join("\n");
}

function nativeCatalog(content: string, old: string, next: string, workspace: boolean): string {
  const out: string[] = [];
  let inBlock = false;
  for (let line of content.split("\n")) {
    if (workspace && line.startsWith('  - "@vizejs/native-') && line.includes(old)) {
      if (!line.includes(next)) line = replaceFirst(line, old, `${old} || ${next}`);
      out.push(line);
      continue;
    }
    if (line === "  native-binaries:") {
      inBlock = true;
      out.push(line);
      continue;
    }
    if (inBlock) {
      let leaves = line !== "" && !line.startsWith("    ");
      if (workspace) leaves = leaves && !line.startsWith("  #");
      if (leaves) inBlock = false;
      else if (workspace && line.startsWith('    "@vizejs/native-') && line.includes(old))
        line = replaceFirst(line, old, next);
      else if (
        !workspace &&
        (line.startsWith("      specifier: ") || line.startsWith("      version: ")) &&
        line.includes(old)
      )
        line = replaceFirst(line, old, next);
    }
    out.push(line);
  }
  return out.join("\n");
}

function readme(content: string, old: string, next: string): string {
  let benchmark = false;
  return content
    .split("\n")
    .map((line) => {
      if (line.includes("<!-- benchmark:readme:start -->")) benchmark = true;
      const result = benchmark ? line : line.replaceAll(old, next);
      if (line.includes("<!-- benchmark:readme:end -->")) benchmark = false;
      return result;
    })
    .join("\n");
}

const escapeRegExp = (text: string): string => text.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");

/** Only version fields of source-less in-tree packages; all foreign bytes exact. */
function cargoLock(content: string, old: string, next: string, names: ReadonlySet<string>): string {
  const dependency = new RegExp(`^(\\s*")([^" ]+) ${escapeRegExp(old)}(",?)$`, "u");
  return content
    .split(/(?=^\[\[package\]\]$)/mu)
    .map((block) => {
      if (!block.startsWith("[[package]]")) return block;
      const packages = field(parseToml(block), "package");
      assert.ok(Array.isArray(packages));
      const lockPackage = object(packages[0], "package");
      const lines = block.split("\n");
      const name = lockPackage.name;
      if (
        typeof name === "string" &&
        names.has(name) &&
        !("source" in lockPackage) &&
        lockPackage.version === old
      ) {
        const index = lines.indexOf(`version = "${old}"`);
        assert.notEqual(index, -1);
        lines[index] = `version = "${next}"`;
      }
      return lines
        .map((line) => {
          const match = dependency.exec(line);
          return match?.[2] !== undefined && names.has(match[2])
            ? `${match[1]}${match[2]} ${next}${match[3]}`
            : line;
        })
        .join("\n");
    })
    .join("");
}

function audit(
  repo: string,
  sourceCut: string,
  releaseHead: string,
  old: string,
  next: string,
  recipePath: string,
  pr: number | undefined,
  offline: boolean,
): Record<string, unknown> {
  const git = (...args: string[]): Buffer =>
    execFileSync("git", args, { cwd: repo, maxBuffer: 1024 * 1024 * 1024 });
  const blob = (rev: string, path: string): Buffer => git("show", `${rev}:${path}`);
  const text = (rev: string, path: string): string => blob(rev, path).toString("utf8");
  const tree = (rev: string): Map<string, TreeEntry> => {
    const rows = new Map<string, TreeEntry>();
    for (const raw of git("ls-tree", "-r", "-z", "--full-tree", rev).toString("utf8").split("\0")) {
      if (!raw) continue;
      const tab = raw.indexOf("\t");
      const [mode = "", type = "", oid = ""] = raw.slice(0, tab).split(" ");
      rows.set(raw.slice(tab + 1), { mode, type, oid });
    }
    return rows;
  };

  assert.match(sourceCut, /^[0-9a-f]{40}$/u);
  assert.match(releaseHead, /^[0-9a-f]{40}$/u);
  const [major = 0, minor = 0] = old.split(".").map(Number);
  assert.deepEqual(
    next.split(".").map(Number),
    [major, minor + 1, 0],
    "not the selected minor bump",
  );
  const parentLine = git("rev-list", "--parents", "-n", "1", releaseHead)
    .toString()
    .trim()
    .split(/\s+/u);
  assert.deepEqual(
    parentLine,
    [releaseHead, sourceCut],
    "candidate is not the sole child of the source cut",
  );
  assert.equal(
    git("show", "-s", "--format=%B", releaseHead).toString().trim(),
    `chore: release v${next}`,
  );
  const before = tree(sourceCut);
  const after = tree(releaseHead);
  assert.deepEqual(
    [...before.keys()].sort(),
    [...after.keys()].sort(),
    "tracked path additions/removals are not in the official recipe",
  );
  assert.equal(
    field(parseToml(text(sourceCut, "Cargo.toml")), "workspace", "package", "version"),
    old,
  );
  assert.equal(
    field(parseToml(text(releaseHead, "Cargo.toml")), "workspace", "package", "version"),
    next,
  );
  const recipeBytes = fs.readFileSync(recipePath);
  const recipe = object(JSON.parse(recipeBytes.toString("utf8")), "recipe");
  assert.equal(recipe.schema, "vize.release.official-version-bridge-recipe");
  const sources = recipeSources(recipe.sources);
  for (const row of sources)
    assert.equal(
      sha(blob(sourceCut, row.path)),
      row.sha256,
      `release recipe needs a new review: ${row.path}`,
    );

  const ownNames = new Set<string>();
  for (const path of before.keys()) {
    if (!path.endsWith("Cargo.toml")) continue;
    const manifestPackage = field(parseToml(text(sourceCut, path)), "package") ?? {};
    const value = field(manifestPackage, "version");
    const inherited =
      typeof value === "object" && value !== null && field(value, "workspace") === true;
    if (value === old || inherited) ownNames.add(String(field(manifestPackage, "name")));
  }
  // Every owned identity has a tracked source manifest; registry packages with
  // a coincidentally equal version are never admitted by name prefix alone.
  const npmPaths = new Set<string>();
  const readmePaths = new Set<string>(before.has("README.md") ? ["README.md"] : []);
  for (const path of before.keys()) {
    const parts = path.split("/");
    if (parts.length === 3 && parts[0] === "npm" && parts[2] === "package.json") npmPaths.add(path);
    else if (
      parts.length === 4 &&
      parts[0] === "npm" &&
      parts[3] === "package.json" &&
      !before.has(`${parts.slice(0, 2).join("/")}/package.json`) &&
      field(JSON.parse(text(sourceCut, path)), "private") !== true
    )
      npmPaths.add(path);
  }
  for (const path of ["editors/vscode/package.json", "editors/vscode-art/package.json"])
    if (before.has(path)) npmPaths.add(path);
  for (const path of before.keys()) {
    const parts = path.split("/");
    if (parts.length === 3 && parts[0] === "npm" && parts[2] === "README.md") readmePaths.add(path);
  }
  for (const path of npmPaths) {
    const readmePath = `${path.slice(0, -"package.json".length)}README.md`;
    if (path.startsWith("npm/") && before.has(readmePath)) readmePaths.add(readmePath);
  }

  const locks = new Set(
    ["Cargo.lock", "editors/zed/Cargo.lock", ...guestLocks].filter((path) => before.has(path)),
  );
  const zedManifests = new Set(
    ["editors/zed/Cargo.toml", "editors/zed/extension.toml"].filter((path) => before.has(path)),
  );
  const allowed = new Set([
    "Cargo.toml",
    "pnpm-lock.yaml",
    "pnpm-workspace.yaml",
    ...locks,
    ...zedManifests,
    ...npmPaths,
    ...readmePaths,
  ]);
  const changed: Record<string, unknown>[] = [];
  const allPaths: Record<string, unknown>[] = [];
  for (const path of [...before.keys()].sort(comparePaths)) {
    const a = before.get(path);
    const b = after.get(path);
    assert.ok(a && b);
    allPaths.push({ path, before: a, after: b });
    if (a.mode === b.mode && a.type === b.type && a.oid === b.oid) continue;
    assert.ok(
      a.mode === b.mode && a.type === "blob" && b.type === "blob",
      `mode/type changed: ${path}`,
    );
    assert.ok(
      allowed.has(path),
      `unapproved implementation/changelog/fixture/provider/dependency path: ${path}`,
    );
    const original = blob(sourceCut, path);
    const actual = blob(releaseHead, path);
    const content = original.toString("utf8");
    let expected: string;
    let kind: string;
    if (path === "Cargo.toml") {
      expected = workspaceManifest(content, old, next, ownNames);
      kind = "owned workspace package version and exact in-tree crate pins";
    } else if (locks.has(path)) {
      expected = cargoLock(content, old, next, ownNames);
      kind =
        "source-less owned crate versions only; all foreign package records and bytes unchanged";
    } else if (npmPaths.has(path)) {
      expected = firstVersion(content, old, next, true);
      const fields = { ...object(JSON.parse(content), path) };
      if (expected !== content) {
        assert.equal(fields.version, old);
        fields.version = next;
      }
      assert.deepEqual(JSON.parse(expected), fields, path);
      kind = "owned package top-level version only; dependency fields unchanged";
    } else if (zedManifests.has(path)) {
      expected = firstVersion(content, old, next);
      kind = "Zed first release version only";
    } else if (path === "pnpm-workspace.yaml") {
      expected = nativeCatalog(content, old, next, true);
      kind = "own native-binaries catalog and own minimum-release-age additions only";
    } else if (path === "pnpm-lock.yaml") {
      expected = nativeCatalog(content, old, next, false);
      kind =
        "own native-binaries catalog specifier/version only; all dependency graph bytes unchanged";
    } else if (readmePaths.has(path)) {
      expected = readme(content, old, next);
      kind = "official README release references; marked benchmark sections unchanged";
    } else {
      throw new Error(path);
    }
    assert.ok(
      Buffer.from(expected).equals(actual),
      `not byte-exact official scoped version transformation: ${path}`,
    );
    changed.push({
      path,
      before: a,
      after: b,
      beforeSha256: sha(original),
      afterSha256: sha(actual),
      classification: kind,
    });
  }
  assert.ok(changed.length > 0, "empty candidate");
  assert.equal(recipe.trackedChangelogMutationsAllowed, false);
  const revTree = (rev: string): string => git("rev-parse", `${rev}^{tree}`).toString().trim();
  const sourceCutRecord = { sha: sourceCut, tree: revTree(sourceCut), version: old };
  const releaseHeadRecord = {
    sha: releaseHead,
    tree: revTree(releaseHead),
    version: next,
    parents: [sourceCut],
  };
  const result: Record<string, unknown> = {
    schema,
    version: 1,
    status: offline ? "OFFLINE_HISTORICAL_RELATION_ONLY" : "CANDIDATE_TREE_RELATION_VERIFIED",
    sourceCut: sourceCutRecord,
    releaseHead: releaseHeadRecord,
    recipeSha256: sha(recipeBytes),
    recipeSources: sources,
    allPathManifestSha256: sha(compact(allPaths)),
    allPathManifest: allPaths,
    changedPathCount: changed.length,
    unchangedPathCount: before.size - changed.length,
    changedPaths: changed,
    allowedCandidatePaths: [...allowed].sort(comparePaths),
    allNonRecipeEntriesByteIdentical: true,
    thirdPartyDependencyContentUnchanged: true,
    trackedChangelogChanged: false,
    generatedReleaseNotes: "GitHub Release metadata only; not a tracked blob allowance",
    implementationFixtureHelperCapsProviderContentUnchanged: true,
    limits: [
      "This verifies the candidate/source relation, not publication, performance, installed behavior or native-default completion.",
      "Public authority must separately authenticate actual tagHead==releaseHead; sourceCut and tagHead remain distinct.",
      "Only initialize.serverInfo.version may be adjusted from sourceCut.version to tagHead.version in the explicitly reviewed public comparator; all other authored objects remain whole.",
    ],
  };
  result.authorityProjection = {
    schema,
    version: 1,
    sourceCut: sourceCutRecord,
    tagHead: { sha: releaseHead, tree: releaseHeadRecord.tree, version: next },
    recipeSha256: result.recipeSha256,
    allPathManifestSha256: result.allPathManifestSha256,
    changedPathManifestSha256: sha(compact(changed)),
    changedPathCount: changed.length,
    unchangedPathCount: before.size - changed.length,
  };
  result.authorityProjectionSha256 = sha(compact(result.authorityProjection));
  if (!offline) {
    assert.ok(pr !== undefined && pr > 7811);
    const api = (path: string): { rawSha256: string; body: unknown } => {
      const raw = execFileSync("gh", ["api", `repos/ubugeeei-prod/vize/${path}`], {
        maxBuffer: 256 * 1024 * 1024,
      });
      return { rawSha256: sha(raw), body: JSON.parse(raw.toString("utf8")) };
    };
    const sourceApi = api(`commits/${sourceCut}`);
    const headApi = api(`commits/${releaseHead}`);
    const pull = api(`pulls/${pr}`);
    assert.equal(field(sourceApi.body, "commit", "verification", "verified"), true);
    assert.equal(field(sourceApi.body, "commit", "verification", "reason"), "valid");
    assert.equal(field(sourceApi.body, "sha"), sourceCut);
    assert.equal(field(headApi.body, "sha"), releaseHead);
    const parents = field(headApi.body, "parents");
    assert.ok(Array.isArray(parents) && parents.length === 1);
    assert.equal(field(parents[0], "sha"), sourceCut);
    assert.equal(field(sourceApi.body, "commit", "tree", "sha"), sourceCutRecord.tree);
    assert.equal(field(headApi.body, "commit", "tree", "sha"), releaseHeadRecord.tree);
    assert.equal(field(pull.body, "head", "sha"), releaseHead);
    assert.equal(field(pull.body, "head", "ref"), `release/v${next}`);
    assert.equal(field(pull.body, "base", "ref"), "main");
    assert.equal(field(pull.body, "head", "repo", "full_name"), "ubugeeei-prod/vize");
    assert.equal(field(pull.body, "base", "repo", "full_name"), "ubugeeei-prod/vize");
    const body = String(field(pull.body, "body"));
    assert.ok(body.includes(`<!-- vize-release-base-version: ${old} -->`));
    assert.ok(body.includes("<!-- vize-release-bump: minor -->"));
    result.officialApi = { sourceCut: sourceApi, releaseHead: headApi, releasePr: pull };
    result.publicationState = "separate actual tag and publication proof required";
  }
  return result;
}

/** Code-point order, matching Git's byte order for UTF-8 paths. */
function comparePaths(left: string, right: string): number {
  return Buffer.compare(Buffer.from(left), Buffer.from(right));
}

const { values, positionals } = parseArgs({
  options: { pr: { type: "string" }, "offline-historical": { type: "boolean", default: false } },
  allowPositionals: true,
});
const [repo, sourceCut, releaseHead, oldVersion, newVersion, recipe, output] = positionals;
if (
  positionals.length !== 7 ||
  !repo ||
  !sourceCut ||
  !releaseHead ||
  !oldVersion ||
  !newVersion ||
  !recipe ||
  !output
)
  throw new Error(
    "usage: REPO SOURCE_CUT RELEASE_HEAD OLD_VERSION NEW_VERSION RECIPE OUTPUT [--pr N]",
  );
const pr = values.pr === undefined ? undefined : Number.parseInt(values.pr, 10);
const offline = values["offline-historical"] === true;
const result = audit(repo, sourceCut, releaseHead, oldVersion, newVersion, recipe, pr, offline);
fs.writeFileSync(output, `${JSON.stringify(result, null, 2)}\n`);
process.stdout.write(
  `${JSON.stringify({
    path: output,
    sha256: sha(fs.readFileSync(output)),
    status: result.status,
    changed: result.changedPathCount,
    unchanged: result.unchangedPathCount,
  })}\n`,
);

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { deriveJsrPublication } from "./jsr.ts";
import type { JsrPublicationPlan } from "./jsr.ts";

export interface PublicationTarget {
  name: string;
  version: string;
}

export interface PublicationPlan {
  head: string;
  parentCut: string;
  version: string;
  npm: PublicationTarget[];
  crates: PublicationTarget[];
  editor: PublicationTarget & { publisher: string };
  githubAssets: string[];
  jsr?: JsrPublicationPlan;
  authority: {
    commitSha256: string;
    tree: string;
    blobs: { path: string; oid: string; sha256: string }[];
  };
}

const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

function requireValue(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function unique(values: string[], description: string): string[] {
  requireValue(values.length > 0, `empty ${description}`);
  requireValue(new Set(values).size === values.length, `duplicate ${description}`);
  return values;
}

function rawGit(root: string, ...args: string[]) {
  return execFileSync("git", ["--no-replace-objects", ...args], {
    cwd: root,
    env: { ...process.env, GIT_NO_REPLACE_OBJECTS: "1" },
    maxBuffer: 4 * 1024 * 1024,
    timeout: 30_000,
  });
}

function requireHead(root: string, head: string) {
  requireValue(/^[0-9a-f]{40}$/.test(head), "full source H commit required");
  requireValue(
    rawGit(root, "for-each-ref", "--format=%(refname)", "refs/replace").length === 0,
    "replacement objects must be removed before deriving public acceptance",
  );
  requireValue(
    rawGit(root, "cat-file", "-t", head).toString().trim() === "commit",
    "H must be a commit",
  );
}

function sourceBlob(root: string, head: string, path: string) {
  requireValue(
    /^[A-Za-z0-9_./-]+$/.test(path) && !path.split("/").includes(".."),
    "unsafe source path",
  );
  const entry = rawGit(root, "ls-tree", "-z", head, "--", path).toString();
  const match = /^(100644|100755) blob ([0-9a-f]{40})\t([^\0]+)\0$/.exec(entry);
  requireValue(match && match[3] === path, `normal source blob required: ${path}`);
  const oid = match[2];
  requireValue(
    rawGit(root, "cat-file", "-t", oid).toString().trim() === "blob",
    `blob type required: ${path}`,
  );
  return { oid, bytes: rawGit(root, "cat-file", "blob", oid) };
}

/** Copy exact H source bytes without following working-tree symlinks or replacements. */
export function readRawBlob(root: string, head: string, path: string): Buffer {
  requireHead(root, head);
  return sourceBlob(root, head, path).bytes;
}

/** Read only immutable raw objects: working files and replacement objects are not authority. */
export function derivePublicationPlan(root: string, head: string): PublicationPlan {
  requireHead(root, head);
  const commit = rawGit(root, "cat-file", "commit", head);
  const headers = commit.toString().split("\n\n", 1)[0];
  const tree = /^tree ([0-9a-f]{40})$/m.exec(headers)?.[1];
  requireValue(tree, "raw H tree missing");
  const parents = [...headers.matchAll(/^parent ([0-9a-f]{40})$/gm)];
  requireValue(parents.length === 1, "raw H must have one immutable C parent");
  const parentCut = parents[0][1];
  const authority: PublicationPlan["authority"] = { commitSha256: sha256(commit), tree, blobs: [] };
  const readBytes = (path: string) => {
    const { oid, bytes } = sourceBlob(root, head, path);
    authority.blobs.push({ path, oid, sha256: sha256(bytes) });
    return bytes;
  };
  const read = (path: string) => readBytes(path).toString("utf8");
  const cargo = read("Cargo.toml");
  const packageSection = /^\[workspace\.package\]\s*\n([\s\S]*?)(?=^\[|$(?![\s\S]))/m.exec(
    cargo,
  )?.[1];
  const versions = [...(packageSection ?? "").matchAll(/^version = "([^"]+)"\s*$/gm)];
  requireValue(versions.length === 1, "one workspace version required");
  const version = versions[0][1];
  requireValue(/^0\.(0|[1-9][0-9]*)\.0$/.test(version), "stable minor source version required");
  requireValue(
    packageSection?.includes('repository = "https://github.com/ubugeeei-prod/vize"'),
    "own repository required",
  );

  const workflow = read(".github/workflows/release.yml");
  const commands = workflow
    .split("\n")
    .filter(
      (line) =>
        !line.trimStart().startsWith("#") && /tools\/moon\/cmd\/publish_npm_package\s/.test(line),
    );
  const paths = unique(
    commands.map((line) => {
      const match =
        /^\s*(?:-\s+)?run: moon run --target native tools\/moon\/cmd\/publish_npm_package -- (npm\/[a-zA-Z0-9_/-]+) --provenance\s*$/.exec(
          line,
        );
      requireValue(match, "unsupported npm publish command in H release workflow");
      return match[1];
    }),
    "npm publication paths",
  );
  const npm = paths.map((path) => {
    const manifest = JSON.parse(read(`${path}/package.json`));
    requireValue(
      typeof manifest.name === "string" &&
        /^(@vizejs\/[a-z0-9-]+|vize|oxlint-plugin-vize)$/.test(manifest.name),
      `invalid npm name: ${path}`,
    );
    requireValue(
      manifest.version === version && manifest.private !== true,
      `source npm version/private mismatch: ${path}`,
    );
    return { name: manifest.name, version: manifest.version };
  });
  const nativeIndex = paths.indexOf("npm/native");
  requireValue(nativeIndex >= 0, "native umbrella publication missing");
  const native = JSON.parse(read("npm/native/package.json"));
  const workspace = read("pnpm-workspace.yaml");
  const catalogs = [...workspace.matchAll(/^  native-binaries:\s*\n((?:    [^\n]*\n|\n)*)/gm)];
  requireValue(catalogs.length === 1, "one native binary catalog required");
  const nativeNames = unique(
    [...catalogs[0][1].matchAll(/^    "(@vizejs\/native-[a-z0-9-]+)": "([^"]+)"\s*$/gm)].map(
      (entry) => {
        requireValue(entry[2] === version, `native catalog version mismatch: ${entry[1]}`);
        return entry[1];
      },
    ),
    "native catalog names",
  );
  const catalogLines = catalogs[0][1]
    .split("\n")
    .filter((line) => line.trim() && !line.trimStart().startsWith("#"));
  requireValue(catalogLines.length === nativeNames.length, "unsupported native catalog entry");
  requireValue(
    nativeNames.every(
      (name) => native.optionalDependencies?.[name] === "catalog:native-binaries",
    ) && Object.keys(native.optionalDependencies ?? {}).length === nativeNames.length,
    "native optional dependencies differ from source catalog",
  );
  requireValue(
    /^\s*(?:-\s+)?run: moon run --target native tools\/moon\/cmd\/publish_npm_package_dirs -- npm\/native\/npm --provenance\s*$/m.test(
      workflow,
    ),
    "native target publication missing",
  );
  npm.push(...nativeNames.map((name) => ({ name, version })));
  unique(
    npm.map((target) => target.name),
    "npm names",
  );

  const publisher = read("tools/moon/cmd/publish_crates/main.mbt");
  const lists = [
    ...publisher.matchAll(/^let published_crates : Array\[String\] = \[\n([\s\S]*?)^\]/gm),
  ];
  requireValue(lists.length === 1, "one published_crates source array required");
  const crateNames = unique(
    lists[0][1]
      .split("\n")
      .filter((line) => line.trim())
      .map((line) => {
        const match = /^\s*"([a-z0-9_]+)",\s*$/.exec(line);
        requireValue(match, "unsupported published_crates entry");
        return match[1];
      }),
    "crate names",
  );
  const editor = JSON.parse(read("editors/vscode/package.json"));
  requireValue(
    editor.version === version &&
      /^[a-z0-9-]+$/.test(editor.publisher) &&
      /^[a-z0-9-]+$/.test(editor.name),
    "source editor identity/version mismatch",
  );
  const jsr = deriveJsrPublication(
    version,
    npm,
    readBytes,
    (path) => rawGit(root, "ls-tree", "-z", head, "--", path).length > 0,
  );
  return {
    head,
    parentCut,
    version,
    npm,
    crates: crateNames.map((name) => ({ name, version })),
    editor: { publisher: editor.publisher, name: editor.name, version },
    githubAssets: /^\s+zed-vize-extension\.tar\.gz\s*$/m.test(workflow)
      ? ["zed-vize-extension.tar.gz"]
      : [],
    authority,
    ...(jsr ? { jsr } : {}),
  };
}

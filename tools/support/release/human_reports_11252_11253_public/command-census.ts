import assert from "node:assert/strict";
import { lstatSync, readFileSync, readdirSync, readlinkSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { fileImage, inside } from "../computed_event_custody.ts";
import type { BinPolicy } from "./dependency-bin-policy.ts";
import type { Image, PublicPayloadReceipt } from "./model.ts";
import type { Sink } from "./evidence.ts";
import { bytes, gitBlob, object, sha256, text, wholeJson } from "./value.ts";
import {
  absoluteUnix,
  derivedNodePath,
  pnpmSource,
  renderPnpm12161Unix,
} from "./pnpm-shim-render.ts";

export interface CommandTarget {
  name: string;
  ownerName: string;
  ownerVersion: string;
  ownerDirectory: string;
  publicTarget: Image;
  wrapper: Image & { rawBase64: string };
}
export interface RootCommandCensus {
  root: string;
  rootPackageSha256: string;
  sourceCommit: string;
  sourceInputPolicyGitBlob: string;
  aliases: Array<{
    name: string;
    path: string;
    directory: string;
    link: string | null;
    manifestSha256: string;
  }>;
  targets: CommandTarget[];
  actualWriterRuntimeAttribution: false;
}
interface Candidate {
  name: string;
  owner: PublicPayloadReceipt;
  target: Image;
  runtime: "node" | null;
}

/** The c7 importer caller scans the complete root modules population. This
 * supported branch requires a unique physical owner; it never guesses ties. */
export function collectRootCommandCensus(
  root: string,
  manifest: Buffer,
  packages: readonly PublicPayloadReceipt[],
  policy: BinPolicy,
  sink: Sink,
): RootCommandCensus {
  absoluteUnix(root);
  const policyRaw = bytes(policy.rawBase64);
  assert.equal(gitBlob(policyRaw), policy.gitBlob);
  assert.equal(sha256(policyRaw), policy.sha256);
  const plan = object(wholeJson(policyRaw));
  assert.equal(plan.sourceCommit, pnpmSource.sourceCommit);
  assert.deepEqual(plan.options, {
    extraNodePaths: ["CONSUMER_VIRTUAL_NODE_MODULES"],
    preferSymlinkedExecutables: false,
    projectModulesDirName: null,
    relocatableRoot: "CONSUMER_ROOT",
  });
  assert(readFileSync(join(root, "package.json")).equals(manifest));
  const modules = join(root, "node_modules"),
    aliases: RootCommandCensus["aliases"] = [];
  const candidates = new Map<string, Candidate[]>();
  const visit = (name: string): void => {
    const path = join(modules, name),
      stat = lstatSync(path);
    sink("root-command-package-before-assertion", {
      name,
      path,
      kind: stat.isSymbolicLink() ? "symlink" : stat.isDirectory() ? "directory" : "other",
      link: stat.isSymbolicLink() ? readlinkSync(path) : null,
    });
    assert(stat.isDirectory() || stat.isSymbolicLink());
    const directory = realpathSync(path);
    assert(inside(root, directory));
    const owners = packages.filter((pkg) => pkg.directory === directory);
    assert.equal(owners.length, 1, "root executable candidate lacks authenticated public owner");
    const owner = owners[0]!;
    assert.equal(owner.name, name, "unregistered root package alias");
    const raw = readFileSync(join(directory, "package.json"));
    assert(raw.equals(bytes(owner.manifestRawBase64)));
    assert(
      owner.files.some(
        (f) => f.path === join(directory, "package.json") && f.sha256 === sha256(raw),
      ),
    );
    aliases.push({
      name,
      path,
      directory,
      link: stat.isSymbolicLink() ? readlinkSync(path) : null,
      manifestSha256: sha256(raw),
    });
    const value = object(wholeJson(raw));
    assert.equal(value.name, owner.name);
    assert.equal(value.version, owner.version);
    assert.equal(
      value.bundledDependencies,
      undefined,
      "bundled root bin owners require explicit support",
    );
    assert.equal(
      value.bundleDependencies,
      undefined,
      "bundled root bin owners require explicit support",
    );
    assert.equal(
      object(value.directories ?? {}).bin,
      undefined,
      "directory-discovered bins are unsupported",
    );
    if (value.bin === undefined) return;
    // These issue-bound original providers use object bin maps. Other forms
    // remain UNKNOWN rather than acquiring an invented executable name.
    const bins = object(value.bin);
    for (const [bin, declared] of Object.entries(bins)) {
      assert.match(bin, /^[A-Za-z0-9._-]+$/u);
      const relative = text(declared).replace(/^\.\//u, "");
      assert(relative && relative.split("/").every((p) => p && p !== "." && p !== ".."));
      const targetPath = join(directory, relative);
      assert(inside(directory, targetPath));
      assert.equal(realpathSync(targetPath), targetPath);
      const target = fileImage(targetPath);
      assert(
        owner.files.some(
          (f) => f.path === target.path && f.bytes === target.bytes && f.sha256 === target.sha256,
        ),
      );
      const targetBytes = readFileSync(target.path);
      const newline = targetBytes.indexOf(10);
      const firstLine = targetBytes
        .subarray(0, newline < 0 ? targetBytes.length : newline)
        .toString("utf8");
      const runtime = firstLine === "#!/usr/bin/env node" ? "node" : null;
      assert(
        runtime === "node" || (!firstLine.startsWith("#!") && !/\.[cm]?js$/u.test(relative)),
        "root target interpreter is UNKNOWN",
      );
      const rows = candidates.get(bin) ?? [];
      if (!rows.some((r) => r.owner.directory === owner.directory && r.target.path === target.path))
        rows.push({ name: bin, owner, target, runtime });
      candidates.set(bin, rows);
    }
  };
  for (const name of readdirSync(modules).sort()) {
    if (name.startsWith(".")) continue; // Same non-package population as c7's scanner.
    if (name.startsWith("@")) {
      const scope = join(modules, name);
      assert(inside(root, realpathSync(scope)) && lstatSync(scope).isDirectory());
      for (const child of readdirSync(scope).sort())
        if (!child.startsWith(".")) visit(name + "/" + child);
    } else visit(name);
  }
  const rootManifest = object(wholeJson(manifest));
  for (const category of ["dependencies", "devDependencies", "optionalDependencies"])
    for (const name of Object.keys(object(rootManifest[category] ?? {})))
      assert(
        aliases.some((row) => row.name === name),
        "original root provider is missing",
      );
  const binDirectory = join(modules, ".bin");
  assert.equal(realpathSync(binDirectory), binDirectory);
  const actualNames = readdirSync(binDirectory).sort();
  sink("complete-root-command-population-before-assertion", {
    aliases,
    candidates: [...candidates].map(([name, rows]) => ({
      name,
      owners: rows.map((r) => ({
        name: r.owner.name,
        version: r.owner.version,
        directory: r.owner.directory,
        target: r.target,
      })),
    })),
    actualBinNames: actualNames,
    sourceInputPolicyGitBlob: policy.gitBlob,
    completeInstalledFileSetQualified: false,
  });
  assert.deepEqual(
    actualNames,
    [...candidates.keys()].sort(),
    "unknown/missing root executable refuses execution",
  );
  const targets = [...candidates]
    .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
    .map(([name, rows]) => {
      assert.equal(rows.length, 1, "root bin owner/winner is ambiguous");
      const row = rows[0]!,
        wrapperPath = join(binDirectory, name),
        stat = lstatSync(wrapperPath);
      assert(stat.isFile() && !stat.isSymbolicLink() && (stat.mode & 0o111) !== 0);
      assert.equal(realpathSync(wrapperPath), wrapperPath);
      const expected = renderPnpm12161Unix({
        target: row.target.path,
        shimPath: wrapperPath,
        runtime: row.runtime,
        nodePath: derivedNodePath(row.owner.directory, [
          join(root, "node_modules/.pnpm/node_modules"),
        ]),
        relocatableRoot: root,
      });
      const actual = readFileSync(wrapperPath);
      sink("mandatory-root-wrapper-before-assertion", {
        name,
        path: wrapperPath,
        owner: row.owner.name,
        publicTarget: row.target,
        expectedBytes: expected.length,
        expectedSha256: sha256(expected),
        actualBytes: actual.length,
        actualSha256: sha256(actual),
        rawBase64: actual.equals(expected) ? actual.toString("base64") : null,
      });
      assert(actual.equals(expected), "mandatory root wrapper is not wholly source-conformant");
      return {
        name,
        ownerName: row.owner.name,
        ownerVersion: row.owner.version,
        ownerDirectory: row.owner.directory,
        publicTarget: row.target,
        wrapper: {
          path: wrapperPath,
          bytes: actual.length,
          sha256: sha256(actual),
          rawBase64: actual.toString("base64"),
        },
      };
    });
  return {
    root,
    rootPackageSha256: sha256(manifest),
    sourceCommit: pnpmSource.sourceCommit,
    sourceInputPolicyGitBlob: policy.gitBlob,
    aliases,
    targets,
    actualWriterRuntimeAttribution: false,
  };
}

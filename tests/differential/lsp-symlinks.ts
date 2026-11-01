import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { sha256 } from "./harness.mjs";
import { safeRuntimePath } from "./lsp-manifest.ts";

export type LspSymlink = { path: string; target: string; sha256: string };
export type LspSymlinkObservation = {
  path: string;
  isSymbolicLink: boolean;
  targetIsRegularFile: boolean;
  linkTarget: string | null;
  canonicalTarget: string;
  sha256: string | null;
};
const overlaps = (left: string, right: string) =>
  left === right || left.startsWith(`${right}/`) || right.startsWith(`${left}/`);

/** Links name only regular inputs that this same workspace materializes. */
export function lspSymlinks(
  value: unknown,
  files: readonly { path: string; sha256: string }[],
): LspSymlink[] | undefined {
  if (value === undefined) return undefined;
  assert(Array.isArray(value) && value.length > 0, "nonempty explicit symlink layout required");
  const entries: unknown[] = value;
  const links: LspSymlink[] = [];
  for (const entry of entries) {
    assert(entry !== null && typeof entry === "object" && !Array.isArray(entry));
    assert.deepEqual(Object.keys(entry).sort(), ["path", "target"]);
    const record = entry as Record<string, unknown>;
    const linkPath = safeRuntimePath(record.path);
    const target = safeRuntimePath(record.target);
    const input = files.find((file) => file.path === target);
    assert(input, "symlink target must be a pinned materialized regular input");
    assert(!files.some((file) => overlaps(linkPath, file.path)), "symlink collides with input");
    assert(!links.some((link) => overlaps(link.path, linkPath)), "symlinks overlap or nest");
    const parent = path.posix.dirname(linkPath);
    assert(
      parent === "." || files.some((file) => file.path.startsWith(`${parent}/`)),
      "symlink parent must be a materialized input directory",
    );
    links.push({ path: linkPath, target, sha256: input.sha256 });
  }
  return links;
}

/** Called only after every pinned regular file and configuration carrier exists. */
export function materializeLspSymlinks(root: string, links: readonly LspSymlink[] | undefined) {
  for (const link of links ?? []) {
    const target = path.join(root, safeRuntimePath(link.target));
    const destination = path.join(root, safeRuntimePath(link.path));
    const stat = fs.lstatSync(target);
    assert(stat.isFile() && !stat.isSymbolicLink(), "pinned target must be a regular file");
    assert.equal(fs.realpathSync(target), target, "target ancestors must belong to the workspace");
    assert.equal(sha256(fs.readFileSync(target)), link.sha256);
    assert.equal(
      fs.lstatSync(destination, { throwIfNoEntry: false }),
      undefined,
      "symlink destination must be absent",
    );
    const parent = path.dirname(destination);
    assert.equal(
      fs.realpathSync(parent),
      parent,
      "symlink destination ancestors must belong to the workspace",
    );
    assert(fs.lstatSync(parent).isDirectory(), "materialized destination directory required");
    fs.symlinkSync(target, destination, "file");
  }
}

/** Preserve actual link identity before any complete-observation verdict. */
export function observeLspSymlinks(
  root: string,
  links: readonly LspSymlink[] | undefined,
): LspSymlinkObservation[] | undefined {
  return links?.map((link) => {
    const file = path.join(root, safeRuntimePath(link.path));
    const isSymbolicLink = fs.lstatSync(file).isSymbolicLink();
    const linkTarget = isSymbolicLink ? fs.readlinkSync(file) : null;
    const canonicalTarget = fs.realpathSync(file);
    const target = path.join(root, safeRuntimePath(link.target));
    const stat = fs.lstatSync(target);
    const targetIsRegularFile = stat.isFile() && !stat.isSymbolicLink();
    // A redirected link must never make this observer read an external file.
    const owned = targetIsRegularFile && canonicalTarget === target;
    return {
      path: link.path,
      isSymbolicLink,
      targetIsRegularFile,
      linkTarget,
      canonicalTarget,
      sha256: owned ? sha256(fs.readFileSync(file)) : null,
    };
  });
}

export function expectedLspSymlinks(
  workspaceUri: string | undefined,
  links: readonly LspSymlink[],
): LspSymlinkObservation[] {
  assert(typeof workspaceUri === "string");
  assert.equal(new URL(workspaceUri).protocol, "file:");
  const root = fileURLToPath(workspaceUri);
  assert.equal(pathToFileURL(root).href, workspaceUri, "canonical source workspace URI required");
  return links.map((link) => ({
    path: link.path,
    isSymbolicLink: true,
    targetIsRegularFile: true,
    linkTarget: path.join(root, safeRuntimePath(link.target)),
    canonicalTarget: path.join(root, safeRuntimePath(link.target)),
    sha256: link.sha256,
  }));
}

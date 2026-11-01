import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { vueConfigFixtureSource } from "../tooling/support/vue-config-fixture.ts";
import { sha256 } from "./harness.mjs";
import { safeRuntimePath } from "./lsp-manifest.ts";
import type { LspFixture } from "./lsp-types.ts";
import {
  expectedLspSymlinks,
  lspSymlinks,
  type LspSymlink,
  type LspSymlinkObservation,
} from "./lsp-symlinks.ts";

export type WorkspaceFileDigest = { path: string; sha256: string };
export type LspWorkspaceMapping = {
  kind: "native-json-to-canonical-vite-v1";
  originalFiles: WorkspaceFileDigest[];
  files: WorkspaceFileDigest[];
  configurations: {
    originalPath: string;
    originalBase64: string;
    originalSha256: string;
    evidencePath: string;
    carrierPath: string;
    carrierBase64: string;
    carrierSha256: string;
  }[];
  symlinks?: LspSymlink[];
};
export type LspWorkspaceEvidence = {
  mapping: LspWorkspaceMapping;
  before?: WorkspaceFileDigest[];
  after?: WorkspaceFileDigest[];
  symlinksBefore?: LspSymlinkObservation[];
  symlinksAfter?: LspSymlinkObservation[];
  symlinkWorkspaceUri?: string;
};

type WorkspaceFile = { runtimePath: string; bytes: Buffer };
const digests = (files: WorkspaceFile[]): WorkspaceFileDigest[] =>
  files.map((file) => ({ path: safeRuntimePath(file.runtimePath), sha256: sha256(file.bytes) }));

/** Keep immutable corpus files separate from the current source-built carrier. */
export function lspWorkspace(fixture: LspFixture) {
  const originalFiles = digests(fixture.files);
  const declared = fixture.data.files ?? (fixture.data.input ? [fixture.data.input] : []);
  assert.deepEqual(
    fixture.inputs.files,
    declared.map((file) => ({ path: file.source, sha256: file.sha256 })),
    "declared immutable LSP inputs changed",
  );
  assert.deepEqual(
    originalFiles,
    declared.map((file) => ({ path: file.runtimePath, sha256: file.sha256 })),
    "complete original LSP workspace inputs changed",
  );
  const files: WorkspaceFile[] = [];
  const carriers: WorkspaceFile[] = [];
  const configurations: LspWorkspaceMapping["configurations"] = [];
  for (const file of fixture.files) {
    const basename = path.posix.basename(file.runtimePath);
    if (!basename.startsWith("vize.config.")) {
      files.push(file);
      continue;
    }
    assert.equal(
      basename,
      "vize.config.json",
      "only the declared native JSON corpus carrier can migrate",
    );
    const evidencePath = `${file.runtimePath}.fixture`;
    const carrierPath = path.posix.join(path.posix.dirname(file.runtimePath), "vite.config.mjs");
    const carrier = Buffer.from(vueConfigFixtureSource(file.bytes));
    files.push({ runtimePath: evidencePath, bytes: file.bytes });
    carriers.push({ runtimePath: carrierPath, bytes: carrier });
    configurations.push({
      originalPath: file.runtimePath,
      originalBase64: file.bytes.toString("base64"),
      originalSha256: sha256(file.bytes),
      evidencePath,
      carrierPath,
      carrierBase64: carrier.toString("base64"),
      carrierSha256: sha256(carrier),
    });
  }
  files.push(...carriers);
  assert.equal(
    new Set(files.map((file) => file.runtimePath)).size,
    files.length,
    "LSP carrier collides with an original input",
  );
  const mapping: LspWorkspaceMapping = {
    kind: "native-json-to-canonical-vite-v1",
    originalFiles,
    files: digests(files),
    configurations,
  };
  const symlinks = lspSymlinks(fixture.data.workspaceSymlinks, mapping.files);
  if (symlinks !== undefined) mapping.symlinks = symlinks;
  return { files, mapping };
}

/** Observe the files actually used by the server, before and after its session. */
export function observeLspWorkspace(
  root: string,
  mapping: LspWorkspaceMapping,
): WorkspaceFileDigest[] {
  return mapping.files.map((file) => ({
    path: file.path,
    sha256: observeFile(root, file.path, mapping.symlinks !== undefined),
  }));
}

function observeFile(root: string, relative: string, sourceOwned: boolean): string {
  const file = path.join(root, safeRuntimePath(relative));
  if (sourceOwned) {
    const stat = fs.lstatSync(file);
    assert(stat.isFile() && !stat.isSymbolicLink(), "owned regular input changed type");
    assert.equal(fs.realpathSync(file), file, "owned input escaped its source workspace");
  }
  return sha256(fs.readFileSync(file));
}

export function validateLspWorkspace(
  fixture: LspFixture,
  evidence: LspWorkspaceEvidence | undefined,
  completed: boolean,
  workspaceUri?: string,
): void {
  assert(evidence, "explicit LSP workspace transport evidence is required");
  const expected = lspWorkspace(fixture).mapping;
  assert.deepEqual(evidence.mapping, expected, "LSP configuration transport mapping changed");
  if (expected.symlinks === undefined) {
    assert.equal(evidence.symlinkWorkspaceUri, undefined, "unrequested link workspace changed");
  } else if (completed) {
    assert.equal(
      evidence.symlinkWorkspaceUri,
      workspaceUri,
      "link workspace must equal the actual wire root",
    );
  }
  for (const phase of ["before", "after"] as const) {
    if (completed)
      assert(Array.isArray(evidence[phase]), `actual ${phase} workspace observation is required`);
    if (evidence[phase] !== undefined)
      assert.deepEqual(evidence[phase], expected.files, `actual ${phase} workspace inputs changed`);
  }
  for (const phase of ["symlinksBefore", "symlinksAfter"] as const) {
    if (expected.symlinks === undefined) {
      assert.equal(evidence[phase], undefined, "unrequested links must not enter old profiles");
    } else if (completed || evidence[phase] !== undefined) {
      assert.deepEqual(
        evidence[phase],
        expectedLspSymlinks(evidence.symlinkWorkspaceUri, expected.symlinks),
      );
    }
  }
}

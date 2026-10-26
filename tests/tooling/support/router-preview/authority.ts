import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const fixtureDirectory = fileURLToPath(
  new URL("../../../fixtures/typechecker/experimental-router-preview-593d7ad/", import.meta.url),
);
export const previewPage = "src/pages/products/[id=int].vue";
export const hash = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
export type FileIdentity = { path: string; bytes: number; sha256: string };
export type SourceAuthority = {
  schema: "vize.router-preview-source";
  version: 1;
  provider: {
    repository: "vuejs/router";
    revision: string;
    tree: string;
    packageName: "vue-router";
    packageVersion: "5.4.0";
    publishedRegistryArchive: false;
  };
  build: { packageManager: string; install: string[]; command: string[]; config: string };
  capabilities: {
    requiredExperimentalTypeExports: string[];
    absentExperimentalExports: string[];
    optIn: string;
  };
  sourceInputs: Array<FileIdentity & { gitBlob: string }>;
  qualification: {
    state: "discovery" | "frozen";
    sourceBuiltPackageIsNotPublishedNpmArchive: true;
    automaticProjection: "not-qualified";
  };
};

export function readAuthority() {
  const bytes = fs.readFileSync(path.join(fixtureDirectory, "source.json"));
  const authority = JSON.parse(bytes.toString("utf8")) as SourceAuthority;
  assert.equal(authority.schema, "vize.router-preview-source");
  assert.equal(authority.version, 1);
  assert.deepEqual(authority.provider, {
    repository: "vuejs/router",
    revision: "593d7ad9ff99e95c24be28e962717df0fb2be763",
    tree: "885476969881154b639ea4e2a0c736ec6fa1796a",
    packageName: "vue-router",
    packageVersion: "5.4.0",
    publishedRegistryArchive: false,
  });
  assert.equal(authority.build.packageManager, "pnpm@12.14.0");
  assert.deepEqual(authority.capabilities.requiredExperimentalTypeExports, [
    "_RoutePropsForFilePath",
  ]);
  assert.deepEqual(authority.capabilities.absentExperimentalExports, ["useRoute"]);
  return { authority, bytes, sha256: hash(bytes) };
}

export function relativeMember(value: string) {
  assert.ok(value && !path.isAbsolute(value));
  assert.equal(
    value.split(/[\\/]/u).some((part) => !part || part === "." || part === ".."),
    false,
  );
  return value;
}

export function fileIdentity(root: string, relative: string): FileIdentity {
  const file = path.join(root, relativeMember(relative));
  assert.equal(fs.lstatSync(file).isFile(), true, relative);
  const bytes = fs.readFileSync(file);
  return { path: relative, bytes: bytes.length, sha256: hash(bytes) };
}

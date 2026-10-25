import assert from "node:assert/strict";
import path from "node:path";
import { parseAllDocuments } from "yaml";

/** Verify every native resolution, including pnpm 12's manager/project documents. */
export function sourceNativeResolutions(text, directory, tarball, integrity) {
  const documents = parseAllDocuments(text).map((document) => {
    assert.deepEqual(document.errors, []);
    return document.toJS();
  });
  const projects = documents.filter(
    (document) => document?.importers?.["."]?.dependencies?.["@vizejs/turbopack-plugin"],
  );
  assert.equal(projects.length, 1, "one installed project must own the packed adapter");
  function selectedFile(specifier) {
    assert.equal(typeof specifier, "string");
    assert.ok(specifier.startsWith("file:"));
    assert.equal(path.resolve(directory, specifier.slice(5)), tarball);
  }
  selectedFile(projects[0].overrides?.["@vizejs/native"]);
  const resolutions = documents.flatMap((document) =>
    Object.entries(document?.packages ?? {}).filter(([key]) => key.startsWith("@vizejs/native@")),
  );
  assert.ok(resolutions.length);
  for (const [key, entry] of resolutions) {
    assert.ok(key.includes("file:"), "a registry native image is not source qualification");
    assert.equal(entry.resolution.integrity, integrity);
    selectedFile(entry.resolution.tarball);
  }
  return resolutions;
}

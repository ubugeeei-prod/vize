import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import type { VizePublicRegistryInstallAuthority } from "./authority-schema.ts";
import { exactFile } from "./custody.ts";

/** Recheck every public package member, including routing JS and bundled runtime files. */
export function recheckPublicPayload(authority: VizePublicRegistryInstallAuthority): void {
  exactFile(authority.payloadManifestPath, authority.payloadManifestSha256);
  const manifest = JSON.parse(fs.readFileSync(authority.payloadManifestPath, "utf8"));
  assert.equal(manifest.schema, "vize-public-registry-payload-files-v1");
  assert.equal(manifest.installRoot, authority.installRoot);
  assert.deepEqual(manifest.source, authority.source);
  assert.ok(Array.isArray(manifest.packages));
  const wanted = new Map<string, { version: string; resolved: string; integrity: string }>(
    authority.registry.map((record) => [record.name, record]),
  );
  wanted.set(authority.bundledCorsa.packageName, authority.bundledCorsa);
  for (const entry of manifest.packages) {
    const expected = wanted.get(entry.name);
    assert.ok(expected, "only the reviewed public package payloads are admitted");
    wanted.delete(entry.name);
    assert.equal(entry.version, expected.version);
    assert.equal(entry.resolved, expected.resolved);
    assert.equal(entry.integrity, expected.integrity);
    assert.equal(entry.allInstalledFileBytesEqualPublicArchive, true);
    assert.match(entry.tarballSHA512, /^[a-f0-9]{128}$/u);
    if (entry.name === authority.bundledCorsa.packageName)
      assert.deepEqual(entry.source, { thirdPartyRegistry: true });
    else assert.deepEqual(entry.source, { H: authority.source.H, R: authority.source.R });
    const packageRoot = path.join(authority.installRoot, "node_modules", entry.name);
    assert.equal(fs.realpathSync(packageRoot), packageRoot);
    assert.ok(Array.isArray(entry.files) && entry.files.length > 0);
    const expectedFiles = new Set<string>();
    for (const file of entry.files) {
      assert.ok(!expectedFiles.has(file.path), "duplicate authenticated public file");
      expectedFiles.add(file.path);
      exactFile(file.path, file.sha256, packageRoot);
      assert.equal(fs.statSync(file.path).size, file.bytes);
    }
    const actualFiles: string[] = [];
    const visit = (directory: string) => {
      for (const name of fs.readdirSync(directory)) {
        const file = path.join(directory, name);
        const stat = fs.lstatSync(file);
        assert.equal(
          stat.isSymbolicLink(),
          false,
          "public package must not redirect into source or unowned content",
        );
        if (stat.isDirectory()) visit(file);
        else {
          assert.ok(stat.isFile());
          actualFiles.push(file);
        }
      }
    };
    visit(packageRoot);
    assert.deepEqual(
      actualFiles.toSorted(),
      [...expectedFiles].toSorted(),
      "the whole installed public file set must remain exact",
    );
  }
  assert.equal(wanted.size, 0, "every reviewed registry package payload is mandatory");
}

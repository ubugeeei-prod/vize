import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  type PinnedFixtureWorkspace,
  repoRoot,
  symlinkDirectory,
} from "../../../_helpers/realworld-patch.ts";

const here = path.dirname(fileURLToPath(import.meta.url));
const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

function packageIdentity(directory: string, name: string, version: string) {
  const physical = fs.realpathSync(directory);
  const bytes = fs.readFileSync(path.join(physical, "package.json"));
  const manifest = JSON.parse(bytes.toString("utf8")) as { name: string; version: string };
  assert.equal(manifest.name, name);
  assert.equal(manifest.version, version);
  return { directory, physical, name, version, manifestSha256: sha256(bytes) };
}

/** Whole original official archive plus explicit already-locked plugin dependencies. */
export function preparePublishedProvider(fixture: PinnedFixtureWorkspace) {
  const capture = path.join(repoRoot, "target/differential/router-page-types", randomUUID());
  fs.mkdirSync(capture, { recursive: true });
  fs.writeFileSync(
    path.join(capture, "preparation.json"),
    `${JSON.stringify(
      {
        fixture: fixture.entry,
        frozenLockSha256: sha256(fs.readFileSync(path.join(repoRoot, "pnpm-lock.yaml"))),
      },
      null,
      2,
    )}\n`,
  );
  fs.copyFileSync(
    path.join(repoRoot, "target/ci/vize.differential-build.json"),
    path.join(capture, "vize.differential-build.json"),
  );
  const destination = fixture.resolve("router-provider");
  const result = execFileSync("python3", [path.join(here, "provider.py"), destination], {
    encoding: "utf8",
    timeout: 60_000,
    maxBuffer: 8 * 1024 * 1024,
  });
  const receipt = JSON.parse(result) as {
    archiveSha256: string;
    publishedSource: string;
    files: Array<{ path: string; sha256: string; bytes: number }>;
  };
  for (const file of ["receipt.json", "vue-router-5.1.0.tgz"]) {
    fs.copyFileSync(path.join(destination, file), path.join(capture, file));
  }
  assert.equal(
    receipt.archiveSha256,
    "5c0bf884438b9c58b1e926663e07572c80c5dcc8d0ddd57a32943a0161d8ee5f",
  );
  assert.equal(receipt.publishedSource, "c0e3226dabccd7596b996ce851386997ea2d3cca");
  assert.equal(receipt.files.length, 70);
  const provider = path.join(destination, "package");
  const providerIdentity = packageIdentity(provider, "vue-router", "5.1.0");
  for (const file of receipt.files) {
    const body = fs.readFileSync(path.join(provider, file.path));
    assert.equal(body.length, file.bytes);
    assert.equal(sha256(body), file.sha256);
  }
  const plugin = fs.readFileSync(path.join(provider, "dist/volar/sfc-typed-router.cjs"), "utf8");
  const requires = [...plugin.matchAll(/require\(["']([^"']+)["']\)/g)].map((match) => match[1]);
  assert.deepEqual(requires, ["muggle-string", "pathe"]);
  const dependencies = [
    packageIdentity(
      path.join(repoRoot, "node_modules/.pnpm/muggle-string@0.4.1/node_modules/muggle-string"),
      "muggle-string",
      "0.4.1",
    ),
    packageIdentity(
      path.join(repoRoot, "node_modules/.pnpm/pathe@2.0.3/node_modules/pathe"),
      "pathe",
      "2.0.3",
    ),
  ];
  for (const dependency of dependencies) {
    symlinkDirectory(dependency.physical, path.join(provider, "node_modules", dependency.name));
  }
  // Use the existing stable Vue catalog importer, not the installed beta peer.
  const vue = packageIdentity(path.join(repoRoot, "docs/node_modules/vue"), "vue", "3.5.35");
  symlinkDirectory(vue.physical, fixture.resolve("node_modules/vue"));
  symlinkDirectory(
    path.join(path.dirname(vue.physical), "@vue"),
    fixture.resolve("node_modules/@vue"),
  );
  symlinkDirectory(provider, fixture.resolve("node_modules/vue-router"));
  const evidence = {
    providerIdentity,
    dependencies,
    vue,
    frozenLockSha256: sha256(fs.readFileSync(path.join(repoRoot, "pnpm-lock.yaml"))),
    archive: receipt,
  };
  fs.writeFileSync(
    path.join(capture, "dependencies.json"),
    `${JSON.stringify(evidence, null, 2)}\n`,
  );
  let observation = 0;
  return {
    capture,
    evidence,
    record(value: unknown) {
      fs.writeFileSync(
        path.join(capture, `${String(++observation).padStart(3, "0")}.json`),
        `${JSON.stringify(value, null, 2)}\n`,
      );
    },
  };
}

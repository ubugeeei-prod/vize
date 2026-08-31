import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  cliHandoffNames,
  createCliPublishHandoff,
  formatCliHandoffSummary,
} from "../../legacy-tools/github/npm-bootstrap-handoff.mjs";

const packageName = "@vizejs/nuxt-lint-config";
const version = "1.2.3";
const source = {
  sourceArtifactName: "release-package-nuxt-lint-config",
  releaseRunId: "123456789",
  releaseTagName: `v${version}`,
  releaseTagSha: "a".repeat(40),
};

function writePackage(root: string): string {
  const packagePath = path.join(root, "package");
  fs.mkdirSync(path.join(packagePath, "dist"), { recursive: true });
  fs.writeFileSync(path.join(packagePath, "dist", "index.mjs"), "export const ok = true;\n");
  fs.writeFileSync(
    path.join(packagePath, "dist", "index.d.mts"),
    "export declare const ok: true;\n",
  );
  fs.writeFileSync(
    path.join(packagePath, "package.json"),
    `${JSON.stringify(
      {
        name: packageName,
        version,
        files: ["dist"],
        type: "module",
        main: "./dist/index.mjs",
        types: "./dist/index.d.mts",
        publishConfig: { access: "public" },
      },
      null,
      2,
    )}\n`,
  );
  return packagePath;
}

test("npm CLI handoff names bind the package identity and version", () => {
  assert.deepEqual(cliHandoffNames(packageName, version), {
    artifactName: "npm-cli-first-publish-vizejs-nuxt-lint-config-1.2.3",
    tarballName: "vizejs-nuxt-lint-config-1.2.3.tgz",
  });
  assert.throws(() => cliHandoffNames("nuxt-lint-config", version), /scoped npm package/);
  assert.throws(() => cliHandoffNames(packageName, "1.2.x"), /Invalid package version/);
});

test("npm CLI handoff reproduces one exact tarball with a verified SHA-512", () => {
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-npm-handoff-test-"));
  try {
    const packagePath = writePackage(tempRoot);
    const firstOutput = path.join(tempRoot, "first-output");
    const secondOutput = path.join(tempRoot, "second-output");
    const first = createCliPublishHandoff({
      packagePath,
      outputPath: firstOutput,
      expectedName: packageName,
      expectedVersion: version,
      ...source,
    });
    const second = createCliPublishHandoff({
      packagePath,
      outputPath: secondOutput,
      expectedName: packageName,
      expectedVersion: version,
      ...source,
    });

    const tarballName = "vizejs-nuxt-lint-config-1.2.3.tgz";
    const firstTarball = fs.readFileSync(path.join(firstOutput, tarballName));
    const secondTarball = fs.readFileSync(path.join(secondOutput, tarballName));
    const expectedSha512 = createHash("sha512").update(firstTarball).digest("hex");
    assert.ok(firstTarball.equals(secondTarball));
    assert.equal(first.tarball.sha512, expectedSha512);
    assert.deepEqual(first, second);
    assert.equal(
      fs.readFileSync(path.join(firstOutput, "SHA512SUMS"), "utf8"),
      `${expectedSha512}  ${tarballName}\n`,
    );
    assert.deepEqual(
      JSON.parse(fs.readFileSync(path.join(firstOutput, "npm-publish-handoff.json"), "utf8")),
      first,
    );
    assert.equal(first.publish.command, `npm publish ./${tarballName} --access public`);
    assert.equal(
      first.publish.trustCommand,
      "npm trust github @vizejs/nuxt-lint-config --file release.yml --repo ubugeeei-prod/vize --env npm --allow-publish --yes",
    );

    const summary = formatCliHandoffSummary(first);
    assert.match(summary, /did not request an OIDC token and did not publish/);
    assert.match(summary, /does not carry GitHub Actions OIDC provenance/);
    assert.match(summary, /npm trust github @vizejs\/nuxt-lint-config/);
    assert.doesNotMatch(summary, /--provenance|NPM_TOKEN/);
  } finally {
    fs.rmSync(tempRoot, { force: true, recursive: true });
  }
});

test("npm CLI handoff rejects artifact identity and public-access drift", () => {
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-npm-handoff-test-"));
  try {
    const packagePath = writePackage(tempRoot);
    assert.throws(
      () =>
        createCliPublishHandoff({
          packagePath,
          outputPath: path.join(tempRoot, "wrong-version"),
          expectedName: packageName,
          expectedVersion: "1.2.4",
          ...source,
        }),
      /Release tag .* must match package version/,
    );

    const manifestPath = path.join(packagePath, "package.json");
    const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
    delete manifest.publishConfig;
    fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
    assert.throws(
      () =>
        createCliPublishHandoff({
          packagePath,
          outputPath: path.join(tempRoot, "private-package"),
          expectedName: packageName,
          expectedVersion: version,
          ...source,
        }),
      /publishConfig\.access as public/,
    );
  } finally {
    fs.rmSync(tempRoot, { force: true, recursive: true });
  }
});

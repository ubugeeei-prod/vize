import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import { expectedBuildIdentity, sourceRevision, validateBuildReceipt } from "./build-receipt.ts";
import { sha256 } from "./sha256.ts";
import { assertActorIdentity } from "../../tools/support/release/public_acceptance/a11y.ts";
import { publicInputs } from "../../tools/support/release/public_acceptance/inputs.ts";
import { assertInstalledLsp } from "../../tools/support/release/public_acceptance/lsp.ts";

/** Official JS references share the existing fresh consumer, never workspace Vize addons. */
export function formatterOraclePackage(moduleUrl: string) {
  if (process.env.PUBLIC_OUTPUT || process.env.PUBLIC_CONSUMER) {
    assert.ok(process.env.PUBLIC_OUTPUT && process.env.PUBLIC_CONSUMER);
    const consumer = fs.realpathSync(process.env.PUBLIC_CONSUMER);
    const manifest = path.join(consumer, "package.json");
    const installed = JSON.parse(fs.readFileSync(manifest, "utf8"));
    assert.equal(installed.dependencies.vue, "3.5.35");
    assert.equal(installed.dependencies["happy-dom"], "20.11.2");
    const require = createRequire(manifest);
    const vue = createRequire(require.resolve("vue"));
    const renderer = createRequire(require.resolve("vue/server-renderer"));
    for (const [owner, name] of [
      [require, "vue"],
      [require, "vue/compiler-sfc"],
      [vue, "@vue/compiler-dom"],
      [renderer, "@vue/compiler-ssr"],
      [require, "vue/server-renderer"],
      [require, "happy-dom"],
    ] as const) {
      const entry = fs.realpathSync(owner.resolve(name));
      assert.ok(
        entry.startsWith(path.join(consumer, "node_modules") + path.sep),
        `official JS oracle provider must come from the public consumer: ${name}`,
      );
    }
    return manifest;
  }
  return fileURLToPath(new URL("../../../npm/ui/package.json", moduleUrl));
}

/** Reuse the existing public installation; never acquire or build another executable. */
export async function formatterCliTarget(root: string, reportName: string) {
  const published = Boolean(process.env.PUBLIC_OUTPUT || process.env.PUBLIC_CONSUMER);
  let binary: string, receipt: unknown, expectedSha: string;
  if (published) {
    const output = fs.realpathSync(process.env.PUBLIC_OUTPUT!);
    const consumer = fs.realpathSync(process.env.PUBLIC_CONSUMER!);
    const load = (name: string) => JSON.parse(fs.readFileSync(path.join(output, name), "utf8"));
    const preparation = load("preparation.json");
    const publication = load("publication.json");
    const source = publicInputs(process.env);
    const actor = assertActorIdentity(sourceRevision(root), process.env);
    assert.deepEqual(actor, preparation.actorPolicy.identity);
    assert.deepEqual(preparation.source, source);
    assert.equal(preparation.consumer, consumer);
    assert.equal(publication.success, true);
    assert.equal(source.recoveryRun, undefined, "normal published consumer required");
    for (const key of ["cut", "head", "tag", "sourcePr", "run"] as const)
      assert.equal(publication.source[key], source[key]);
    assert.equal(preparation.version, source.tag.slice(1));
    const installed = await assertInstalledLsp(output);
    assert.equal(installed.cli.releaseId, publication.github.releaseId);
    assert.equal(installed.cli.tag, source.tag);
    assert.equal(installed.cli.unpack.members.length, 1);
    assert.equal(installed.cli.executableSha256, installed.cli.unpack.members[0].sha256);
    assert.equal(installed.cli.probe.status, 0);
    assert.equal(installed.cli.probe.signal, null);
    assert.equal(installed.cli.probe.error, undefined);
    assert.equal(
      installed.cli.probe.stdoutBase64,
      Buffer.from(`vize ${preparation.version}\n`).toString("base64"),
    );
    assert.equal(installed.cli.probe.stderrBase64, "");
    binary = installed.cli.executable;
    expectedSha = installed.cli.executableSha256;
    receipt = { source, actor, cli: installed.cli, npmLockSha256: installed.npmLockSha256 };
    assert.ok(!fs.existsSync(path.join(output, reportName)), "preserve earlier formatter evidence");
  } else {
    const identity = expectedBuildIdentity(root);
    const build = JSON.parse(
      fs.readFileSync(path.join(root, `${identity.binaryPath}.differential-build.json`), "utf8"),
    );
    validateBuildReceipt(build, identity);
    binary = path.join(root, identity.binaryPath);
    expectedSha = identity.binarySha256;
    receipt = build;
  }
  const assertUnchanged = () => {
    assert.equal(fs.realpathSync(binary), binary, "explicit physical formatter executable required");
    assert.ok(fs.lstatSync(binary).isFile());
    assert.equal(sha256(fs.readFileSync(binary)), expectedSha, "formatter executable changed");
  };
  assertUnchanged();
  return {
    binary,
    receipt,
    kind: published ? "installed-public-cli" : "source-built-cli",
    reportPath: path.join(
      published ? process.env.PUBLIC_OUTPUT! : path.join(root, "target/differential"),
      reportName,
    ),
    assertUnchanged,
  };
}

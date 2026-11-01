import assert from "node:assert/strict";
import { copyFileSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { archiveComparison, recheckFiles } from "../public_install/archive.ts";
import { captureDeclarationGraph, fileImage } from "../computed_event_custody.ts";
import { capture, cleanEnvironment } from "./capture.ts";
import { manifestBytes, validateInstallation } from "./installation.ts";
import { publicArchive } from "./network.ts";
import type { Fixture, InstallReceipt, Recipe, ReleaseAuthority } from "./model.ts";
import { array, bytes, object, sha256, text, wholeJson } from "./value.ts";
import { discard, type Sink } from "./evidence.ts";
export async function installPublic(
  root: string,
  kind: InstallReceipt["kind"],
  fixture: Fixture,
  recipe: Recipe,
  authority: ReleaseAuthority,
  pnpmPath: string,
  output: string,
  archiveCache: Map<string, Buffer>,
  sink: Sink = discard,
): Promise<InstallReceipt> {
  assert.equal(process.version, recipe.node);
  assert.equal(process.platform, recipe.platform);
  assert.equal(process.arch, recipe.arch);
  mkdirSync(root);
  root = realpathSync(root);
  const env = cleanEnvironment(),
    node = fileImage(process.execPath),
    pnpm = fileImage(realpathSync(pnpmPath));
  const checkVersion = capture(pnpm.path, ["--version"], root, env, output, `${kind}-pnpm-version`);
  assert.equal(checkVersion.status, 0);
  assert.equal(checkVersion.stderrBase64, "");
  assert.equal(bytes(checkVersion.stdoutBase64).toString(), recipe.pnpm + "\n");
  // Explicit empty user/global configuration bounds registry resolution. The
  // reporter's unobserved machine settings and lockfile are never invented.
  const user = join(output, `${kind}-user.npmrc`),
    global = join(output, `${kind}-global.npmrc`);
  writeFileSync(user, "", { flag: "wx" });
  writeFileSync(global, "", { flag: "wx" });
  env.npm_config_userconfig = user;
  env.npm_config_globalconfig = global;
  env.npm_config_registry = "https://registry.npmjs.org";
  const manifest = manifestBytes(fixture, authority.version, kind === "tsrs-supplement");
  for (const file of fixture.files) {
    const target = join(root, file.path);
    mkdirSync(join(target, ".."), { recursive: true });
    writeFileSync(target, file.path === "package.json" ? manifest : bytes(file.base64), {
      flag: "wx",
    });
  }
  const installed = capture(pnpm.path, ["install"], root, env, output, `${kind}-install`);
  assert.equal(installed.error, null);
  assert.equal(installed.signal, null);
  assert.equal(installed.status, 0);
  const listed = capture(
    pnpm.path,
    ["list", "--depth", "Infinity", "--json"],
    root,
    env,
    output,
    `${kind}-installed-closure`,
  );
  assert.equal(listed.status, 0);
  assert.equal(listed.stderrBase64, "");
  const list = array(wholeJson(bytes(listed.stdoutBase64)));
  assert.equal(list.length, 1);
  const entries = new Map<string, { name: string; version: string; directory: string }>();
  const visit = (value: unknown): void => {
    const current = object(value);
    for (const key of ["dependencies", "devDependencies", "optionalDependencies"]) {
      if (current[key] === undefined) continue;
      for (const [name, raw] of Object.entries(object(current[key]))) {
        const row = object(raw),
          directory = realpathSync(text(row.path));
        const pkg = object(wholeJson(readFileSync(join(directory, "package.json"))));
        assert.equal(pkg.name, name);
        assert.equal(pkg.version, row.version);
        const id = directory;
        if (!entries.has(id)) {
          entries.set(id, { name, version: text(pkg.version), directory });
          visit(row);
        }
      }
    }
  };
  visit(list[0]);
  assert(entries.size > 0);
  const packages: InstallReceipt["packages"] = [];
  for (const entry of entries.values()) {
    const downloaded = await publicArchive(entry.name, entry.version, sink);
    sink("installed-package-before-archive-assertion", {
      entry,
      metadata: downloaded.metadata,
      manifestRawBase64: readFileSync(join(entry.directory, "package.json")).toString("base64"),
    });
    const archiveId = entry.name + "@" + entry.version,
      prior = archiveCache.get(archiveId);
    if (prior)
      assert(
        prior.equals(downloaded.body),
        "one role cannot silently merge different provider package versions",
      );
    else archiveCache.set(archiveId, downloaded.body);
    const comparison = archiveComparison(entry.directory, downloaded.body, downloaded.integrity);
    packages.push({
      name: entry.name,
      version: entry.version,
      directory: entry.directory,
      resolved: downloaded.resolved,
      integrity: downloaded.integrity,
      archiveSha512: comparison.tarballSHA512,
      allInstalledBytesEqualArchive: true,
      files: comparison.files.map((v) => ({ ...v })),
    });
  }
  const probe = capture(
    node.path,
    [fileURLToPath(new URL("./identity-probe.ts", import.meta.url)), root],
    root,
    env,
    output,
    `${kind}-independent-native-image-probe`,
  );
  assert.equal(probe.status, 0);
  assert.equal(probe.stderrBase64, "");
  const native = object(
    wholeJson(bytes(probe.stdoutBase64)),
  ) as unknown as InstallReceipt["native"];
  const cliRoot = realpathSync(join(root, "node_modules/vize"));
  const lockRaw = readFileSync(join(root, "pnpm-lock.yaml"));
  const receipt: InstallReceipt = {
    schema: "vize.humanReports11252_11253.install.v1",
    kind,
    root,
    packageManifest: wholeJson(manifest) as InstallReceipt["packageManifest"],
    packageRawBase64: manifest.toString("base64"),
    lockRawBase64: lockRaw.toString("base64"),
    lockSha256: sha256(lockRaw),
    node,
    nodeVersion: process.version,
    pnpm,
    pnpmVersion: recipe.pnpm,
    platform: process.platform,
    arch: process.arch,
    packageManagerEnvironment: {
      npm_config_userconfig: user,
      npm_config_globalconfig: global,
      npm_config_registry: "https://registry.npmjs.org",
    },
    kernel: (await import("node:os")).release(),
    packages,
    cli: fileImage(join(cliRoot, "bin/vize")),
    sdk: captureDeclarationGraph(cliRoot).files,
    mapperExec: ["node", "./bin/vize", "content-mapper"],
    native,
  };
  writeFileSync(
    join(output, `${kind}-installed-authority.json`),
    JSON.stringify(receipt, null, 2) + "\n",
    { flag: "wx" },
  );
  validateInstallation(receipt, fixture, recipe, authority, archiveCache);
  const sdkOutput = join(output, "sdk", kind);
  mkdirSync(sdkOutput, { recursive: true });
  for (const declaration of receipt.sdk)
    copyFileSync(
      declaration.path,
      join(sdkOutput, declaration.path.slice(cliRoot.length + 1).replaceAll("/", "--")),
    );
  return receipt;
}
export function recheckInstalled(receipt: InstallReceipt): void {
  recheckFiles(
    receipt.packages.map((pkg) => ({
      packageDirectory: pkg.directory,
      files: pkg.files,
    })),
  );
  assert.deepEqual(
    captureDeclarationGraph(realpathSync(join(receipt.root, "node_modules/vize"))).files,
    receipt.sdk,
  );
  assert.deepEqual(fileImage(receipt.node.path), receipt.node);
  assert.deepEqual(fileImage(receipt.pnpm.path), receipt.pnpm);
  assert(readFileSync(join(receipt.root, "pnpm-lock.yaml")).equals(bytes(receipt.lockRawBase64)));
}

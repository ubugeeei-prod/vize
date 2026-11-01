import assert from "node:assert/strict";
import { copyFileSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { recheckFiles } from "../public_install/archive.ts";
import { captureDeclarationGraph, fileImage } from "../computed_event_custody.ts";
import { capture, cleanEnvironment } from "./capture.ts";
import {
  manifestBytes,
  validateCapturePrerequisites,
  validateInstallation,
} from "./installation.ts";
import { prepareInstalledGraph } from "./package-graph.ts";
import {
  absentPackage,
  hostFacts,
  managerFiles,
  validateOptionalAbsence,
} from "./optional-residency.ts";
import type {
  CaptureInstallReceipt,
  Fixture,
  InstallationFacts,
  InstallReceipt,
  Recipe,
  ReleaseAuthority,
} from "./model.ts";
import { bytes, object, sha256, text, wholeJson } from "./value.ts";
import { discard, type Sink } from "./evidence.ts";
import { checkCaseManagers } from "./input-state.ts";
import type { BinPolicy } from "./dependency-bin-policy.ts";
import { recheckBinPackage } from "./dependency-bin-record.ts";
import { collectBinConfiguration, recheckBinConfiguration } from "./bin-configuration.ts";
import { captureExecutableChains, recheckExecutableChains } from "./command-chains.ts";
import { recheckCapturePayload } from "./capture-payload.ts";
export interface PreparedInstallation {
  capture: CaptureInstallReceipt;
  qualify: () => InstallReceipt;
  recheck: (expectedCaseManifest?: Buffer) => void;
}
export async function preparePublic(
  root: string,
  kind: InstallReceipt["kind"],
  fixture: Fixture,
  recipe: Recipe,
  authority: ReleaseAuthority,
  pnpmPath: string,
  output: string,
  archiveCache: Map<string, Buffer>,
  sink: Sink = discard,
  binPolicy: BinPolicy,
): Promise<PreparedInstallation> {
  assert.equal(process.version, recipe.node);
  assert.equal(process.platform, recipe.platform);
  assert.equal(process.arch, recipe.arch);
  mkdirSync(root);
  root = realpathSync(root);
  const env = cleanEnvironment(sink),
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
  const configuration = collectBinConfiguration(root, env, manifest, sink);
  writeFileSync(
    join(output, `${kind}-preinstall-bin-configuration.json`),
    JSON.stringify(configuration, null, 2) + "\n",
    { flag: "wx" },
  );
  const installed = capture(pnpm.path, ["install"], root, env, output, `${kind}-install`);
  recheckBinConfiguration(configuration, manifest, sink);
  assert.equal(installed.error, null);
  assert.equal(installed.signal, null);
  assert.equal(installed.status, 0);
  const host = hostFacts(),
    manager = managerFiles(root),
    lockRaw = readFileSync(join(root, "pnpm-lock.yaml"));
  const installContextPath = join(output, `${kind}-install-context.json`);
  writeFileSync(
    installContextPath,
    JSON.stringify(
      {
        root,
        host,
        node,
        pnpm,
        originalArgv: ["install"],
        managerFiles: manager,
        registryEnvironment: {
          npm_config_userconfig: user,
          npm_config_globalconfig: global,
          npm_config_registry: env.npm_config_registry,
        },
      },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
  writeFileSync(join(output, `${kind}-pnpm-lock.yaml.txt`), lockRaw, { flag: "wx" });
  for (const file of manager)
    if (file.state === "PRESENT")
      writeFileSync(
        join(output, `${kind}-root-${file.path.replaceAll("/", "--")}.txt`),
        bytes(text(file.rawBase64)),
        { flag: "wx" },
      );
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
  const required = new Map([
    ["vize", authority.version],
    ["vue", recipe.vue],
    ["typescript", recipe.typescript],
    ["@vizejs/native", authority.version],
    [
      recipe.platform === "darwin" ? "@vizejs/native-darwin-arm64" : "@vizejs/native-linux-x64-gnu",
      authority.version,
    ],
  ]);
  if (recipe.vueTsc) required.set("vue-tsc", recipe.vueTsc);
  if (kind === "tsrs-supplement") required.set("tsrs", "0.20.1-ts7.1.0-beta");
  const graph = await prepareInstalledGraph(
    root,
    wholeJson(bytes(listed.stdoutBase64)),
    lockRaw,
    host,
    required,
    archiveCache,
    sink,
    undefined,
    {
      path: installContextPath,
      managerVersion: recipe.pnpm,
      conformance: {
        role: recipe.issue === 11252 ? "runtime-prop-null-11252" : "imports-vue-ts-11253",
        policy: binPolicy,
        manager: {
          node,
          pnpm,
          version: checkVersion,
          reportedVersion: recipe.pnpm,
          contextPath: installContextPath,
          configuration,
        },
      },
    },
  );
  const packages = graph.packages;
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
  const facts: InstallationFacts = {
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
    host,
    managerFiles: manager,
    declaredOptionalNotInstalled: graph.absent,
    packageManagerEnvironment: {
      npm_config_userconfig: user,
      npm_config_globalconfig: global,
      npm_config_registry: "https://registry.npmjs.org",
    },
    kernel: (await import("node:os")).release(),
    packages,
    binPolicy,
    cli: fileImage(join(cliRoot, "bin/vize")),
    sdk: captureDeclarationGraph(cliRoot).files,
    mapperExec: ["node", "./bin/vize", "content-mapper"],
    native,
  };
  const executableChains = captureExecutableChains({
    root,
    kind,
    recipe,
    manifest,
    configuration,
    binPolicy,
    packages,
    catalog: graph.catalog,
    node,
    pnpm,
    env,
    sink,
  });
  const receipt: CaptureInstallReceipt = {
    ...facts,
    schema: "vize.humanReports11252_11253.capturePrerequisites.v1",
    packages,
    binConfiguration: configuration,
    executableChains,
    mandatoryExecutableChainsAuthenticated: true,
    completeInstalledFileSetQualified: false,
    accepted: false,
    productQualified: false,
    afterQualified: false,
    issueQualified: false,
  };
  writeFileSync(
    join(output, `${kind}-capture-prerequisites.json`),
    JSON.stringify(receipt, null, 2) + "\n",
    { flag: "wx" },
  );
  validateCapturePrerequisites(receipt, fixture, recipe, authority, archiveCache, sink);
  const sdkOutput = join(output, "sdk", kind);
  mkdirSync(sdkOutput, { recursive: true });
  for (const declaration of receipt.sdk)
    copyFileSync(
      declaration.path,
      join(sdkOutput, declaration.path.slice(cliRoot.length + 1).replaceAll("/", "--")),
    );
  return {
    capture: receipt,
    recheck: (expectedCaseManifest) => recheckCaptureInstallation(receipt, expectedCaseManifest),
    qualify: () => {
      const qualified: InstallReceipt = {
        ...facts,
        schema: "vize.humanReports11252_11253.install.v1",
        packages: graph.qualify(),
      };
      validateInstallation(qualified, fixture, recipe, authority, archiveCache);
      writeFileSync(
        join(output, `${kind}-installed-authority.json`),
        JSON.stringify(qualified, null, 2) + "\n",
        { flag: "wx" },
      );
      return qualified;
    },
  };
}
export async function installPublic(
  ...args: Parameters<typeof preparePublic>
): Promise<InstallReceipt> {
  return (await preparePublic(...args)).qualify();
}
export function recheckCaptureInstallation(
  receipt: CaptureInstallReceipt,
  expectedCaseManifest?: Buffer,
): void {
  assert.equal(receipt.schema, "vize.humanReports11252_11253.capturePrerequisites.v1");
  assert.equal(receipt.completeInstalledFileSetQualified, false);
  checkCaseManagers(receipt.managerFiles, managerFiles(receipt.root), expectedCaseManifest);
  assert.deepEqual(hostFacts(), receipt.host);
  recheckCapturePayload(receipt.packages);
  recheckExecutableChains(receipt.executableChains, receipt, expectedCaseManifest);
  recheckBinConfiguration(receipt.binConfiguration, expectedCaseManifest);
  assert.deepEqual(
    captureDeclarationGraph(realpathSync(join(receipt.root, "node_modules/vize"))).files,
    receipt.sdk,
  );
  assert.deepEqual(fileImage(receipt.native.path), {
    path: receipt.native.path,
    bytes: receipt.native.bytes,
    sha256: receipt.native.sha256,
  });
  assert.deepEqual(fileImage(receipt.node.path), receipt.node);
  assert.deepEqual(fileImage(receipt.pnpm.path), receipt.pnpm);
  assert(readFileSync(join(receipt.root, "pnpm-lock.yaml")).equals(bytes(receipt.lockRawBase64)));
  for (const row of receipt.declaredOptionalNotInstalled) {
    validateOptionalAbsence(
      row,
      receipt.root,
      bytes(receipt.lockRawBase64),
      receipt.host,
      receipt.packages,
    );
    assert.deepEqual(
      absentPackage(receipt.root, row.physical.path, row.parentDirectory, row.name, row.version),
      row.physical,
    );
  }
}
export function recheckInstalled(receipt: InstallReceipt, expectedCaseManifest?: Buffer): void {
  checkCaseManagers(receipt.managerFiles, managerFiles(receipt.root), expectedCaseManifest);
  assert.deepEqual(hostFacts(), receipt.host);
  for (const row of receipt.declaredOptionalNotInstalled) {
    validateOptionalAbsence(
      row,
      receipt.root,
      bytes(receipt.lockRawBase64),
      receipt.host,
      receipt.packages,
    );
    assert.deepEqual(
      absentPackage(receipt.root, row.physical.path, row.parentDirectory, row.name, row.version),
      row.physical,
    );
  }
  recheckFiles(
    receipt.packages.map((pkg) => ({
      packageDirectory: pkg.directory,
      files: pkg.files,
    })),
  );
  for (const pkg of receipt.packages)
    if (pkg.allInstalledBytesEqualArchive === false) {
      recheckBinPackage(pkg, receipt);
      recheckBinConfiguration(pkg.binContext.configuration, expectedCaseManifest);
    }
  assert.deepEqual(
    captureDeclarationGraph(realpathSync(join(receipt.root, "node_modules/vize"))).files,
    receipt.sdk,
  );
  assert.deepEqual(fileImage(receipt.node.path), receipt.node);
  assert.deepEqual(fileImage(receipt.pnpm.path), receipt.pnpm);
  assert(readFileSync(join(receipt.root, "pnpm-lock.yaml")).equals(bytes(receipt.lockRawBase64)));
}

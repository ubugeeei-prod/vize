import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { publicInputs } from "./inputs.ts";
import {
  packageProvider,
  installedCorsaProvider,
  publicLspCli,
  rejectLspOverrides,
  versionProbe,
  contained,
} from "./lsp_assets.ts";
import { assertBuiltSupport, cargoCommand, freezeSupportLock } from "./lsp_cargo.ts";
import { assertAllLspPackets } from "./lsp_packets.ts";
import { assertLspSource, sha256, targets } from "./lsp_source.ts";
import {
  atomicArguments,
  atomicCaptureDirectory,
  atomicCommandName,
  atomicInputs,
  atomicReceiptFile,
  observeAtomicOriginal,
  assertAtomicOriginalReceipt,
} from "./lsp_atomic.ts";

const validator = fileURLToPath(
  new URL(
    "../../../../tests/performance/support/warm-type-backed-public-archives.ts",
    import.meta.url,
  ),
);
export const archiveValidatorSha256 = () => sha256(fs.readFileSync(validator));
export async function assertInstalledLsp(output: string) {
  const load = (name: string) => JSON.parse(fs.readFileSync(path.join(output, name), "utf8"));
  const preparation = load("preparation.json"),
    receipt = load("lsp.json");
  const source = publicInputs(process.env);
  assert.deepEqual(receipt.source, source);
  assert.equal(receipt.version, preparation.version);
  assert.equal(receipt.success, true);
  assertLspSource(preparation.lsp);
  assert.equal(receipt.archiveValidatorSha256, archiveValidatorSha256());
  assert.equal(contained(receipt.cli.executable, preparation.consumer), receipt.cli.executable);
  assert.equal(sha256(fs.readFileSync(receipt.cli.executable)), receipt.cli.executableSha256);
  assert.equal(sha256(fs.readFileSync(receipt.cli.archive)), receipt.cli.archiveSha256);
  assert.equal(
    sha256(fs.readFileSync(path.join(preparation.lsp.adapter, "Cargo.lock"))),
    receipt.cargo.lockSha256,
  );
  assert.equal(
    sha256(fs.readFileSync(path.join(preparation.consumer, "package-lock.json"))),
    receipt.npmLockSha256,
  );
  assert.deepEqual(
    await installedCorsaProvider(
      preparation.consumer,
      preparation.version,
      source.head,
      receipt.provider.name,
      receipt.provider.version,
    ),
    receipt.provider,
  );
  assert.deepEqual(packageProvider(preparation.consumer, "vue", "3.5.35"), receipt.vue);
  assert.deepEqual(
    assertAllLspPackets(
      preparation.lsp,
      source,
      receipt.cli,
      receipt.provider,
      preparation.consumer,
      output,
      preparation.version,
    ),
    receipt.packets,
  );
  assertBuiltSupport(
    fs.readFileSync(path.join(output, "cargo-test.stdout"), "utf8"),
    receipt.cargo.allowedBuildIds,
  );
  assertAtomicOriginalReceipt(
    preparation.lsp,
    source,
    receipt.cli,
    receipt.provider,
    preparation.consumer,
    output,
    preparation.version,
    receipt.cargo,
  );
  return receipt;
}

async function run() {
  rejectLspOverrides();
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  const source = publicInputs(process.env),
    output = path.resolve(process.env.PUBLIC_OUTPUT!);
  const load = (name: string) => JSON.parse(fs.readFileSync(path.join(output, name), "utf8"));
  const preparation = load("preparation.json"),
    publication = load("publication.json");
  assert.deepEqual(preparation.source, source);
  assert.equal(publication.success, true);
  assert.deepEqual(publication.source.head, source.head);
  assert.equal(publication.source.tag, source.tag);
  assert.equal(preparation.archiveValidatorSha256, archiveValidatorSha256());
  assertLspSource(preparation.lsp);
  const consumer = fs.realpathSync(preparation.consumer),
    version = preparation.version;
  const cli = await publicLspCli(source.tag, publication.github.releaseId, output, consumer);
  assert.equal(Buffer.from(cli.probe.stdoutBase64, "base64").toString().trim(), `vize ${version}`);
  const npmCli = JSON.parse(
    fs.readFileSync(path.join(consumer, "node_modules/vize/package.json"), "utf8"),
  );
  assert.equal(npmCli.version, version);
  const { name, version: providerVersion } = preparation.lsp.metadata.provider;
  assert.equal(
    npmCli.optionalDependencies[name],
    providerVersion,
    "published CLI declares exact H provider",
  );
  const provider = await installedCorsaProvider(
    consumer,
    version,
    source.head,
    name,
    providerVersion,
  );
  const vue = packageProvider(consumer, "vue", "3.5.35");
  assert.equal(fs.realpathSync(path.join(consumer, "playground/node_modules/vue")), vue.directory);
  const providerProbe = versionProbe(provider.executable!);
  assert.equal(
    Buffer.from(providerProbe.stdoutBase64, "base64").toString().trim(),
    `Version ${providerVersion}`,
  );
  fs.writeFileSync(
    path.join(output, "lsp-installed-authority.json"),
    JSON.stringify({ source, version, cli, provider, vue, providerProbe }, null, 2) + "\n",
    { flag: "wx" },
  );
  const npmLockSha256 = sha256(fs.readFileSync(path.join(consumer, "package-lock.json")));
  const environment = {
    ...process.env,
    RUSTUP_TOOLCHAIN: preparation.lsp.metadata.toolchain,
    CARGO_BIN_EXE_vize: cli.executable,
    SOURCE_SHA: source.head,
    VIZE_CORSA_DEV_PATHS: "0",
    VIZE_TEST_REQUIRE_TSGO: "1",
    VIZE_TEST_FIX_HISTORY_CAPTURE_DIR: path.join(output, "lsp"),
    VIZE_INLAY_HINT_CAPTURE: path.join(output, "lsp-processes"),
    CARGO_TARGET_DIR: path.join(consumer, "target/lsp-support"),
  };
  const cargo = freezeSupportLock(
    preparation.lsp.adapter,
    preparation.lsp.sourceTree,
    environment,
    output,
  );
  const command = cargoCommand(
    [
      "test",
      "--locked",
      "--message-format=json",
      "--manifest-path",
      path.join(preparation.lsp.adapter, "Cargo.toml"),
      ...targets.flatMap((target) => ["--test", target]),
      "--",
      "--nocapture",
    ],
    preparation.lsp.adapter,
    environment,
    output,
    "cargo-test",
  );
  const built = assertBuiltSupport(
    fs.readFileSync(path.join(output, "cargo-test.stdout"), "utf8"),
    cargo.allowedBuildIds,
  );
  fs.writeFileSync(
    path.join(output, "lsp-atomic-inputs.json"),
    JSON.stringify(atomicInputs(preparation.lsp, cli, provider), null, 2) + "\n",
    { flag: "wx" },
  );
  cargoCommand(
    atomicArguments(preparation.lsp.adapter),
    preparation.lsp.adapter,
    { ...environment, VIZE_INLAY_HINT_CAPTURE: path.join(output, atomicCaptureDirectory) },
    output,
    atomicCommandName,
  );
  const atomicOriginal = observeAtomicOriginal(
    preparation.lsp,
    source,
    cli,
    provider,
    consumer,
    output,
    version,
    cargo,
  );
  assert.equal(
    sha256(fs.readFileSync(path.join(preparation.lsp.adapter, "Cargo.lock"))),
    cargo.lockSha256,
  );
  assert.equal(sha256(fs.readFileSync(path.join(consumer, "package-lock.json"))), npmLockSha256);
  assert.equal(sha256(fs.readFileSync(cli.executable)), cli.executableSha256);
  assert.deepEqual(
    await installedCorsaProvider(consumer, version, source.head, name, providerVersion),
    provider,
  );
  assert.deepEqual(packageProvider(consumer, "vue", "3.5.35"), vue);
  assertLspSource(preparation.lsp);
  const packets = assertAllLspPackets(
    preparation.lsp,
    source,
    cli,
    provider,
    consumer,
    output,
    version,
  );
  fs.writeFileSync(
    path.join(output, atomicReceiptFile),
    JSON.stringify(atomicOriginal, null, 2) + "\n",
    { flag: "wx" },
  );
  fs.writeFileSync(
    path.join(output, "lsp.json"),
    JSON.stringify(
      {
        schema: "vize-public-original-event-rename-v1",
        source,
        version,
        cli,
        provider,
        vue,
        providerProbe,
        npmLockSha256,
        cargo,
        command,
        built,
        packets,
        archiveValidatorSha256: archiveValidatorSha256(),
        success: true,
      },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
  console.log(JSON.stringify({ source, version, contexts: packets.contexts, success: true }));
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await run();

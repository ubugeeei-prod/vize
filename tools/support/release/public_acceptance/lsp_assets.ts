import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { stripTypeScriptTypes } from "node:module";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { resolveBundledCorsaRuntime } from "../../../../npm/cli/src/corsa-runtime.ts";
import { unpackPublicArchive } from "../../../../tests/performance/support/warm-type-backed-public-archives.ts";
import { publicReader } from "./http.ts";
import { rejectOverrides } from "./installed.ts";
import { sha256 } from "./lsp_source.ts";
import { readRawBlob } from "./plan.ts";

export const lspOverrides = [
  "VIZE_CORSA_DEV_PATHS",
  "CARGO_BIN_EXE_vize",
  "SOURCE_SHA",
  "VIZE_TEST_REQUIRE_TSGO",
  "VIZE_TEST_CONTENT_MAPPER_TSGO",
  "VIZE_TEST_CONTENT_MAPPER_VUE",
  "VIZE_TEST_DISABLE_TSGO",
  "VIZE_TEST_FIX_HISTORY_CAPTURE_DIR",
  "VIZE_INLAY_HINT_CAPTURE",
  "VIZE_LSP_PASSIVE_EVIDENCE",
  "RUSTC_WRAPPER",
  "RUSTC_WORKSPACE_WRAPPER",
  "RUSTFLAGS",
  "CARGO_ENCODED_RUSTFLAGS",
  "CARGO_BUILD_RUSTC",
  "RUSTC",
  "RUSTUP_TOOLCHAIN",
];
export function rejectLspOverrides(environment = process.env) {
  rejectOverrides(environment);
  for (const name of lspOverrides)
    assert.ok(!environment[name], `LSP override must be absent: ${name}`);
}
export function contained(file: string, root: string) {
  const real = fs.realpathSync(file);
  assert.ok(
    real.startsWith(fs.realpathSync(root) + path.sep),
    `outside consumer authority: ${file}`,
  );
  return real;
}
export function packageProvider(consumer: string, name: string, version: string) {
  const lock = JSON.parse(fs.readFileSync(path.join(consumer, "package-lock.json"), "utf8"));
  const location = `node_modules/${name}`;
  const entry = lock.packages[location];
  assert.equal(lock.lockfileVersion, 3);
  assert.equal(entry.version, version);
  assert.equal(entry.link, undefined);
  assert.equal(new URL(entry.resolved).origin, "https://registry.npmjs.org");
  assert.match(entry.integrity, /^sha512-[A-Za-z0-9+/]+={0,2}$/u);
  const directory = contained(path.join(consumer, location), path.join(consumer, "node_modules"));
  const manifest = fs.readFileSync(path.join(directory, "package.json"));
  const pkg = JSON.parse(manifest.toString());
  assert.equal(pkg.name, name);
  assert.equal(pkg.version, version);
  return {
    name,
    version,
    directory,
    resolved: entry.resolved,
    integrity: entry.integrity,
    manifestSha256: sha256(manifest),
  };
}
/** Use the shipped CLI's version-bound launcher contract, never a guessed or host binary. */
export async function installedCorsaProvider(
  consumer: string,
  cliVersion: string,
  head: string,
  name: string,
  version: string,
  root = process.cwd(),
) {
  const cli = packageProvider(consumer, "vize", cliVersion);
  const provider = packageProvider(consumer, name, version);
  const manifest = JSON.parse(fs.readFileSync(path.join(cli.directory, "package.json"), "utf8"));
  assert.equal(
    manifest.optionalDependencies?.[name],
    version,
    "published CLI declares exact H provider",
  );
  assert.equal(name, `@typescript/typescript-${process.platform}-${process.arch}`);
  const sourceFile = "npm/cli/src/corsa-runtime.ts";
  const module = contained(path.join(cli.directory, "src/corsa-runtime.ts"), cli.directory);
  const source = fs.readFileSync(module);
  const moduleSha256 = sha256(source);
  assert.equal(
    moduleSha256,
    sha256(readRawBlob(root, head, sourceFile)),
    "installed launcher resolver must match frozen H source",
  );
  // Both actors may inspect these bytes. Only the reviewed checkout implementation
  // executes: installed code must never run inside the credentialed finalizer.
  const trusted = fs.readFileSync(
    fileURLToPath(new URL("../../../../npm/cli/src/corsa-runtime.ts", import.meta.url)),
  );
  assert.equal(
    moduleSha256,
    sha256(trusted),
    "frozen installed resolver must match the reviewed trusted resolver contract",
  );
  const javascript = Buffer.from(
    stripTypeScriptTypes(source.toString(), { sourceUrl: pathToFileURL(module).href }),
  );
  const selected: unknown = resolveBundledCorsaRuntime({
    packageRoot: cli.directory,
    platform: process.platform,
    arch: process.arch,
  });
  assert.ok(
    typeof selected === "string" && selected.length > 0,
    "shipped Corsa provider required; no host fallback",
  );
  const executable = contained(selected, provider.directory);
  assert.ok(fs.lstatSync(executable).isFile(), "installed Corsa executable must be a regular file");
  return {
    ...provider,
    executable,
    executableSha256: sha256(fs.readFileSync(executable)),
    resolver: { cli, module, moduleSha256, javascriptSha256: sha256(javascript), sourceFile, head },
  };
}
/** Trusted finalizer rechecks every captured field without importing installed code. */
export async function assertInstalledCorsaProvider(
  consumer: string,
  cliVersion: string,
  head: string,
  observed: Awaited<ReturnType<typeof installedCorsaProvider>>,
  root = process.cwd(),
) {
  const current = await installedCorsaProvider(
    consumer,
    cliVersion,
    head,
    observed.name,
    observed.version,
    root,
  );
  assert.deepEqual(current, observed, "installed Corsa provider observation changed");
  return current;
}
export function versionProbe(executable: string) {
  const result = spawnSync(executable, ["--version"], { timeout: 30_000, maxBuffer: 1024 * 1024 });
  const receipt = {
    executable,
    arguments: ["--version"],
    status: result.status,
    signal: result.signal,
    error: result.error && String(result.error),
    stdoutBase64: result.stdout.toString("base64"),
    stderrBase64: result.stderr.toString("base64"),
  };
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0);
  return receipt;
}

/** Acquire the already authenticated release's real GNU archive; no source product build. */
export async function publicLspCli(
  tag: string,
  releaseId: number,
  output: string,
  consumer: string,
) {
  const physicalConsumer = fs.realpathSync(consumer),
    checkout = fs.realpathSync(process.cwd());
  assert.notEqual(physicalConsumer, checkout);
  assert.ok(!physicalConsumer.startsWith(checkout + path.sep));
  const { json, text } = publicReader();
  const release = await json(
    `https://api.github.com/repos/ubugeeei-prod/vize/releases/tags/${tag}`,
  );
  assert.equal(release.id, releaseId);
  assert.equal(release.tag_name, tag);
  assert.equal(release.draft, false);
  assert.equal(release.prerelease, false);
  assert.ok(Array.isArray(release.assets));
  const name = "vize-x86_64-unknown-linux-gnu.tar.gz";
  const asset = (n: string) => {
    const entries = (release.assets as Record<string, unknown>[]).filter(
      (entry) => entry.name === n,
    );
    assert.equal(entries.length, 1);
    const entry = entries[0];
    assert.equal(entry.state, "uploaded");
    assert.ok(Number.isSafeInteger(entry.id));
    assert.equal(
      entry.browser_download_url,
      `https://github.com/ubugeeei-prod/vize/releases/download/${tag}/${n}`,
    );
    return entry;
  };
  const item = asset(name),
    checksumAsset = asset(`${name}.sha256`);
  const archive = path.join(output, name);
  const response = await fetch(String(item.browser_download_url), {
    redirect: "follow",
    credentials: "omit",
    signal: AbortSignal.timeout(30_000),
  });
  assert.equal(response.ok, true);
  assert.ok(response.body);
  assert.ok(Number(item.size) > 0 && Number(item.size) < 256 * 1024 * 1024);
  const fd = fs.openSync(archive, "wx");
  let bytes = 0;
  try {
    for await (const part of response.body) {
      bytes += part.byteLength;
      assert.ok(bytes <= Number(item.size));
      fs.writeSync(fd, part);
    }
  } finally {
    fs.closeSync(fd);
  }
  assert.equal(bytes, item.size);
  const hash = sha256(fs.readFileSync(archive));
  assert.equal(item.digest, `sha256:${hash}`);
  const checksum = await text(String(checksumAsset.browser_download_url), "follow");
  assert.equal(checksum.trim(), `${hash}  ${name}`);
  const destination = path.join(physicalConsumer, "installed-cli");
  const unpack = unpackPublicArchive("tar", archive, destination);
  fs.copyFileSync(
    path.join(destination, "archive-audit.json"),
    path.join(output, "cli-archive-audit.json"),
  );
  fs.copyFileSync(`${destination}.process.json`, path.join(output, "cli-unpack-process.json"));
  assert.equal(unpack.members.length, 1);
  const executable = contained(path.join(destination, "vize"), physicalConsumer);
  assert.equal(sha256(fs.readFileSync(executable)), unpack.members[0].sha256);
  return {
    releaseId,
    tag,
    asset: item,
    checksumAsset,
    archive,
    archiveSha256: hash,
    unpack,
    executable,
    executableSha256: unpack.members[0].sha256,
    probe: versionProbe(executable),
  };
}

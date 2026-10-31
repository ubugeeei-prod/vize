import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { parse, parseAllDocuments } from "yaml";

const runtimeVariables = [
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "TSRS_PATH",
  "TSRS_EXECUTABLE",
] as const;
function record(value: unknown): asserts value is Record<string, unknown> {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
}
function member(value: unknown, name: string): unknown {
  record(value);
  return value[name];
}
const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const read = (root: string, file: string) => fs.readFileSync(path.join(root, file), "utf8");

/** Provision only the locked checkout test provider, without changing product discovery. */
export function provisionFullRustRuntime(
  root: string,
  environment: NodeJS.ProcessEnv = process.env,
) {
  const overrides = Object.fromEntries(
    runtimeVariables.flatMap((name) => (environment[name] ? [[name, environment[name]]] : [])),
  );
  const context = {
    schema: "vize.rust.native-runtime-provider",
    schemaVersion: 1,
    sourceSha: environment.GITHUB_SHA ?? null,
    execution: {
      runId: environment.GITHUB_RUN_ID ?? null,
      runAttempt: environment.GITHUB_RUN_ATTEMPT ?? null,
    },
  };
  const save = (provider: object) => {
    const receipt = { ...context, ...provider };
    const destination = path.join(root, "target/nextest/full/native-runtime-provider.json");
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, `${JSON.stringify(receipt, null, 2)}\n`, { flag: "wx" });
    return receipt;
  };
  // Invalid caller choices retain normal resolver precedence and failure.
  if (Object.keys(overrides).length > 0)
    return save({ selection: "inherited-explicit-runtime", overrides });

  const name = `@typescript/typescript-${process.platform}-${process.arch}`;
  const manifest = JSON.parse(read(root, "package.json")) as unknown;
  assert.equal(member(member(manifest, "optionalDependencies"), name), "catalog:corsa-runtime");
  const workspace = parse(read(root, "pnpm-workspace.yaml")) as unknown;
  const version = member(member(member(workspace, "catalogs"), "corsa-runtime"), name);
  assert.ok(typeof version === "string" && /^\d+\.\d+\.\d+(?:-[\w.-]+)?$/.test(version));
  // pnpm keeps its package-manager bootstrap and project lock in separate documents.
  const locks = parseAllDocuments(read(root, "pnpm-lock.yaml"))
    .map((document) => {
      assert.equal(document.errors.length, 0, "fixture runtime lock must parse completely");
      const value = document.toJS() as unknown;
      record(value);
      return value;
    })
    .filter((value) => Object.hasOwn(value, "catalogs"));
  assert.equal(locks.length, 1, "fixture runtime needs exactly one project lock authority");
  const lock = locks[0];
  const catalog = member(member(member(lock, "catalogs"), "corsa-runtime"), name);
  assert.equal(member(catalog, "specifier"), version);
  assert.equal(member(catalog, "version"), version);
  const installed = member(
    member(member(member(lock, "importers"), "."), "optionalDependencies"),
    name,
  );
  assert.equal(member(installed, "specifier"), "catalog:corsa-runtime");
  assert.equal(member(installed, "version"), version);

  const directory = fs.realpathSync(path.join(root, "node_modules", name));
  const metadataBytes = fs.readFileSync(path.join(directory, "package.json"));
  const metadata = JSON.parse(metadataBytes.toString("utf8")) as unknown;
  assert.equal(member(metadata, "name"), name, "fixture runtime package identity differs");
  assert.equal(member(metadata, "version"), version, "fixture runtime package version differs");
  const candidate = path.join(directory, "lib", process.platform === "win32" ? "tsc.exe" : "tsc");
  assert.ok(fs.lstatSync(candidate).isFile(), "fixture runtime must be a regular native image");
  const executable = fs.realpathSync(candidate);
  assert.equal(executable, candidate, "fixture runtime image must belong to its selected package");
  assert.doesNotMatch(executable, /[\r\n]/, "runtime path cannot be written safely to GITHUB_ENV");
  fs.accessSync(executable, fs.constants.X_OK);
  const image = fs.readFileSync(executable);
  const observed = spawnSync(executable, ["--version"], {
    encoding: "utf8",
    env: { ...environment },
    timeout: 10_000,
    maxBuffer: 1024 * 1024,
  });
  assert.ifError(observed.error);
  assert.equal(observed.signal, null, "native version probe was terminated by a signal");
  assert.equal(observed.status, 0, observed.stderr);
  assert.equal(observed.stderr, "", "native version probe produced stderr");
  assert.equal(
    observed.stdout,
    `Version ${version}\n`,
    "native version differs from locked provider",
  );
  assert.equal(
    digest(fs.readFileSync(executable)),
    digest(image),
    "runtime changed during version probe",
  );
  assert.ok(environment.GITHUB_ENV, "full Rust runtime setup requires GITHUB_ENV");
  const receipt = save({
    selection: "checkout-locked-platform-package",
    package: { name, version, directory, metadataSha256: digest(metadataBytes) },
    executable: {
      path: executable,
      sha256: digest(image),
      bytes: image.length,
      version: {
        args: ["--version"],
        stdout: observed.stdout,
        stderr: observed.stderr,
        exitCode: observed.status,
        signal: observed.signal,
      },
    },
  });
  fs.appendFileSync(environment.GITHUB_ENV, `CORSA_PATH=${executable}\n`);
  return receipt;
}

if (import.meta.main) provisionFullRustRuntime(process.cwd());

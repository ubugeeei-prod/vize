import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import { hash, parse, record, string, write, type MapperHost, type RecordValue } from "./model.ts";

const version = "7.1.0-beta";
const platform = "@typescript/typescript-linux-x64";
const expectedBinary = "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70";
const pins = {
  typescript: {
    integrity:
      "sha512-cjWjZAJDsmZzsNC5GwHNlW0YPM65jZa5oR0KyQcYX3KSVIfTvtBq//vewISyatKk4OLi5grZjHhIVE9fdrw1UQ==",
    resolved: "https://registry.npmjs.org/typescript/-/typescript-7.1.0-beta.tgz",
  },
  [platform]: {
    integrity:
      "sha512-kcD51ihf+pnI5KT4uQQYG04OpwznaZYBlIQhDhl8fflFd27NktxXsUeguxZyvUMyWfum7IF2bBS6Jv7ox0AgFQ==",
    resolved:
      "https://registry.npmjs.org/@typescript/typescript-linux-x64/-/typescript-linux-x64-7.1.0-beta.tgz",
  },
};
export function readMapperHost(
  delivery: RecordValue,
  cppWorkspace: string,
  output: string,
): MapperHost {
  assert.equal(process.platform, "linux", "the additional hosted column is Linux x64");
  assert.equal(process.arch, "x64");
  const plan = record(delivery.mapperHost);
  assert.equal(plan.route, "hosted-content-mapper");
  assert.equal(plan.sdkVersion, version);
  const workspace = fs.realpathSync(string(plan.workspace));
  assert.notEqual(workspace, cppWorkspace);
  assert.ok(!workspace.startsWith(cppWorkspace + path.sep));
  assert.ok(!cppWorkspace.startsWith(workspace + path.sep));
  const owned = (file: string) => {
    const real = fs.realpathSync(file);
    assert.ok(real.startsWith(workspace + path.sep));
    return real;
  };
  const manifest = (file: string) => record(parse(fs.readFileSync(owned(file))));
  const tsManifestPath = owned(path.join(workspace, "node_modules/typescript/package.json"));
  const ts = manifest(tsManifestPath);
  assert.equal(ts.name, "typescript");
  assert.equal(ts.version, version);
  assert.equal(record(ts.optionalDependencies)[platform], version);
  const require = createRequire(tsManifestPath);
  const nativeManifestPath = owned(require.resolve(platform + "/package.json"));
  const native = manifest(nativeManifestPath);
  assert.equal(native.name, platform);
  assert.equal(native.version, version);
  const backend = owned(path.join(path.dirname(nativeManifestPath), "lib/tsc"));
  const bytes = fs.readFileSync(backend);
  assert.equal(bytes.length, 24522912);
  assert.equal(hash(bytes), expectedBinary);
  const lockBytes = fs.readFileSync(path.join(workspace, "package-lock.json"));
  const lock = record(parse(lockBytes));
  const packages = record(lock.packages);
  const packagePins: RecordValue[] = [];
  for (const [name, file] of [
    ["typescript", tsManifestPath],
    [platform, nativeManifestPath],
  ] as const) {
    const key = path.relative(workspace, path.dirname(file)).replaceAll("\\", "/");
    const row = record(packages[key]);
    assert.equal(row.version, version);
    const pin = pins[name];
    assert.ok(pin);
    assert.equal(row.integrity, pin.integrity);
    assert.equal(row.resolved, pin.resolved);
    packagePins.push({ name, file, lockKey: key, version, ...pin });
  }
  const setupFile = fs.realpathSync(string(plan.setupReceipt));
  const setupBytes = fs.readFileSync(setupFile);
  const setup = record(parse(setupBytes));
  assert.equal(setup.schema, "vize.typeAdmission.mapperHostSetup.v1");
  assert.equal(setup.role, "hosted-content-mapper");
  assert.equal(setup.sdkVersion, version);
  assert.equal(setup.workspace, workspace);
  assert.equal(setup.status, "prepared");
  const executable = record(setup.executable);
  assert.equal(executable.file, backend);
  assert.equal(executable.bytes, bytes.length);
  assert.equal(executable.sha256, expectedBinary);
  assert.equal(record(setup.lock).sha256, hash(lockBytes));
  const setupReceipt = { path: setupFile, sha256: hash(setupBytes), record: setup };
  const probes: unknown[] = [];
  for (const option of ["--version", "--help"]) {
    const result = spawnSync(backend, [option], { cwd: workspace, timeout: 30_000 });
    write(path.join(output, "mapper-host-" + option.slice(2) + ".txt"), {
      command: [backend, option],
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      stdoutBase64: result.stdout?.toString("base64") ?? "",
      stderrBase64: result.stderr?.toString("base64") ?? "",
    });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0);
    if (option === "--version")
      assert.equal(
        new TextDecoder("utf8", { fatal: true }).decode(result.stdout).trim(),
        "Version " + version,
      );
    probes.push({ option, status: result.status });
  }
  fs.writeFileSync(path.join(output, "mapper-host-lock.original.txt"), lockBytes, { flag: "wx" });
  const identity = {
    workspace,
    backend,
    version,
    sha256: expectedBinary,
    lockSha256: hash(lockBytes),
    packagePins,
    setupReceipt,
    probes,
    route: "hosted-content-mapper",
    projectedPackageGraph: cppWorkspace,
    sriScope:
      "installed npm-ci lock entries match independently pinned registry tarball integrities",
    runtimeCapability:
      "version/help only here; required authored negative and repair stages run below",
  };
  write(path.join(output, "mapper-host-identity.txt"), identity);
  return identity;
}

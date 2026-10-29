import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { environment, verifyCustody } from "./process.ts";
import { verifyPublicProvider } from "../public-provider.ts";
import { readMapperHost } from "./mapper-host.ts";
import { hash, parse, record, string, write, type RecordValue, type Runtime } from "./model.ts";
export async function installedRuntime(input: {
  workspace: string;
  version: string;
  output: string;
  candidate: string;
  delivery: RecordValue;
}): Promise<Runtime> {
  const { version, candidate, delivery } = input;
  const workspace = fs.realpathSync(input.workspace),
    output = path.resolve(input.output);
  fs.mkdirSync(output, { recursive: true });
  for (const key of [
    "NODE_OPTIONS",
    "VIZE_PREFER_WORKSPACE_BINDING",
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
    "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
    "CORSA_PATH",
    "CORSA_EXECUTABLE",
    "TSGO_PATH",
    "TSGO_EXECUTABLE",
    "VIZE_PUBLIC_NATIVE_CUSTODY",
    "VIZE_OXLINT_NATIVE_CUSTODY",
  ])
    assert.ok(!process.env[key], "initial source/runtime override: " + key);
  const owned = (file: string) => {
    const real = fs.realpathSync(file);
    assert.ok(real.startsWith(workspace + path.sep));
    return real;
  };
  const packageRoot = (name: string) => owned(path.join(workspace, "node_modules", name));
  const manifest = (root: string) =>
    record(parse(fs.readFileSync(path.join(root, "package.json"))));
  const cliRoot = packageRoot("vize"),
    nativeRoot = packageRoot("@vizejs/native"),
    vueRoot = packageRoot("vue");
  assert.equal(manifest(cliRoot).version, version);
  assert.equal(manifest(nativeRoot).version, version);
  const vueVersion = string(manifest(vueRoot).version);
  assert.ok(["3.6.0-rc.13", "3.5.42"].includes(vueVersion));
  const require = createRequire(path.join(cliRoot, "package.json"));
  require("@vizejs/native");
  const images = Object.keys(require.cache)
    .filter((name) => name.endsWith(".node"))
    .map(owned);
  assert.equal(images.length, 1);
  const image = images[0];
  assert.ok(image);
  const bytes = fs.readFileSync(image),
    native = { path: image, bytes: bytes.length, sha256: hash(bytes) };
  const provider = await verifyPublicProvider(version, candidate, native);
  const bin = record(manifest(cliRoot).bin),
    cli = owned(path.join(cliRoot, string(bin.vize)));
  const platform = "@typescript/typescript-" + process.platform + "-" + process.arch;
  const platformManifest = owned(require.resolve(platform + "/package.json"));
  const platformRoot = path.dirname(platformManifest),
    backendPackage = manifest(platformRoot);
  assert.equal(backendPackage.name, platform);
  assert.equal(backendPackage.version, "7.0.2");
  const backend = owned(
    path.join(platformRoot, "lib", process.platform === "win32" ? "tsc.exe" : "tsc"),
  );
  const dependencies = record(manifest(cliRoot).optionalDependencies);
  if (dependencies[platform] !== undefined) assert.equal(dependencies[platform], "7.0.2");
  else {
    const tsManifest = owned(require.resolve("typescript/package.json"));
    const ts = record(parse(fs.readFileSync(tsManifest)));
    assert.equal(ts.version, "7.0.2");
    assert.equal(record(ts.optionalDependencies)[platform], "7.0.2");
  }
  const backendSha256 = hash(fs.readFileSync(backend));
  const hosted = readMapperHost(delivery, workspace, output);
  const node = fs.realpathSync(process.execPath);
  const observer = fileURLToPath(new URL("./observer.ts", import.meta.url));
  const lock = fs.readFileSync(path.join(workspace, "package-lock.json"));
  write(path.join(output, "identity.txt"), {
    delivery,
    workspace,
    version,
    candidate,
    node,
    cli: { path: cli, sha256: hash(fs.readFileSync(cli)) },
    native,
    provider,
    backend: { path: backend, sha256: backendSha256, manifest: backendPackage },
    vue: { root: vueRoot, version: vueVersion },
    mapperHost: hosted,
    lockSha256: hash(lock),
  });
  fs.writeFileSync(path.join(output, "package-lock.original.txt"), lock, { flag: "wx" });
  const runtime = {
    workspace,
    version,
    candidate,
    node,
    cli,
    native,
    backend,
    backendSha256,
    mapperHost: hosted,
    observer,
    output,
  };
  for (const [name, executable, argv, expected] of [
    ["cli", node, [cli, "--version"], "vize " + version],
    ["backend", backend, ["--version"], "Version 7.0.2"],
  ] as const) {
    const prefix = path.join(output, "version-" + name + "-native");
    const probe = spawnSync(executable, [...argv], {
      cwd: workspace,
      env: environment(runtime, prefix),
      timeout: 30_000,
    });
    write(path.join(output, "version-" + name + ".txt"), {
      command: [executable, ...argv],
      status: probe.status,
      signal: probe.signal,
      error: probe.error?.message ?? null,
      stdoutBase64: probe.stdout?.toString("base64") ?? "",
      stderrBase64: probe.stderr?.toString("base64") ?? "",
    });
    assert.equal(probe.error, undefined);
    assert.equal(probe.signal, null);
    assert.equal(probe.status, 0);
    assert.equal(new TextDecoder("utf8", { fatal: true }).decode(probe.stdout).trim(), expected);
    if (name === "cli") verifyCustody(runtime, prefix);
  }
  return runtime;
}

import { requiredEnv } from "../shared/files.ts";
import { resolve as resolveWorkflowInput } from "node:path";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { spawnSync } from "node:child_process";
const {
  assertCheckerVersion,
  assertPublicPackage,
  assertCustody,
  checkerVersion,
  checkerVersionOutput,
  fileImage,
  findNpmCli,
} = (await import(
  pathToFileURL(resolveWorkflowInput("./tools/support/release/computed_event_names.ts")).href
)) as typeof import("../../../../tools/support/release/computed_event_names.ts");
const version = requiredEnv("PUBLIC_VERSION");
assert.match(version ?? "", /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/u);
const evidence = path.join(requiredEnv("RUNNER_TEMP"), "component-aria-consumer");
const consumer = path.join(requiredEnv("RUNNER_TEMP"), "component-aria-installation");
assert.equal(fs.existsSync(evidence), false);
assert.equal(fs.existsSync(consumer), false);
fs.mkdirSync(evidence, { recursive: true });
fs.mkdirSync(consumer);
const save = (file: string, value: unknown) =>
  fs.writeFileSync(path.join(evidence, file), JSON.stringify(value, null, 2) + "\n");
const environment = Object.fromEntries(
  Object.entries(process.env).filter(
    ([name]) =>
      !/^(npm_config_|VIZE_|CORSA_|TSGO_|TSRS_|NAPI_RS_)/iu.test(name) &&
      name.toLowerCase() !== "path" &&
      !["NODE_OPTIONS", "NODE_PATH", "CARGO_BIN_EXE_vize"].includes(name),
  ),
);
environment.PATH =
  path.dirname(process.execPath) + path.delimiter + (process.env.PATH ?? process.env.Path ?? "");
const run = (
  id: string,
  executable: string,
  args: readonly string[],
  cwd: string | undefined,
  timeout: number,
) => {
  const observed = spawnSync(executable, args, {
    cwd,
    env: environment,
    encoding: "utf8",
    timeout,
    maxBuffer: 32 * 1024 * 1024,
  });
  const row = {
    id,
    executable,
    args,
    cwd,
    status: observed.status,
    signal: observed.signal,
    error: observed.error
      ? {
          message: observed.error.message,
          code: (observed.error as NodeJS.ErrnoException).code,
        }
      : null,
    stdout: observed.stdout ?? "",
    stderr: observed.stderr ?? "",
  };
  save(id + ".json", row);
  fs.writeFileSync(path.join(evidence, id + ".stdout.raw"), row.stdout);
  fs.writeFileSync(path.join(evidence, id + ".stderr.raw"), row.stderr);
  return row;
};
const packageFile = path.join(consumer, "package.json");
fs.writeFileSync(
  packageFile,
  JSON.stringify(
    {
      name: "published-component-aria-11144",
      private: true,
      type: "module",
      dependencies: { vize: version, vue: "3.5.43", tsrs: checkerVersion },
    },
    null,
    2,
  ) + "\n",
);
fs.copyFileSync(packageFile, path.join(evidence, "package.json"));
const userConfig = path.join(evidence, "npm-user.config");
const globalConfig = path.join(evidence, "npm-global.config");
fs.writeFileSync(userConfig, "");
fs.writeFileSync(globalConfig, "");
const npm = findNpmCli();
const installed = run(
  "install",
  process.execPath,
  [
    npm,
    "install",
    "--ignore-scripts",
    "--include=optional",
    "--no-audit",
    "--no-fund",
    "--package-lock=true",
    "--install-strategy=hoisted",
    "--registry=https://registry.npmjs.org",
    "--userconfig=" + userConfig,
    "--globalconfig=" + globalConfig,
  ],
  consumer,
  300_000,
);
const lockFile = path.join(consumer, "package-lock.json");
if (fs.existsSync(lockFile)) fs.copyFileSync(lockFile, path.join(evidence, "package-lock.json"));
assert.equal(installed.error, null);
assert.equal(installed.signal, null);
assert.equal(installed.status, 0);
const lock = JSON.parse(fs.readFileSync(lockFile, "utf8"));
assert.equal(lock.lockfileVersion, 3);
const images = [fileImage(packageFile), fileImage(lockFile), fileImage(npm)];
const packages: { name: string; version: string; directory: string; lock: unknown }[] = [];
const identify = (name: string, expected: string) => {
  const directory = fs.realpathSync(path.join(consumer, "node_modules", name));
  const file = path.join(directory, "package.json");
  const manifest = JSON.parse(fs.readFileSync(file, "utf8"));
  assertPublicPackage(manifest, lock.packages["node_modules/" + name], name, expected);
  images.push(fileImage(file));
  packages.push({
    name,
    version: expected,
    directory,
    lock: lock.packages["node_modules/" + name],
  });
  return { directory, manifest };
};
const vize = identify("vize", version);
identify("vue", "3.5.43");
const checker = identify("tsrs", checkerVersion);
const resolver = path.join(checker.directory, "lib/getExePath.js");
const { default: getExePath } = await import(pathToFileURL(resolver).href);
const host = fs.realpathSync(getExePath());
const providerManifest = JSON.parse(
  fs.readFileSync(path.join(path.dirname(host), "package.json"), "utf8"),
);
assert.match(providerManifest.name, /^@tsrs\//u);
assert.equal(checker.manifest.optionalDependencies[providerManifest.name], checkerVersion);
const provider = identify(providerManifest.name, checkerVersion);
const relative = path.relative(provider.directory, host);
assert.ok(relative && !relative.startsWith("..") && !path.isAbsolute(relative));
assert.equal(path.basename(host), process.platform === "win32" ? "tsrs.exe" : "tsrs");
images.push(fileImage(host), fileImage(resolver), fileImage(process.execPath));
const declarations = [
  "virtual-ts-globals.d.ts",
  "virtual-ts-patterns.d.ts",
  "virtual-ts.d.ts",
  "virtual-ts.d.mts",
  "virtual-ts.d.cts",
  "content-mapper-helpers.d.ts",
  "content-mapper-helpers.d.mts",
  "content-mapper-helpers.d.cts",
  "generated/content-mapper-types.d.ts",
];
for (const file of ["package.json", ...declarations]) {
  const source = path.join(vize.directory, file);
  const target = path.join(evidence, "installed-vize", file);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.copyFileSync(source, target);
  if (file !== "package.json") images.push(fileImage(source));
}
const observedVersion = run("mapper-version", host, ["--version"], consumer, 60_000);
assertCheckerVersion(observedVersion);
save("installation.json", {
  version,
  packages,
  node: fileImage(process.execPath),
  "host": fileImage(host),
  images,
});
const output = path.join(evidence, "acceptance");
const acceptance = run(
  "acceptance",
  process.execPath,
  [
    path.join(
      requiredEnv("GITHUB_WORKSPACE"),
      "tools/support/compat/content-mapper/aria-component-attrs.ts",
    ),
    "--consumer",
    consumer,
    "--mapper-host",
    host,
    "--mapper-version",
    checkerVersionOutput.slice(0, -1),
    "--output",
    output,
  ],
  process.env.GITHUB_WORKSPACE,
  600_000,
);
assertCustody(images);
assert.equal(acceptance.error, null);
assert.equal(acceptance.signal, null);
assert.equal(acceptance.status, 0);
const providers = JSON.parse(fs.readFileSync(path.join(output, "providers.json"), "utf8"));
const report = JSON.parse(fs.readFileSync(path.join(output, "report.json"), "utf8"));
assert.equal(providers.mode, "unmodified-installed-package");
assert.equal(providers.source, undefined);
assert.equal(providers.sourceCli, undefined);
assert.equal(providers.package.version, version);
assert.deepEqual(
  (providers.declarations as { file: string }[]).map((row) => row.file),
  declarations,
);
assert.equal(report.installedVersion, version);
assert.equal(report.completeCases, 20);
assert.equal(report.passed, 20);
assert.equal(report.independentPropReference, true);
assert.equal(report.rawContext.passed, true);
assert.equal(report.rawContext.referenceErrors, 3);
assert.equal(report.rawContext.helperDeclarations, 3);
save("accepted.json", {
  version,
  completeCases: 20,
  physicalDeclarations: 9,
  rawContext: report.rawContext,
});

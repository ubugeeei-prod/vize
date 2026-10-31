import assert from "node:assert/strict";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import {
  asRecord,
  asString,
  parseJson,
  readJsonRecord,
  type JsonRecord,
} from "../../json-value.ts";
import { npmArgv, type Recorder, success } from "../../config/windows_consumer/common.ts";
import { bootstrapPnpm } from "../editor_vite_stub/install.ts";
import { digest, inside } from "../editor_vite_stub/filesystem.ts";
import { validateNativeReceipt } from "../constructor_original27_public/authority.ts";
import { originalFiles, type Phase } from "./contract.ts";

export function identity(
  root: string,
  phase: Phase,
  version: string,
  authority: JsonRecord,
  candidate: string,
  run: string,
  record: Recorder,
): JsonRecord {
  const output = join(record.root, phase + ".installed-identity.json");
  success(
    record.run(
      phase + "-installed-identity",
      [
        process.execPath,
        fileURLToPath(new URL("../editor_vite_stub/identity-probe.ts", import.meta.url)),
        root,
        output,
      ],
      root,
    ),
  );
  const value = readJsonRecord(output);
  assert.equal(value.platform, "darwin");
  assert.equal(value.arch, "arm64");
  assert.equal(value.node, "v26.6.0");
  for (const key of ["cliVersion", "nativeVersion", "platformVersion", "pluginVersion"])
    assert.equal(value[key], version);
  assert.deepEqual(value.hostVersions, {
    "vite-plus": "0.3.0",
    vue: "3.5.43",
    typescript: "6.0.3",
  });
  const binary = asRecord(value.binary);
  validateNativeReceipt(
    authority,
    {
      target: "darwin-arm64",
      bytes: Number(binary.bytes),
      sha256: asString(binary.sha256),
      token: asString(asRecord(value.identity).token),
    },
    version,
    candidate,
    run,
  );
  const lock = readFileSync(join(root, "pnpm-lock.yaml"), "utf8");
  for (const name of [
    "vize",
    "@vizejs/vite-plugin",
    "@vizejs/native",
    "@vizejs/native-darwin-arm64",
  ]) {
    const sri = asRecord(asRecord(authority.npmSRI)[name]);
    assert.equal(new URL(asString(sri.metadataOrigin)).origin, "https://registry.npmjs.org");
    if (phase === "original-default-provider") {
      const result = record.run(
        phase + "-registry-" + name.replaceAll(/[@/]/gu, "-"),
        npmArgv(
          "view",
          name + "@" + version,
          "dist",
          "--json",
          "--registry=https://registry.npmjs.org",
        ),
        root,
      );
      success(result);
      const dist = asRecord(parseJson(result.stdout));
      assert.equal(dist.integrity, sri.integrity);
      assert.equal(dist.tarball, sri.tarball);
    }
    assert(lock.includes(asString(sri.integrity)), "actual public lock SRI: " + name);
  }
  copyFileSync(join(root, "pnpm-lock.yaml"), join(record.root, phase + ".pnpm-lock.yaml"));
  copyFileSync(join(root, "package.json"), join(record.root, phase + ".package.json"));
  return value;
}
export function install(
  root: string,
  tools: string,
  version: string,
  original: JsonRecord,
  record: Recorder,
): string[] {
  mkdirSync(root, { recursive: false });
  writeFileSync(join(root, "package.json"), '{"private":true,"type":"module"}\n', { flag: "wx" });
  for (const [name, content] of originalFiles(original)) {
    mkdirSync(dirname(join(root, name)), { recursive: true });
    writeFileSync(join(root, name), content, { flag: "wx" });
  }
  const pnpm = bootstrapPnpm(tools, record);
  success(
    record.run(
      "public-install-original-role",
      [
        ...pnpm,
        "add",
        "-D",
        "--save-exact",
        "--registry=https://registry.npmjs.org",
        "vize@" + version,
        "@vizejs/vite-plugin@" + version,
        "vite-plus@0.3.0",
        "vue@3.5.43",
        "typescript@6.0.3",
      ],
      root,
      process.env,
      600_000,
    ),
  );
  return pnpm;
}
/** Installed roles are not assertions about which backend the product selects. */
export function installedProviders(root: string, phase: Phase) {
  const manifest = readJsonRecord(join(root, "package.json"));
  const dependencies = asRecord(manifest.devDependencies);
  const stable = "@typescript/typescript-darwin-arm64";
  const cliRoot = realpathSync(join(root, "node_modules/vize"));
  const cli = readJsonRecord(join(cliRoot, "package.json"));
  assert.equal(cli.name, "vize");
  assert.equal(asRecord(cli.optionalDependencies)[stable], "7.0.2");
  const cliRequire = createRequire(join(cliRoot, "package.json"));
  const bundledRoot = dirname(realpathSync(cliRequire.resolve(stable + "/package.json")));
  const bundled = readJsonRecord(join(bundledRoot, "package.json"));
  assert.equal(bundled.name, stable);
  assert.equal(bundled.version, "7.0.2");
  const installed = [
    {
      role: "published-cli-optional",
      name: stable,
      version: "7.0.2",
      packageRoot: bundledRoot,
      executable: realpathSync(join(bundledRoot, "lib/tsc")),
    },
  ];
  if (phase === "original-default-provider") {
    assert.equal(dependencies[stable], undefined);
    assert.equal(dependencies.tsrs, undefined);
    assert.equal(existsSync(join(root, "node_modules", stable)), false);
    assert.equal(existsSync(join(root, "node_modules/tsrs")), false);
  } else if (phase === "stable-user-provider") {
    assert.equal(dependencies[stable], "7.0.2");
    assert.equal(dependencies.tsrs, undefined);
    const packageRoot = realpathSync(join(root, "node_modules", stable));
    const metadata = readJsonRecord(join(packageRoot, "package.json"));
    assert.equal(metadata.name, stable);
    assert.equal(metadata.version, "7.0.2");
    installed.push({
      role: "explicit-user-stable",
      name: stable,
      version: "7.0.2",
      packageRoot,
      executable: realpathSync(join(packageRoot, "lib/tsc")),
    });
  } else {
    assert.equal(dependencies[stable], undefined);
    assert.equal(dependencies.tsrs, "0.20.1-ts7.1.0-beta");
    assert.equal(
      existsSync(join(root, "node_modules", stable)),
      false,
      "stable provider must be removed by pnpm, never hidden with a resolver override",
    );
    const packageRoot = realpathSync(join(root, "node_modules/tsrs"));
    const metadata = readJsonRecord(join(packageRoot, "package.json"));
    assert.equal(metadata.name, "tsrs");
    assert.equal(metadata.version, "0.20.1-ts7.1.0-beta");
    const declared = asRecord(metadata.optionalDependencies);
    assert.equal(declared["@tsrs/darwin-arm64"], "0.20.1-ts7.1.0-beta");
    const require = createRequire(join(packageRoot, "package.json"));
    const platform = dirname(realpathSync(require.resolve("@tsrs/darwin-arm64/package.json")));
    const image = readJsonRecord(join(platform, "package.json"));
    assert.equal(image.name, "@tsrs/darwin-arm64");
    assert.equal(image.version, "0.20.1-ts7.1.0-beta");
    installed.push({
      role: "reported-tsrs-installed",
      name: "@tsrs/darwin-arm64",
      version: "0.20.1-ts7.1.0-beta",
      packageRoot: platform,
      executable: realpathSync(join(platform, "tsrs")),
    });
  }
  for (const entry of installed) {
    assert(inside(root, entry.packageRoot) && inside(entry.packageRoot, entry.executable));
  }
  return { cliRoot, installed };
}
export function provider(root: string, phase: Phase, record: Recorder): string[] {
  const inventory = installedProviders(root, phase);
  const packages = [
    ...new Map(inventory.installed.map(({ name, version }) => [name, { name, version }])).values(),
  ];
  if (phase === "reported-tsrs") packages.push({ name: "tsrs", version: "0.20.1-ts7.1.0-beta" });
  const lock = readFileSync(join(root, "pnpm-lock.yaml"), "utf8");
  const origins = packages.map(({ name, version }) => {
    const result = record.run(
      phase + "-provider-registry-" + name.replaceAll(/[@/]/gu, "-"),
      npmArgv(
        "view",
        name + "@" + version,
        "dist",
        "--json",
        "--registry=https://registry.npmjs.org",
      ),
      root,
    );
    success(result);
    const dist = asRecord(parseJson(result.stdout));
    assert.equal(new URL(asString(dist.tarball)).origin, "https://registry.npmjs.org");
    assert.match(asString(dist.integrity), /^sha512-[A-Za-z0-9+/]+={0,2}$/u);
    assert(
      lock.includes(asString(dist.integrity)),
      "installed provider public SRI must be in the actual pnpm lock: " + name,
    );
    return { name, version, dist };
  });
  record.json(phase + ".provider.json", {
    ...inventory,
    images: inventory.installed.map((entry) => ({ ...entry, image: digest(entry.executable) })),
    origins,
    selection:
      "Installed candidates only. A declared tsrs package need not be selected ahead of the published CLI platform backend. Actual text-image observation is separate; an unobserved selection stays unknown.",
  });
  return [...new Set(inventory.installed.map((entry) => entry.executable))];
}

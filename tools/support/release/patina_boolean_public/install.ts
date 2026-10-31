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
    if (phase === "original-no-provider") {
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
export function provider(root: string, phase: Phase, record: Recorder): string | undefined {
  const manifest = readJsonRecord(join(root, "package.json"));
  const dependencies = asRecord(manifest.devDependencies);
  const stable = "@typescript/typescript-darwin-arm64";
  if (phase === "original-no-provider") {
    assert.equal(dependencies[stable], undefined);
    assert.equal(dependencies.tsrs, undefined);
    assert.equal(existsSync(join(root, "node_modules", stable)), false);
    assert.equal(existsSync(join(root, "node_modules/tsrs")), false);
    record.json(phase + ".provider.json", {
      selectedExecutable: null,
      reason:
        "The exact original role has no declared native provider under the current explicit-user-provider policy.",
    });
    return undefined;
  }
  let packageRoot: string, executable: string;
  if (phase === "stable-user-provider") {
    assert.equal(dependencies[stable], "7.0.2");
    assert.equal(dependencies.tsrs, undefined);
    packageRoot = realpathSync(join(root, "node_modules", stable));
    const metadata = readJsonRecord(join(packageRoot, "package.json"));
    assert.equal(metadata.name, stable);
    assert.equal(metadata.version, "7.0.2");
    executable = realpathSync(join(packageRoot, "lib/tsc"));
  } else {
    assert.equal(dependencies[stable], undefined);
    assert.equal(dependencies.tsrs, "0.20.1-ts7.1.0-beta");
    assert.equal(
      existsSync(join(root, "node_modules", stable)),
      false,
      "stable provider must be removed by pnpm, never hidden with a resolver override",
    );
    packageRoot = realpathSync(join(root, "node_modules/tsrs"));
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
    executable = realpathSync(join(platform, "tsrs"));
  }
  assert(inside(root, packageRoot) && inside(root, executable));
  record.json(phase + ".provider.json", {
    executable,
    image: digest(executable),
    packageRoot,
    selection:
      "Prospective expected image only; actual ps/lsof observation is separately mandatory.",
  });
  return executable;
}

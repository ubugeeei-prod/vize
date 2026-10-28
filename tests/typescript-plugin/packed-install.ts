import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

interface Manifest {
  readonly name: string;
  readonly version: string;
  readonly files: readonly string[];
  readonly [field: string]: unknown;
}

export interface PackedConsumer {
  readonly consumer: string;
  readonly native: string;
  readonly plugin: string;
  readonly pluginPath: string;
  readonly nativeFiles: readonly string[];
  readonly pluginFiles: readonly string[];
  readonly pluginManifest: Manifest;
}

export const digest = (data: string | Buffer): string =>
  createHash("sha256").update(data).digest("hex");
export const run = (command: string, args: readonly string[], cwd: string): string =>
  execFileSync(command, args, { cwd, encoding: "utf8", timeout: 90_000, maxBuffer: 8_000_000 });

/** Pack the exact-source native and plugin packages and install them in a fresh consumer. */
export function installPackedConsumer(
  root: string,
  output: string,
  temporary: string,
  typescriptVersion: string,
): PackedConsumer {
  const manifest = (name: string) =>
    JSON.parse(fs.readFileSync(path.join(root, name, "package.json"), "utf8")) as Manifest;
  const nativeManifest = manifest("npm/native");
  const pluginManifest = manifest("npm/typescript-plugin");
  function pack(name: string, packageJson: object, files: readonly [string, string][]): string {
    const stage = path.join(temporary, name);
    fs.mkdirSync(stage);
    for (const [source, target] of files) fs.copyFileSync(source, path.join(stage, target));
    fs.writeFileSync(path.join(stage, "package.json"), JSON.stringify(packageJson, null, 2));
    const report = (
      JSON.parse(
        run("npm", ["pack", "--ignore-scripts", "--json", "--pack-destination", output], stage),
      ) as { readonly filename: string }[]
    )[0];
    fs.writeFileSync(path.join(output, `${name}-pack.json`), JSON.stringify(report, null, 2));
    return path.join(output, report.filename);
  }
  const nativeFiles = fs
    .readdirSync(path.join(root, "npm/native"))
    .filter((name) => nativeManifest.files.includes(name) || name.endsWith(".node"));
  assert(
    nativeFiles.some((name) => name.endsWith(".node")),
    "Build the exact-source native binary before this test",
  );
  const native = pack(
    "native",
    {
      ...nativeManifest,
      scripts: {},
      optionalDependencies: {},
      devDependencies: {},
      files: nativeFiles,
    },
    nativeFiles.map((name) => [path.join(root, "npm/native", name), name]),
  );
  const pluginFiles = fs
    .readdirSync(path.join(root, "npm/typescript-plugin"))
    .filter((name) => /\.cjs$|\.d\.cts$|^README\.md$/.test(name));
  const plugin = pack(
    "plugin",
    { ...pluginManifest, dependencies: { "@vizejs/native": nativeManifest.version } },
    pluginFiles.map((name) => [path.join(root, "npm/typescript-plugin", name), name]),
  );
  const consumer = path.join(temporary, "consumer");
  fs.mkdirSync(consumer);
  fs.writeFileSync(
    path.join(consumer, "package.json"),
    JSON.stringify({ private: true, name: "vize-tsserver-consumer", version: "1.0.0" }),
  );
  fs.writeFileSync(
    path.join(output, "install.log"),
    run(
      "npm",
      [
        "install",
        "--ignore-scripts",
        "--no-audit",
        "--no-fund",
        native,
        plugin,
        `typescript@${typescriptVersion}`,
        "vue@3.5.35",
      ],
      consumer,
    ),
  );
  const pluginPath = path.join(consumer, "node_modules/@vizejs/typescript-vue-plugin");
  for (const name of pluginFiles)
    assert.equal(
      digest(fs.readFileSync(path.join(pluginPath, name))),
      digest(fs.readFileSync(path.join(root, "npm/typescript-plugin", name))),
    );
  return { consumer, native, plugin, pluginPath, nativeFiles, pluginFiles, pluginManifest };
}

import fs from "node:fs";
import { runEntrypoint } from "../../command-runtime.ts";

interface Options {
  packageJsonPath: string;
  versionPackageJsonPath: string;
  print: boolean;
}

function parseArgs(argv: readonly string[]): Options {
  const positional = argv.filter((argument) => argument !== "--print");
  if (positional.length < 1 || positional.length > 2) {
    throw new Error(
      "Usage: node tools/support/release/inject_native_optional_deps/main.ts <package-json> [version-package-json] [--print]",
    );
  }
  return {
    packageJsonPath: positional[0],
    versionPackageJsonPath: positional[1] ?? positional[0],
    print: argv.includes("--print"),
  };
}

function readManifest(filename: string): Record<string, unknown> {
  const value: unknown = JSON.parse(fs.readFileSync(filename, "utf8"));
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${filename}: package.json must be an object`);
  }
  return value as Record<string, unknown>;
}

function packageVersion(manifest: Readonly<Record<string, unknown>>, filename: string): string {
  if (typeof manifest.version !== "string" || manifest.version.length === 0) {
    throw new Error(`Failed to read version from ${filename}`);
  }
  return manifest.version;
}

export function injectNativeOptionalDependencyVersions(
  manifest: Record<string, unknown>,
  version: string,
): void {
  const dependencies = manifest.optionalDependencies;
  if (dependencies === null || typeof dependencies !== "object" || Array.isArray(dependencies)) {
    return;
  }
  for (const name of Object.keys(dependencies)) {
    if (name.startsWith("@vizejs/native-")) {
      (dependencies as Record<string, unknown>)[name] = version;
    }
  }
}

runEntrypoint(() => {
  const options = parseArgs(process.argv.slice(2));
  const manifest = readManifest(options.packageJsonPath);
  const version = packageVersion(
    readManifest(options.versionPackageJsonPath),
    options.versionPackageJsonPath,
  );
  injectNativeOptionalDependencyVersions(manifest, version);
  fs.writeFileSync(options.packageJsonPath, `${JSON.stringify(manifest, null, 2)}\n`);
  console.log(`Updated optionalDependencies to native version ${version}`);
  if (options.print) console.log(JSON.stringify(manifest, null, 2));
});

import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const rootRequire = createRequire(path.join(root, "package.json"));
const shikiPackageJsonPath = rootRequire.resolve("shiki/package.json");
const shikiRequire = createRequire(shikiPackageJsonPath);

export const textmateModulePath = shikiRequire.resolve("@shikijs/vscode-textmate");
export const onigurumaModulePath = shikiRequire.resolve("@shikijs/engine-oniguruma");
export const onigurumaWasmPath = path.join(path.dirname(shikiPackageJsonPath), "dist", "onig.wasm");
export const textmateDependencyVersions = {
  engineOniguruma: packageVersion(onigurumaModulePath, "@shikijs/engine-oniguruma"),
  shiki: JSON.parse(fs.readFileSync(shikiPackageJsonPath, "utf8")).version as string,
  vscodeTextmate: packageVersion(textmateModulePath, "@shikijs/vscode-textmate"),
};

export function shikiLanguageModulePath(name: string): string {
  return shikiRequire.resolve(`@shikijs/langs/${name}`);
}

function packageVersion(modulePath: string, expectedName: string): string {
  let directory = path.dirname(modulePath);
  while (directory !== path.dirname(directory)) {
    const candidate = path.join(directory, "package.json");
    if (fs.existsSync(candidate)) {
      const manifest = JSON.parse(fs.readFileSync(candidate, "utf8")) as {
        name?: string;
        version?: string;
      };
      if (manifest.name === expectedName && manifest.version != null) return manifest.version;
    }
    directory = path.dirname(directory);
  }
  throw new Error(`could not resolve ${expectedName} package version from ${modulePath}`);
}

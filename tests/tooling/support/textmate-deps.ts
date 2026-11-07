import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const testsRequire = createRequire(path.join(root, "tests/package.json"));

export const textmateModulePath = testsRequire.resolve("vscode-textmate");
export const onigurumaModulePath = testsRequire.resolve("vscode-oniguruma");
const onigurumaPackageJsonPath = testsRequire.resolve("vscode-oniguruma/package.json");
export const onigurumaWasmPath = path.join(
  path.dirname(onigurumaPackageJsonPath),
  "release/onig.wasm",
);
export const textmateDependencyVersions = {
  vscodeOniguruma: packageVersion(onigurumaModulePath, "vscode-oniguruma"),
  vscodeTextmate: packageVersion(textmateModulePath, "vscode-textmate"),
};

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

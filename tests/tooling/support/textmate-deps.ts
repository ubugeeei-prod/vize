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

export function shikiLanguageModulePath(name: string): string {
  return shikiRequire.resolve(`@shikijs/langs/${name}`);
}

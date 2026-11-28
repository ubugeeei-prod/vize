import { spawnSync } from "node:child_process";
import { readFileSync, mkdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = fileURLToPath(new URL("../", import.meta.url));
const repositoryRoot = path.resolve(packageRoot, "../..");
const target =
  process.arch === "arm64"
    ? "aarch64-apple-darwin"
    : process.arch === "x64"
      ? "x86_64-apple-darwin"
      : undefined;
if (process.platform !== "darwin" || target === undefined)
  throw new Error("The first native host requires a supported macOS build host.");
const nativeDirectory = path.join(repositoryRoot, "npm/native");
const nativePackage = JSON.parse(
  readFileSync(path.join(nativeDirectory, "package.json"), "utf8"),
) as { readonly version: string };
const receipt =
  process.env.VIZE_NATIVE_BUILD_RECEIPT ?? path.join(packageRoot, ".build/compiler-build.json");
mkdirSync(path.dirname(receipt), { recursive: true });
const result = spawnSync(
  process.execPath,
  [
    path.join(repositoryRoot, "tools/support/github/build-napi-package.ts"),
    nativeDirectory,
    path.join(repositoryRoot, "crates/vize_vitrine/Cargo.toml"),
    "vize_vitrine",
    target,
  ],
  {
    cwd: repositoryRoot,
    env: {
      ...process.env,
      VIZE_RELEASE_VERSION: nativePackage.version,
      VIZE_NATIVE_BUILD_RECEIPT: receipt,
    },
    stdio: "inherit",
  },
);
if (result.error) throw result.error;
if (result.status !== 0) process.exitCode = result.status ?? 1;

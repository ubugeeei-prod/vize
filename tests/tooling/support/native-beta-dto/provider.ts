import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
const { SDK_VERSION, SDK_SHA256, SDK_ROOT, VIZE_NATIVE_DTO_CAPTURE, GITHUB_ENV } =
  process.env as NodeJS.ProcessEnv &
    Record<
      "SDK_VERSION" | "SDK_SHA256" | "SDK_ROOT" | "VIZE_NATIVE_DTO_CAPTURE" | "GITHUB_ENV",
      string
    >;
const sha = (value: Uint8Array) => createHash("sha256").update(value).digest("hex");
const pkg = (name: string) => path.join(SDK_ROOT, "node_modules", name, "package.json");
const main = JSON.parse(fs.readFileSync(pkg("typescript"), "utf8")) as Record<string, unknown>;
const platform = JSON.parse(
  fs.readFileSync(pkg("@typescript/typescript-linux-x64"), "utf8"),
) as Record<string, unknown>;
assert.equal(main.version, SDK_VERSION);
assert.equal(platform.version, SDK_VERSION);
const executable = fs.realpathSync(
  path.join(SDK_ROOT, "node_modules/@typescript/typescript-linux-x64/lib/tsc"),
);
assert.equal(sha(fs.readFileSync(executable)), SDK_SHA256);
assert.equal(
  execFileSync(executable, ["--version"], { encoding: "utf8" }),
  `Version ${SDK_VERSION}\n`,
);
fs.copyFileSync(
  path.join(SDK_ROOT, "package-lock.json"),
  path.join(VIZE_NATIVE_DTO_CAPTURE, "provider-package-lock.json"),
);
const metadata = {
  version: SDK_VERSION,
  executable,
  executableSha256: SDK_SHA256,
  main,
  platform,
  packageHashes: {
    typescript: sha(fs.readFileSync(pkg("typescript"))),
    platform: sha(fs.readFileSync(pkg("@typescript/typescript-linux-x64"))),
  },
};
fs.writeFileSync(
  path.join(VIZE_NATIVE_DTO_CAPTURE, "provider.json"),
  JSON.stringify(metadata, null, 2) + "\n",
);
for (const [name, value] of Object.entries({
  VIZE_NATIVE_DTO_RUNTIME_PATH: executable,
  VIZE_NATIVE_DTO_ACTUAL_RUNTIME_PATH: executable,
  VIZE_NATIVE_DTO_SDK_VERSION: SDK_VERSION,
  VIZE_NATIVE_DTO_SDK_SHA256: SDK_SHA256,
}))
  fs.appendFileSync(GITHUB_ENV, `${name}=${value}\n`);

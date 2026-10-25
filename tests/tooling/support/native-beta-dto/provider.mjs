import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
const { SDK_VERSION, SDK_SHA256, SDK_ROOT, VIZE_NATIVE_DTO_CAPTURE, GITHUB_ENV } = process.env;
const sha = (value) => createHash("sha256").update(value).digest("hex");
const pkg = (name) => path.join(SDK_ROOT, "node_modules", name, "package.json");
const main = JSON.parse(fs.readFileSync(pkg("typescript"), "utf8"));
const platform = JSON.parse(fs.readFileSync(pkg("@typescript/typescript-linux-x64"), "utf8"));
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
const wrapper = path.join(VIZE_NATIVE_DTO_CAPTURE, "actual-native");
// Each actual process is traced separately. The wrapper emits no protocol bytes.
fs.writeFileSync(
  wrapper,
  '#!/bin/sh\nexec strace -ff -ttt -s 16777216 -xx -e trace=read,write,readv,writev,process -o "$VIZE_NATIVE_DTO_CAPTURE/syscalls" "$VIZE_NATIVE_DTO_ACTUAL_RUNTIME_PATH" "$@"\n',
  { mode: 0o755 },
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
  VIZE_NATIVE_DTO_RUNTIME_PATH: wrapper,
  VIZE_NATIVE_DTO_ACTUAL_RUNTIME_PATH: executable,
  VIZE_NATIVE_DTO_SDK_VERSION: SDK_VERSION,
  VIZE_NATIVE_DTO_SDK_SHA256: SDK_SHA256,
}))
  fs.appendFileSync(GITHUB_ENV, `${name}=${value}\n`);

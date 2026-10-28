import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { randomBytes } from "node:crypto";
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { asArray, asRecord, asString, parseJson } from "../../json-value.ts";
import { cleanEnv, npmArgv, Recorder, sha256, success } from "./common.ts";
import { runLoaderUnits } from "./units.ts";

const [sourceArg, outputArg, evidenceArg] = process.argv.slice(2);
assert.equal(process.env.GITHUB_ACTIONS, "true", "source SDK production runs only in Actions");
assert.equal(process.version, "v24.22.1");
assert.ok(sourceArg && outputArg && evidenceArg);
const source = resolve(sourceArg),
  output = resolve(outputArg);
mkdirSync(output, { recursive: true });
const record = new Recorder(evidenceArg);
const head = record.run("source-head", ["git", "rev-parse", "HEAD"], source);
success(head);
assert.equal(head.stdout.trim(), process.env.SOURCE_SHA);
runLoaderUnits(source, join(record.root, "loader-suite"));
const token = randomBytes(16).toString("hex");
const env = { ...cleanEnv(), VIZE_NATIVE_BUILD_TOKEN: token };
success(
  record.run(
    "source-native-and-cli-build",
    [
      "cargo",
      "build",
      "--locked",
      "--profile",
      "ci",
      "-p",
      "vize",
      "-p",
      "vize_vitrine",
      "--features",
      "vize_vitrine/napi",
    ],
    source,
    env,
    1_800_000,
  ),
);
const target = process.platform === "win32" ? "win32-x64-msvc" : "linux-x64-gnu";
assert.equal(process.arch, "x64");
const image = join(
  source,
  "target/ci",
  process.platform === "win32" ? "vize_vitrine.dll" : "libvize_vitrine.so",
);
const cli = join(source, "target/ci", process.platform === "win32" ? "vize.exe" : "vize");
const stage = join(output, "host-package");
mkdirSync(stage, { recursive: true });
const filename = "vize-vitrine." + target + ".node";
copyFileSync(image, join(stage, filename));
const require = createRequire(import.meta.url);
const native: { getNativeBuildToken(): string; runCli(args: string[]): void } = require(
  join(stage, filename),
);
assert.equal(native.getNativeBuildToken(), token);
assert.equal(typeof native.runCli, "function");
const version = record.run("source-cli-version", [cli, "--version"], source);
success(version);
const sourceVersion = version.stdout.trim().replace(/^vize /, "");
writeFileSync(
  join(stage, "package.json"),
  JSON.stringify(
    {
      name: "@vizejs/native-" + target,
      version: "0.601.0",
      main: filename,
      files: [filename],
      os: [process.platform],
      cpu: ["x64"],
      license: "MIT",
      description:
        "Source-image SDK control: published 0.601 wrappers with an exact reviewed source image; not a public release.",
    },
    null,
    2,
  ) + "\n",
);
const packed = record.run(
  "source-image-sdk-pack",
  npmArgv("pack", "--ignore-scripts", "--json", "--pack-destination", output),
  stage,
);
success(packed);
const pack = asRecord(asArray(parseJson(packed.stdout))[0]);
const tarball = join(output, asString(pack.filename));
const receipt = {
  kind: "source-image-sdk",
  source: head.stdout.trim(),
  wrapperVersion: "0.601.0",
  sourceVersion,
  package: "@vizejs/native-" + target,
  target,
  tarball,
  integrity: asString(pack.integrity),
  nativeBuildToken: token,
  nodeFile: filename,
  nativeBytes: readFileSync(image).length,
  nativeSha256: sha256(readFileSync(image)),
  cliBinary: cli,
  cliSha256: sha256(readFileSync(cli)),
  policy:
    "Original published JS/CLI/native wrappers stay unchanged; only the host npm image is an exact source build. Normal package identity guards remain active. This is not public acceptance.",
};
record.json("source-image-sdk.json", receipt);
writeFileSync(join(output, "source-image-sdk.json"), JSON.stringify(receipt, null, 2) + "\n");
console.log(JSON.stringify({ sourceImageSdk: receipt.source, target, tarball }));

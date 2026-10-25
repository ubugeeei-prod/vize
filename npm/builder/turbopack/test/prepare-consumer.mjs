import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawn } from "node:child_process";
import { createRequire } from "node:module";
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { sourceNativeResolutions } from "./consumer-lock.mjs";

export const packageRoot = fileURLToPath(new URL("../", import.meta.url));
export const repoRoot = path.resolve(packageRoot, "../../..");
export const output = path.resolve(
  repoRoot,
  process.env.VIZE_TURBOPACK_ARTIFACTS ??
    `target/turbopack-consumers/${process.env.GITHUB_RUN_ID ?? "local"}-${process.env.GITHUB_RUN_ATTEMPT ?? "1"}`,
);
export const consumer = path.join(output, "consumer");
const requireNative = createRequire(path.join(repoRoot, "npm/native/package.json"));
export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

export async function assertBundledHelpers(directory) {
  for (const entry of await fs.readdir(directory, { withFileTypes: true })) {
    const filename = path.join(directory, entry.name);
    if (entry.isDirectory()) await assertBundledHelpers(filename);
    else {
      const content = await fs.readFile(filename, "utf8");
      assert.doesNotMatch(content, /(?:from|require\(|import\()\s*["']@vizejs\/builder-shared/);
    }
  }
}

export async function run(name, executable, args, cwd = repoRoot, extraEnv = {}) {
  const start = performance.now();
  const result = await new Promise((resolve, reject) => {
    const process = spawn(executable, args, {
      cwd,
      env: { ...globalThis.process.env, ...extraEnv },
      stdio: ["ignore", "pipe", "pipe"],
    });
    const stdout = [];
    const stderr = [];
    process.stdout.on("data", (data) => stdout.push(data));
    process.stderr.on("data", (data) => stderr.push(data));
    process.once("error", reject);
    process.once("close", (status, signal) =>
      resolve({ status, signal, stdout: Buffer.concat(stdout), stderr: Buffer.concat(stderr) }),
    );
  });
  const receipt = {
    command: [executable, ...args],
    cwd,
    status: result.status,
    signal: result.signal,
    elapsedMs: performance.now() - start,
  };
  await fs.writeFile(path.join(output, `${name}.stdout`), result.stdout);
  await fs.writeFile(path.join(output, `${name}.stderr`), result.stderr);
  await fs.writeFile(path.join(output, `${name}.json`), JSON.stringify(receipt, null, 2) + "\n");
  assert.equal(result.status, 0, `${name} failed:\n${result.stderr}\n${result.stdout}`);
  return { ...result, ...receipt };
}

export async function prepareConsumer() {
  await fs.mkdir(output, { recursive: true });
  await assert.rejects(
    fs.stat(consumer),
    { code: "ENOENT" },
    "each source qualification uses a fresh consumer directory",
  );
  const revision = (await run("source-revision", "git", ["rev-parse", "HEAD"])).stdout
    .toString()
    .trim();
  assert.equal(revision, process.env.GITHUB_SHA);
  const nativeRoot = path.join(repoRoot, "npm/native");
  const sourceReceipt = JSON.parse(
    await fs.readFile(path.join(nativeRoot, ".artifacts/native/build-token-receipt.json"), "utf8"),
  );
  assert.equal(sourceReceipt.source, revision);
  const native = requireNative("./index.js");
  assert.equal(native.getNativeBuildToken(), sourceReceipt.identity.token);
  const binary = await fs.readFile(path.join(nativeRoot, sourceReceipt.binary.filename));
  assert.equal(sha256(binary), sourceReceipt.binary.sha256);
  const packs = path.join(output, "packs");
  await fs.mkdir(packs, { recursive: true });
  const nativeStage = path.join(output, "native");
  await fs.mkdir(nativeStage);
  const nativeManifest = JSON.parse(
    await fs.readFile(path.join(nativeRoot, "package.json"), "utf8"),
  );
  for (const file of nativeManifest.files)
    await fs.copyFile(path.join(nativeRoot, file), path.join(nativeStage, file));
  await fs.writeFile(path.join(nativeStage, sourceReceipt.binary.filename), binary);
  const manifest = {
    ...nativeManifest,
    files: [...nativeManifest.files, sourceReceipt.binary.filename],
  };
  // A selected-host source consumer carries the actual built image; registries are not a fallback.
  delete manifest.optionalDependencies;
  delete manifest.devDependencies;
  delete manifest.scripts;
  await fs.writeFile(
    path.join(nativeStage, "package.json"),
    JSON.stringify(manifest, null, 2) + "\n",
  );
  const nativePack = await run(
    "pack-native",
    "npm",
    ["pack", "--json", "--pack-destination", packs],
    nativeStage,
  );
  const nativeTar = path.join(packs, JSON.parse(nativePack.stdout.toString())[0].filename);
  const tarballs = { "@vizejs/native": nativeTar };
  for (const [name, directory] of [
    ["turbopack", packageRoot],
    ["rspack", path.join(packageRoot, "../rspack")],
  ]) {
    const before = new Set(await fs.readdir(packs));
    await run(`pack-${name}`, "vp", [
      "exec",
      "pnpm",
      "--dir",
      directory,
      "pack",
      "--out",
      path.join(packs, `${name}.tgz`),
    ]);
    const added = (await fs.readdir(packs)).filter(
      (file) => !before.has(file) && file.endsWith(".tgz"),
    );
    assert.equal(added.length, 1);
    tarballs[`@vizejs/${name}-plugin`] = path.join(packs, added[0]);
    const packedManifest = JSON.parse(
      (
        await run(`manifest-${name}`, "tar", [
          "-xOf",
          tarballs[`@vizejs/${name}-plugin`],
          "package/package.json",
        ])
      ).stdout.toString(),
    );
    assert.equal(packedManifest.dependencies["@vizejs/builder-shared"], undefined);
    for (const section of ["dependencies", "optionalDependencies", "peerDependencies"]) {
      assert.ok(
        Object.values(packedManifest[section] ?? {}).every(
          (value) => !/^(workspace|catalog):/.test(value),
        ),
      );
    }
  }
  await fs.cp(path.join(packageRoot, "test/fixtures"), consumer, { recursive: true });
  await fs.writeFile(
    path.join(consumer, "package.json"),
    JSON.stringify(
      {
        private: true,
        type: "module",
        dependencies: {
          ...Object.fromEntries(
            Object.entries(tarballs).map(([name, file]) => [name, `file:${file}`]),
          ),
          "@rspack/core": "2.1.8",
          next: "16.4.3",
          react: "19.2.5",
          "react-dom": "19.2.5",
          vue: "3.5.44",
          "sass-loader": "17.0.1",
          sass: "1.106.1",
        },
      },
      null,
      2,
    ) + "\n",
  );
  await run("install-consumer", "npm", ["install", "--no-audit", "--no-fund"], consumer);
  const installed = createRequire(path.join(consumer, "package.json"));
  const loadedNative = installed("@vizejs/native");
  assert.equal(loadedNative.getNativeBuildToken(), sourceReceipt.identity.token);
  assert.ok(installed.resolve("@vizejs/native").startsWith(consumer + path.sep));
  assert.equal(
    sha256(
      await fs.readFile(
        path.join(path.dirname(installed.resolve("@vizejs/native")), sourceReceipt.binary.filename),
      ),
    ),
    sourceReceipt.binary.sha256,
  );
  assert.throws(() => installed.resolve("@vizejs/builder-shared"));
  assert.equal(typeof installed("@vizejs/turbopack-plugin").withVize, "function");
  assert.equal(typeof installed("@vizejs/turbopack-plugin/loader").default, "function");
  assert.equal(typeof installed("@vizejs/rspack-plugin").VizePlugin, "function");
  for (const entry of [
    "@vizejs/turbopack-plugin/loader",
    "@vizejs/turbopack-plugin/scope-loader",
    "@vizejs/rspack-plugin",
  ]) {
    const issuer = createRequire(installed.resolve(entry));
    assert.equal(issuer.resolve("@vizejs/native"), installed.resolve("@vizejs/native"));
    assert.equal(issuer("@vizejs/native").getNativeBuildToken(), sourceReceipt.identity.token);
  }
  for (const name of ["turbopack", "rspack"]) {
    const directory = path.join(consumer, `node_modules/@vizejs/${name}-plugin/dist`);
    await assertBundledHelpers(directory);
  }
  await fs.writeFile(
    path.join(consumer, "public-types.mts"),
    `import {withVize} from '@vizejs/turbopack-plugin';\nimport {compileTurbopackSfc} from '@vizejs/turbopack-plugin/loader';\nconst config=withVize({compiler:{define:{__VUE_OPTIONS_API__:false}}});\nconst r=compileTurbopackSfc('/fixture/App.vue','<template/>',{ssr:false,isProduction:true,styleLanguages:['css']});\nif(r.map){const mappings:string=r.map.mappings;}\nvoid config;\n`,
  );
  await run(
    "public-types",
    process.execPath,
    [
      path.join(packageRoot, "node_modules/typescript/bin/tsc"),
      "--ignoreConfig",
      "--noEmit",
      "--strict",
      "--skipLibCheck",
      "--target",
      "ES2022",
      "--module",
      "NodeNext",
      "--moduleResolution",
      "NodeNext",
      "public-types.mts",
    ],
    consumer,
  );
  // A hoisted install can hide missing public runtime dependencies. Resolve the
  // same tarballs from a standalone pnpm consumer with isolated node_modules.
  const strictConsumer = path.join(output, "strict-consumer");
  await fs.mkdir(strictConsumer);
  const strictManifest = JSON.parse(await fs.readFile(path.join(consumer, "package.json"), "utf8"));
  strictManifest.packageManager = JSON.parse(
    await fs.readFile(path.join(repoRoot, "package.json"), "utf8"),
  ).packageManager;
  await fs.writeFile(
    path.join(strictConsumer, "package.json"),
    JSON.stringify(strictManifest, null, 2) + "\n",
  );
  await fs.writeFile(
    path.join(strictConsumer, "pnpm-workspace.yaml"),
    `packages: ["."]\noverrides:\n  "@vizejs/native": ${JSON.stringify(`file:${nativeTar}`)}\n`,
  );
  for (const frozen of [false, true]) {
    await run(frozen ? "strict-install-frozen" : "strict-install", "vp", [
      "exec",
      "pnpm",
      "--dir",
      strictConsumer,
      "install",
      "--ignore-scripts",
      frozen ? "--frozen-lockfile" : "--no-frozen-lockfile",
      "--strict-peer-dependencies",
    ]);
  }
  const strictLockBytes = await fs.readFile(path.join(strictConsumer, "pnpm-lock.yaml"));
  const nativeIntegrity = `sha512-${createHash("sha512")
    .update(await fs.readFile(nativeTar))
    .digest("base64")}`;
  const nativeResolutions = sourceNativeResolutions(
    strictLockBytes.toString(),
    strictConsumer,
    nativeTar,
    nativeIntegrity,
  );
  const strictRequire = createRequire(path.join(strictConsumer, "package.json"));
  const strictNative = strictRequire.resolve("@vizejs/native");
  const strictEntries = {};
  for (const [entry, exported] of [
    ["@vizejs/turbopack-plugin", "withVize"],
    ["@vizejs/turbopack-plugin/loader", "default"],
    ["@vizejs/turbopack-plugin/style-loader", "default"],
    ["@vizejs/turbopack-plugin/scope-loader", "default"],
    ["@vizejs/rspack-plugin", "VizePlugin"],
    ["@vizejs/rspack-plugin/loader", "default"],
  ]) {
    const resolved = strictRequire.resolve(entry);
    assert.ok(resolved.startsWith(strictConsumer + path.sep));
    const loaded = strictRequire(entry);
    assert.equal(
      typeof (typeof loaded === "function" ? loaded : loaded[exported]),
      "function",
      `${entry} exposes its public callable`,
    );
    const issuer = createRequire(resolved);
    assert.equal(issuer("webpack-sources/package.json").version, "3.5.1");
    assert.equal(issuer.resolve("@vizejs/native"), strictNative);
    assert.equal(issuer("@vizejs/native").getNativeBuildToken(), sourceReceipt.identity.token);
    assert.throws(() => issuer.resolve("@vizejs/builder-shared"));
    strictEntries[entry] = {
      file: path.relative(strictConsumer, resolved),
      webpackSources: issuer("webpack-sources/package.json").version,
    };
  }
  assert.equal(
    sha256(await fs.readFile(path.join(path.dirname(strictNative), sourceReceipt.binary.filename))),
    sourceReceipt.binary.sha256,
  );
  await fs.copyFile(
    path.join(consumer, "public-types.mts"),
    path.join(strictConsumer, "public-types.mts"),
  );
  await run(
    "strict-public-types",
    process.execPath,
    [
      path.join(packageRoot, "node_modules/typescript/bin/tsc"),
      "--ignoreConfig",
      "--noEmit",
      "--strict",
      "--skipLibCheck",
      "--target",
      "ES2022",
      "--module",
      "NodeNext",
      "--moduleResolution",
      "NodeNext",
      "public-types.mts",
    ],
    strictConsumer,
  );
  await fs.writeFile(
    path.join(output, "strict-resolution.json"),
    JSON.stringify(
      {
        entries: strictEntries,
        nativeSha256: sourceReceipt.binary.sha256,
        nativeResolutions,
        lockSha256: sha256(strictLockBytes),
      },
      null,
      2,
    ) + "\n",
  );
  const receipt = {
    source: revision,
    native: sourceReceipt,
    tarballs: Object.fromEntries(
      await Promise.all(
        Object.entries(tarballs).map(async ([name, file]) => [
          name,
          { file: path.basename(file), sha256: sha256(await fs.readFile(file)) },
        ]),
      ),
    ),
    runtimes: Object.fromEntries(
      ["@rspack/core", "next", "react", "react-dom", "vue"].map((name) => [
        name,
        installed(`${name}/package.json`).version,
      ]),
    ),
  };
  await fs.writeFile(
    path.join(output, "consumer-receipt.json"),
    JSON.stringify(receipt, null, 2) + "\n",
  );
  return receipt;
}

// Pack and install outside the workspace, then exercise only installed exports.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

function run(command: string, args: string[], cwd: string, env: NodeJS.ProcessEnv) {
  const result = spawnSync(command, args, { cwd, env, encoding: "utf8", timeout: 120_000 });
  assert.equal(
    result.status,
    0,
    `${command} ${args.join(" ")}\n${result.error ?? ""}\n${result.stderr}\n${result.stdout}`,
  );
  return result.stdout;
}

export function verifyStandalonePluginSdk(
  root: string,
  nativeInterop = true,
  nativeTypesPath = path.join(root, "npm/native/index.d.ts"),
  installPolicy: "offline" | "default" = "offline",
) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-plugin-sdk-install-"));
  const env = {
    ...process.env,
    npm_config_registry:
      installPolicy === "offline" ? "http://127.0.0.1:9" : "https://registry.npmjs.org",
    npm_config_fetch_retries: "0",
    npm_config_cache: path.join(directory, "npm-cache"),
    npm_config_userconfig: path.join(directory, "user.npmrc"),
    npm_config_globalconfig: path.join(directory, "global.npmrc"),
  };
  delete env.NODE_PATH;
  delete env.NODE_TEST_CONTEXT;
  try {
    assert.ok(!directory.startsWith(`${fs.realpathSync(root)}${path.sep}`));
    fs.writeFileSync(env.npm_config_userconfig, "");
    fs.writeFileSync(env.npm_config_globalconfig, "");
    fs.writeFileSync(
      path.join(directory, "package.json"),
      JSON.stringify({ name: "standalone-plugin-author", private: true, type: "module" }),
    );
    const packed = JSON.parse(
      run(
        "npm",
        [
          "pack",
          path.join(root, "npm/plugin-sdk"),
          "--json",
          "--ignore-scripts",
          "--pack-destination",
          directory,
        ],
        directory,
        env,
      ),
    );
    const [artifact] = packed;
    assert.ok(artifact, "npm pack must produce a tarball");
    const tarball = path.join(directory, artifact.filename);
    run(
      "npm",
      [
        "install",
        tarball,
        "--ignore-scripts",
        "--no-audit",
        "--no-fund",
        "--package-lock=false",
        ...(installPolicy === "offline" ? ["--omit=peer", "--legacy-peer-deps", "--offline"] : []),
      ],
      directory,
      env,
    );
    const installed = path.join(directory, "node_modules/@vizejs/plugin-sdk");
    assert.equal(fs.lstatSync(installed).isSymbolicLink(), false);
    assert.equal(fs.existsSync(path.join(directory, "node_modules/@vizejs/native")), false);
    const manifest = JSON.parse(fs.readFileSync(path.join(installed, "package.json"), "utf8"));
    assert.notEqual(manifest.private, true);
    assert.equal(manifest.publishConfig.access, "public");
    assert.equal(manifest.peerDependenciesMeta["@vizejs/native"].optional, true);
    assert.match(manifest.peerDependencies["@vizejs/native"], /^>=0\.429\.0$/);
    assert.deepEqual(Object.keys(manifest.dependencies ?? {}), []);
    assert.deepEqual(Object.keys(manifest.optionalDependencies ?? {}), []);
    for (const entry of [
      "index.js",
      "index.d.ts",
      "facts.d.ts",
      "fixes.js",
      "fixes.d.ts",
      "sandbox.js",
      "sandbox.d.ts",
      "sandbox-worker.js",
      "stages.js",
      "stages.d.ts",
      "visitor.js",
      "visitor.d.ts",
      "identity.js",
    ]) {
      assert.ok(fs.existsSync(path.join(installed, entry)), `tarball lacks ${entry}`);
    }
    const fixtures = fileURLToPath(new URL("../fixtures/davinci-plugin-sdk", import.meta.url));
    fs.copyFileSync(path.join(fixtures, "packed-runtime.mjs"), path.join(directory, "runtime.mjs"));
    const runtime = run(process.execPath, ["runtime.mjs"], directory, env).trim();
    const require = createRequire(path.join(root, "tests/package.json"));
    const compiler = require.resolve("typescript/lib/tsc.js");
    fs.copyFileSync(
      path.join(fixtures, "packed-authoring.ts"),
      path.join(directory, "packed-authoring.ts"),
    );
    const compilerOptions = {
      strict: true,
      skipLibCheck: false,
      noEmit: true,
      target: "ES2022",
      lib: ["ESNext"],
      module: "NodeNext",
      moduleResolution: "NodeNext",
      types: [],
    };
    fs.writeFileSync(
      path.join(directory, "tsconfig.json"),
      JSON.stringify({ compilerOptions, files: ["packed-authoring.ts"] }),
    );
    run(process.execPath, [compiler, "--project", "tsconfig.json"], directory, env);
    if (nativeInterop) {
      const peer = path.join(directory, "node_modules/@vizejs/native");
      fs.mkdirSync(peer, { recursive: true });
      fs.copyFileSync(nativeTypesPath, path.join(peer, "index.d.ts"));
      const native = JSON.parse(
        fs.readFileSync(path.join(root, "npm/native/package.json"), "utf8"),
      );
      fs.writeFileSync(
        path.join(peer, "package.json"),
        JSON.stringify({ name: native.name, version: native.version, types: "index.d.ts" }),
      );
      for (const dependency of ["@types/node", "undici-types"]) {
        const resolver =
          dependency === "undici-types"
            ? createRequire(require.resolve("@types/node/package.json"))
            : require;
        const source = path.dirname(resolver.resolve(`${dependency}/package.json`));
        fs.cpSync(source, path.join(directory, "node_modules", dependency), { recursive: true });
      }
      fs.copyFileSync(
        path.join(fixtures, "packed-native-interop.ts"),
        path.join(directory, "packed-native-interop.ts"),
      );
      fs.writeFileSync(
        path.join(directory, "tsconfig.json"),
        JSON.stringify({
          compilerOptions: { ...compilerOptions, types: ["node"] },
          files: ["packed-authoring.ts", "packed-native-interop.ts"],
        }),
      );
      run(process.execPath, [compiler, "--project", "tsconfig.json"], directory, env);
    }
    return {
      package: artifact.name,
      version: artifact.version,
      runtime,
      declarations: true,
      installPolicy,
      nativeInterop,
    };
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

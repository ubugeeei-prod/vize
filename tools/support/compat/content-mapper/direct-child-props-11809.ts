import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";
import { preparePublishManifest } from "../../release/smoke_release_install/prepare.ts";
import { verifyHelperAbiFallbacks } from "./direct-child-helper-abi-11809.ts";
import {
  roles,
  props,
  states,
  assertInventory,
  expectedOutput,
  assertOutput,
  identity,
  physicalHost,
  archiveIdentity,
  lockedPackage,
  sha256,
  writeJson,
  run,
  successful,
  assertNativeObservation,
  type NativeObservation,
} from "./direct-child-props-runtime-11809.ts";

const repository = fileURLToPath(new URL("../../../../", import.meta.url));
const fixture = path.join(
  repository,
  "tests/_fixtures/differential/typechecker/direct-child-props-11809",
);
const nativeDirectory = path.join(repository, "npm/native");
const observer = fileURLToPath(new URL("./direct-child-props-runtime-11809.ts", import.meta.url));
const overrides = [
  "NODE_PATH",
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
  "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
  "VIZE_PREFER_WORKSPACE_BINDING",
  "VIZE_PUBLIC_NATIVE_CUSTODY",
];
const cleanEnvironment = () => {
  for (const key of overrides) assert.ok(!process.env[key], `${key} must be absent`);
  const env = { ...process.env };
  delete env.NODE_OPTIONS;
  delete env.VIZE_TEST_CHILD_PROPS_TRACE;
  return env;
};

function stagePackages(directory: string, image: string) {
  fs.mkdirSync(directory, { recursive: true });
  const platform = path.join(directory, "platform");
  fs.mkdirSync(platform);
  const nativeManifest = JSON.parse(
    fs.readFileSync(path.join(nativeDirectory, "package.json"), "utf8"),
  ) as { version: string };
  fs.copyFileSync(image, path.join(platform, path.basename(image)));
  writeJson(path.join(platform, "package.json"), {
    name: "@vizejs/native-linux-x64-gnu",
    version: nativeManifest.version,
    main: path.basename(image),
    files: [path.basename(image)],
    os: ["linux"],
    cpu: ["x64"],
    libc: ["glibc"],
  });
  for (const [name, source] of [
    ["native", nativeDirectory],
    ["cli", path.join(repository, "npm/cli")],
  ] as const) {
    const destination = path.join(directory, name);
    fs.mkdirSync(destination);
    const manifest = JSON.parse(fs.readFileSync(path.join(source, "package.json"), "utf8")) as {
      files: string[];
    };
    fs.copyFileSync(path.join(source, "package.json"), path.join(destination, "package.json"));
    for (const member of manifest.files)
      fs.cpSync(path.join(source, member), path.join(destination, member), { recursive: true });
    preparePublishManifest(destination);
  }
  return [platform, path.join(directory, "native"), path.join(directory, "cli")];
}

export function main(args = process.argv.slice(2)) {
  assert.equal(args.length, 2);
  assert.equal(args[0], "--capture");
  const capture = args[1];
  assert.ok(capture);
  const output = path.resolve(capture);
  fs.mkdirSync(output, { recursive: true });
  const cells: string[] = [];
  let qualified = false;
  try {
    assert.match(process.version, /^v24\./u, "existing source producer/controller uses Node24");
    assert.equal(process.platform, "linux");
    assert.equal(process.arch, "x64");
    const source = execFileSync("git", ["rev-parse", "HEAD"], {
      cwd: repository,
      encoding: "utf8",
    }).trim();
    assert.equal(source, process.env.GITHUB_SHA);
    const receiptPath = path.join(nativeDirectory, ".artifacts/native/build-token-receipt.json");
    const producer = JSON.parse(fs.readFileSync(receiptPath, "utf8")) as {
      source: string;
      node: string;
      platform: string;
      arch: string;
      identity: { kind: string; token: string };
      binary: { filename: string; sha256: string };
    };
    assert.equal(producer.source, source);
    assert.equal(producer.node, process.version);
    assert.equal(producer.platform, "linux");
    assert.equal(producer.arch, "x64");
    assert.equal(producer.identity.kind, "native");
    assert.match(producer.identity.token, /^[a-f0-9]{32}$/u);
    const image = path.join(nativeDirectory, producer.binary.filename);
    assert.equal(identity(image).sha256, producer.binary.sha256);
    fs.copyFileSync(receiptPath, path.join(output, "producer-native.json"));
    writeJson(path.join(output, "source.json"), {
      source,
      tree: execFileSync("git", ["rev-parse", "HEAD^{tree}"], {
        cwd: repository,
        encoding: "utf8",
      }).trim(),
      workingDiff: sha256(execFileSync("git", ["diff", "--binary", "HEAD"], { cwd: repository })),
      controller: identity(process.execPath),
      producerNode: process.version,
    });
    const env = cleanEnvironment();
    const npmCli = fs.realpathSync(
      path.join(path.dirname(process.execPath), "../lib/node_modules/npm/bin/npm-cli.js"),
    );
    const tools = path.join(output, "tools");
    successful(
      run(
        process.execPath,
        [
          npmCli,
          "install",
          "--prefix",
          tools,
          "--ignore-scripts",
          "--no-audit",
          "--no-fund",
          "--save-exact",
          "node-linux-x64@22.23.3",
        ],
        repository,
        path.join(output, "install-node"),
        env,
      ),
    );
    const node = identity(path.join(tools, "node_modules/node-linux-x64/bin/node"));
    const nodeManifest = JSON.parse(
      fs.readFileSync(path.join(tools, "node_modules/node-linux-x64/package.json"), "utf8"),
    ) as { version: string };
    assert.equal(nodeManifest.version, "22.23.3");
    const nodeLock = JSON.parse(fs.readFileSync(path.join(tools, "package-lock.json"), "utf8"));
    const nodePackage = lockedPackage(nodeLock, "node_modules/node-linux-x64", "22.23.3");
    writeJson(path.join(output, "node-package.json"), {
      node,
      nodePackage,
      lock: identity(path.join(tools, "package-lock.json")),
    });
    const runtime = run(
      node.path,
      [
        "-p",
        "JSON.stringify({version:process.version,platform:process.platform,arch:process.arch,execPath:process.execPath})",
      ],
      repository,
      path.join(output, "consumer-node"),
      env,
    );
    successful(runtime);
    assert.deepEqual(JSON.parse(runtime.stdout.toString()), {
      version: "v22.23.3",
      platform: "linux",
      arch: "x64",
      execPath: node.path,
    });
    const childEnv = {
      ...env,
      PATH: `${path.dirname(node.path)}${path.delimiter}${env.PATH ?? ""}`,
    };
    const staging = path.join(repository, "target", `direct-child-props-11809-${process.pid}`);
    const packs = path.join(output, "packs");
    fs.mkdirSync(packs);
    const dependencies: Record<string, string> = {
      typescript: "7.1.0-beta",
      tsrs: "0.21.0-ts7.1.0-beta",
      vue: "3.5.44",
    };
    for (const directory of stagePackages(staging, image)) {
      const result = run(
        process.execPath,
        [npmCli, "pack", "--ignore-scripts", "--json", "--pack-destination", packs],
        directory,
        path.join(output, `pack-${path.basename(directory)}`),
        env,
      );
      successful(result);
      const packed = JSON.parse(result.stdout.toString()) as {
        name: string;
        filename: string;
        integrity: string;
      }[];
      assert.equal(packed.length, 1);
      const [archive] = packed;
      assert.ok(archive);
      const tarball = path.join(packs, archive.filename);
      dependencies[archive.name] = `file:${tarball}`;
      writeJson(path.join(output, `archive-${path.basename(directory)}.json`), {
        name: archive.name,
        ...archiveIdentity(tarball, archive.integrity),
        manifest: identity(path.join(directory, "package.json")),
      });
    }
    const install = path.join(output, "install");
    fs.mkdirSync(install);
    writeJson(path.join(install, "package.json"), { private: true, dependencies });
    successful(
      run(
        node.path,
        [npmCli, "install", "--ignore-scripts", "--no-audit", "--no-fund"],
        install,
        path.join(output, "install-source-packages"),
        childEnv,
      ),
    );
    const require = createRequire(path.join(install, "package.json"));
    const manifest = JSON.parse(
      fs.readFileSync(path.join(install, "node_modules/vize/package.json"), "utf8"),
    ) as { typescript: { contentMapper: { exec: string[] } } };
    assert.deepEqual(manifest.typescript.contentMapper.exec, [
      "node",
      "./bin/vize",
      "content-mapper",
    ]);
    const cli = fs.realpathSync(path.join(install, "node_modules/vize/bin/vize"));
    const installedImage = require.resolve("@vizejs/native-linux-x64-gnu");
    assert.equal(identity(installedImage).sha256, producer.binary.sha256);
    const tsRequire = createRequire(require.resolve("typescript/package.json"));
    const tsrsRequire = createRequire(require.resolve("tsrs/package.json"));
    const officialPlatform = physicalHost(
      tsRequire,
      "@typescript/typescript-linux-x64",
      "7.1.0-beta",
      "lib/tsc",
    );
    const tsrsPlatform = physicalHost(
      tsrsRequire,
      "@tsrs/linux-x64-gnu",
      "0.21.0-ts7.1.0-beta",
      "tsrs",
    );
    const hosts = {
      official: officialPlatform.executable.path,
      tsrs: tsrsPlatform.executable.path,
    };
    const upstream = JSON.parse(
      fs.readFileSync(path.join(install, "node_modules/tsrs/UPSTREAM.json"), "utf8"),
    ) as { commit: string };
    assert.equal(upstream.commit, "3e9236361fa97a9e6c685f4833ac457ae2d8c50a");
    assert.notEqual(identity(hosts.official).sha256, identity(hosts.tsrs).sha256);
    const lockPath = path.join(install, "package-lock.json");
    const lock = JSON.parse(fs.readFileSync(lockPath, "utf8"));
    const packages = Object.fromEntries(
      [
        [require.resolve("typescript/package.json"), "7.1.0-beta"],
        [require.resolve("tsrs/package.json"), "0.21.0-ts7.1.0-beta"],
        [require.resolve("vue/package.json"), "3.5.44"],
        [officialPlatform.manifest.path, "7.1.0-beta"],
        [tsrsPlatform.manifest.path, "0.21.0-ts7.1.0-beta"],
      ].map(([file, version]) => {
        assert.ok(file && version);
        const key = path.relative(install, path.dirname(file));
        const manifest = JSON.parse(fs.readFileSync(file, "utf8")) as { version: string };
        assert.equal(manifest.version, version);
        return [key, { ...lockedPackage(lock, key, version), manifest: identity(file) }];
      }),
    );
    writeJson(path.join(output, "consumer-providers.json"), {
      node,
      hosts: Object.fromEntries(roles.map((role) => [role, identity(hosts[role])])),
      image: identity(installedImage),
      cli: identity(cli),
      upstream,
      packages,
      lock: identity(lockPath),
    });
    for (const role of roles) {
      const version = run(
        hosts[role],
        ["--version"],
        install,
        path.join(output, `${role}-version`),
        childEnv,
      );
      successful(version);
      assert.match(version.stdout.toString(), /7\.1\.0-beta/u);
    }
    const metadata = spawnSync("go", ["version", "-m", hosts.official], { env: childEnv });
    writeJson(path.join(output, "official-go-metadata.json"), {
      status: metadata.status,
      error: metadata.error?.message ?? null,
      stdout: metadata.stdout?.toString("base64"),
      stderr: metadata.stderr?.toString("base64"),
    });
    if (metadata.status === 0)
      assert.ok(metadata.stdout.toString().includes(`vcs.revision=${upstream.commit}`));
    const helpers = [
      "content-mapper-helpers.d.ts",
      "content-mapper-helpers.d.mts",
      "content-mapper-helpers.d.cts",
      "generated/content-mapper-types.d.ts",
    ].map((file) => identity(path.join(install, "node_modules/vize", file)));
    writeJson(path.join(output, "installed-helpers.json"), helpers);
    const fixtureNames = [
      "PropsChild.vue.txt",
      "PropsParent.vue.txt",
      "Missing.vue.txt",
      "tsconfig.vize-no-shim.json.txt",
    ];
    const fixtureInputs = fixtureNames.map((file) => identity(path.join(fixture, file)));
    writeJson(path.join(output, "fixture-inputs.json"), { source, inputs: fixtureInputs });
    const child = fs.readFileSync(path.join(fixture, "PropsChild.vue.txt"), "utf8");
    const parent = fs.readFileSync(path.join(fixture, "PropsParent.vue.txt"), "utf8");
    const missing = fs.readFileSync(path.join(fixture, "Missing.vue.txt"), "utf8");
    for (const role of roles)
      for (const prop of props)
        for (const shim of [false, true])
          for (const state of states) {
            const id = `${role}-${prop}-${shim}-${state}`;
            const project = path.join(output, "cases", id);
            fs.mkdirSync(project, { recursive: true });
            fs.symlinkSync(
              path.join(install, "node_modules"),
              path.join(project, "node_modules"),
              "dir",
            );
            fs.writeFileSync(
              path.join(project, "PropsChild.vue"),
              prop === "count" ? child.replace("benchCount", "count") : child,
            );
            fs.writeFileSync(
              path.join(project, "PropsParent.vue"),
              parent
                .replace("bench-count", prop)
                .replace("'bench-negative'", state === "wrong" ? "'bench-negative'" : "42"),
            );
            if (state === "missing") fs.writeFileSync(path.join(project, "Missing.vue"), missing);
            const config = JSON.parse(
              fs.readFileSync(path.join(fixture, "tsconfig.vize-no-shim.json.txt"), "utf8"),
            ) as { include: string[] };
            if (state === "missing") config.include.push("Missing.vue");
            if (shim) {
              config.include.push("shim.d.ts");
              fs.writeFileSync(
                path.join(project, "shim.d.ts"),
                "declare module '*.vue' { import type { DefineComponent } from 'vue'; const component: DefineComponent<{}, {}, any>; export default component; }\n",
              );
            }
            writeJson(path.join(project, "tsconfig.json"), config);
            const inputs = [
              "PropsChild.vue",
              "PropsParent.vue",
              "tsconfig.json",
              ...(shim ? ["shim.d.ts"] : []),
              ...(state === "missing" ? ["Missing.vue"] : []),
            ].map((file) => identity(path.join(project, file)));
            writeJson(path.join(project, "inputs.json"), {
              id,
              source,
              expected: expectedOutput(state),
              files: inputs,
            });
            const argv = [
              "--runExternalCode",
              "--noEmit",
              "--pretty",
              "false",
              "-p",
              "tsconfig.json",
            ];
            const result = run(hosts[role], argv, project, path.join(project, "process"), childEnv);
            assertOutput(result.status, result.stdout, result.stderr, expectedOutput(state));
            cells.push(id);
            if (prop === "bench-count" && !shim && state === "wrong") {
              const trace = path.join(project, "observed-loads");
              const observed = run(
                hosts[role],
                argv,
                project,
                path.join(project, "observed-process"),
                {
                  ...childEnv,
                  NODE_OPTIONS: `--import=${pathToFileURL(observer).href}`,
                  VIZE_TEST_CHILD_PROPS_TRACE: trace,
                },
              );
              assertOutput(
                observed.status,
                observed.stdout,
                observed.stderr,
                expectedOutput(state),
              );
              const records = fs.readdirSync(trace).filter((file) => file.endsWith(".json"));
              assert.equal(records.length, 1, "one actual mapper native load");
              const [recordName] = records;
              assert.ok(recordName);
              const record = JSON.parse(
                fs.readFileSync(path.join(trace, recordName), "utf8"),
              ) as NativeObservation;
              assertNativeObservation(
                record,
                node,
                producer.binary.sha256,
                producer.identity.token,
                cli,
              );
              assert.equal(record.image.path, fs.realpathSync(installedImage));
            }
            assert.deepEqual(
              inputs.map((file) => identity(file.path)),
              inputs,
              "authored case inputs remain unchanged",
            );
          }
    assertInventory(cells);
    verifyHelperAbiFallbacks(install, output, hosts, childEnv);
    assert.deepEqual(
      helpers.map((file) => identity(file.path)),
      helpers,
    );
    assert.deepEqual(
      fixtureNames.map((file) => identity(path.join(fixture, file))),
      fixtureInputs,
    );
    assert.deepEqual(
      identity(process.execPath),
      JSON.parse(fs.readFileSync(path.join(output, "source.json"), "utf8")).controller,
    );
    console.log(
      "11809_SOURCE_PACKAGE_CELLS",
      JSON.stringify({
        source,
        cells: cells.length,
        observationContrasts: 2,
        producerNode: process.version,
        consumerNode: "v22.23.3",
      }),
    );
    qualified = true;
  } finally {
    writeJson(path.join(output, "cell-inventory.json"), { cells, qualified });
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();

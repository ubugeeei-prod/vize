import assert from "node:assert/strict";
import { createHash, randomBytes } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { spawnSync, execFileSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";
import { artSource, componentSource, runLifecycleWorker } from "./sfc-optional-lifecycle-worker.ts";
const sourceSha = "1525ff2f7621e8f0b3053451568bec48ff5b8855";
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const fileHash = (file) => hash(fs.readFileSync(file));
const git = (root, ...args) =>
  execFileSync("git", ["-C", root, ...args], { encoding: "utf8" }).trim();
const write = (file, value) => fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
const self = fileURLToPath(import.meta.url);
/** Preserve both process streams and defer the verdict until every diagnostic mode ran. */
function run(args, cwd, env, log) {
  const fd = fs.openSync(log, "w");
  const stderr = fs.openSync(log + ".stderr", "w");
  try {
    const child = spawnSync(args[0], args.slice(1), { cwd, env, stdio: ["ignore", fd, stderr] });
    return {
      command: args,
      cwd,
      log,
      stderrLog: log + ".stderr",
      status: child.status,
      signal: child.signal,
      error: child.error?.message,
    };
  } finally {
    fs.closeSync(fd);
    fs.closeSync(stderr);
  }
}
function success(value) {
  if (value.error || value.signal || value.status !== 0) {
    for (const log of [value.log, value.stderrLog]) {
      if (fs.existsSync(log)) console.error(fs.readFileSync(log, "utf8").slice(-32768));
    }
  }
  assert.equal(value.error, undefined);
  assert.equal(value.signal, null, JSON.stringify(value));
  assert.equal(value.status, 0, JSON.stringify(value));
}
/** Install and authenticate the immutable public control without workspace links. */
async function install(project, output) {
  const packages = [
    "@vizejs/native",
    "@vizejs/native-linux-x64-gnu",
    "@vizejs/vite-plugin",
    "@vizejs/vite-plugin-musea",
  ];
  const metadata = await Promise.all(
    packages.map(async (name) => {
      const base = "https://registry.npmjs.org/" + encodeURIComponent(name);
      const get = async (url, accept) => {
        const response = await fetch(url, {
          headers: { Accept: accept, "Cache-Control": "no-cache" },
          signal: AbortSignal.timeout(15000),
        });
        assert.equal(response.status, 200);
        return response.json();
      };
      const [exact, packument] = await Promise.all([
        get(base + "/0.566.0", "application/json"),
        get(base, "application/vnd.npm.install-v1+json"),
      ]);
      assert.equal(exact.version, "0.566.0");
      assert.match(exact.dist.integrity, /^sha512-/);
      assert.equal(packument.versions["0.566.0"].dist.integrity, exact.dist.integrity);
      assert.equal(packument.versions["0.566.0"].dist.tarball, exact.dist.tarball);
      return { name, version: exact.version, ...exact.dist };
    }),
  );
  write(path.join(output, "public-registry-metadata.json"), metadata);
  write(path.join(project, "package.json"), {
    name: "optional-lifecycle-public",
    private: true,
    type: "module",
  });
  success(
    run(
      [
        "npm",
        "install",
        "--prefix",
        project,
        "--workspaces=false",
        "--registry=https://registry.npmjs.org",
        "--prefer-online",
        "--no-audit",
        "--no-fund",
        ...packages.map((name) => name + "@0.566.0"),
        "vite@8.3.5",
        "vue@3.5.35",
        "typescript@6.0.3",
      ],
      project,
      process.env,
      path.join(output, "public-install.log"),
    ),
  );
  const lock = JSON.parse(fs.readFileSync(path.join(project, "package-lock.json"), "utf8"));
  for (const item of metadata) {
    const folder = path.join(project, "node_modules", item.name);
    assert.equal(fs.lstatSync(folder).isSymbolicLink(), false);
    assert.ok(fs.realpathSync(folder).startsWith(project + path.sep));
    assert.equal(
      JSON.parse(fs.readFileSync(path.join(folder, "package.json"), "utf8")).version,
      item.version,
    );
    const entry = lock.packages["node_modules/" + item.name];
    assert.equal(entry.version, item.version);
    assert.equal(entry.integrity, item.integrity);
    assert.equal(entry.resolved, item.tarball);
  }
  write(path.join(output, "public-package-lock.json"), lock);
}
/** Bind a worker to one exact native image and its actual JS dependency entries. */
function identity(label, side, mode, root, source, binary, token) {
  const require = createRequire(path.join(root, "package.json"));
  const esmEntry = (name) => {
    const folder = path.join(root, "node_modules", name);
    const manifest = JSON.parse(fs.readFileSync(path.join(folder, "package.json"), "utf8"));
    const target = manifest.exports["."].import;
    assert.equal(typeof target, "string", name + " must expose its actual ESM import entry");
    return path.join(folder, target);
  };
  const nativeEntry =
    side === "source"
      ? path.join(source, "npm/native/index.js")
      : require.resolve("@vizejs/native");
  if (!binary) {
    const folder = path.dirname(require.resolve("@vizejs/native-linux-x64-gnu/package.json"));
    const binaries = fs.readdirSync(folder).filter((name) => name.endsWith(".node"));
    assert.equal(binaries.length, 1);
    binary = path.join(folder, binaries[0]);
  }
  return {
    label,
    side,
    mode,
    attempt: 1,
    nativeEntry,
    binary: fs.realpathSync(binary),
    binarySha256: fileHash(binary),
    token,
    viteEntry: require.resolve("vite"),
    vueEntry: require.resolve("vue/dist/vue.runtime.esm-bundler.js"),
    typescriptEntry: require.resolve("typescript"),
    pluginEntry:
      side === "source"
        ? path.join(source, "npm/builder/vite/src/index.ts")
        : esmEntry("@vizejs/vite-plugin"),
    museaEntry:
      side === "source"
        ? path.join(source, "npm/builder/vite-musea/src/plugin/index.ts")
        : esmEntry("@vizejs/vite-plugin-musea"),
    fixtureParent: root,
  };
}
/** Replay one attempt per retained image and publish a qualified receipt only after all pass. */
async function campaign(source, output) {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  for (const key of [
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "MALLOC_CHECK_",
    "MALLOC_PERTURB_",
    "GLIBC_TUNABLES",
    "LD_PRELOAD",
  ])
    assert.equal(process.env[key], undefined, key);
  source = fs.realpathSync(source);
  output = path.resolve(output);
  fs.mkdirSync(output, { recursive: true });
  assert.equal(git(source, "rev-parse", "HEAD"), sourceSha);
  assert.equal(git(source, "diff", "--name-only", "HEAD"), "");
  const temporary = fs.mkdtempSync(
    path.join(fs.realpathSync(process.env.RUNNER_TEMP), "sfc-optional-lifecycle-"),
  );
  const publicProject = path.join(temporary, "public");
  fs.mkdirSync(publicProject);
  const libcFiles = execFileSync("dpkg-query", ["-L", "libc6"], { encoding: "utf8" })
    .split("\n")
    .filter((file) => path.basename(file) === "libc_malloc_debug.so.0");
  assert.equal(libcFiles.length, 1, "libc6 must provide one malloc debug image");
  const mallocDebug = {
    packagePath: libcFiles[0],
    realpath: fs.realpathSync(libcFiles[0]),
    sha256: fileHash(libcFiles[0]),
  };
  const receipt = {
    schemaVersion: 2,
    issue: {
      number: 10582,
      url: "https://github.com/ubugeeei-prod/vize/issues/10582",
      author: "ubugeeei",
      reported: {
        run: "https://github.com/ubugeeei-prod/vize/actions/runs/40176237408",
        job: 121117574432,
        head: "63f34b0ac2d7fb81aa9c0c989f87ed2b03c1b9a3",
        workflowSource: "4094fcf8fa07d02cc8307f6773d3ebeb37895ed6",
        artifact: {
          id: 12592975351,
          name: "musea-native-inline-4094fcf8fa07d02cc8307f6773d3ebeb37895ed6",
          size: 62118,
          digest: "sha256:526751a52a8850b3e3e0a968f54df9029e336b72930bea2c7b4975305bb6ff58",
        },
      },
      controlled: {
        run: "https://github.com/ubugeeei-prod/vize/actions/runs/40177938970",
        source: sourceSha,
        driver: "879bc5b087b28daecb7bb284aa64630e0fdbc9f3",
        artifact: {
          id: 12592534636,
          name: "sfc-optional-lifecycle-879bc5b087b28daecb7bb284aa64630e0fdbc9f3-1",
          size: 87263,
          digest: "sha256:15ee0dd32e3ba9640ca357989a5fe214591233883125bf92ef5ca6cbb3d98320",
        },
        images: {
          public: "837681f00ae64f54837138fa0c5eda28fe337d4920b87de4d1fbc1d55587d1dc",
          source: "22a7d8dfa32482726dc2721aa3fbbb0d163f7a10c75d96a0b2e95ddd630459cf",
        },
      },
    },
    source: sourceSha,
    driver: process.env.GITHUB_SHA,
    driverSha256: fileHash(self),
    workerSha256: fileHash(
      fileURLToPath(new URL("./sfc-optional-lifecycle-worker.ts", import.meta.url)),
    ),
    recordedRun: `${process.env.GITHUB_SERVER_URL}/${process.env.GITHUB_REPOSITORY}/actions/runs/${process.env.GITHUB_RUN_ID}`,
    fixtureSha256: hash(componentSource),
    artSha256: hash(artSource),
    sourceTrees: Object.fromEntries(
      ["crates", "davinci", "vendor", "npm/native"].map((name) => [
        name,
        git(source, "rev-parse", "HEAD:" + name),
      ]),
    ),
    throughputMeasured: false,
    attemptsPerImage: 1,
    imageOrder: ["public", "source", "source-allocator", "source-memcheck"],
    environment: {
      node: process.version,
      platform: process.platform,
      arch: process.arch,
      valgrind: execFileSync("valgrind", ["--version"], { encoding: "utf8" }).trim(),
      libc: execFileSync("getconf", ["GNU_LIBC_VERSION"], { encoding: "utf8" }).trim(),
      packages: execFileSync("dpkg-query", ["-W", "valgrind", "libc6"], {
        encoding: "utf8",
      }).trim(),
      mallocDebug,
    },
    builds: [],
    workers: [],
    observationsComplete: false,
    qualified: false,
  };
  assert.equal(receipt.environment.valgrind, "valgrind-3.22.0");
  const retain = () => write(path.join(output, "receipt.json"), receipt);
  retain();
  await install(publicProject, output);
  const native = path.join(source, "npm/native");
  const command = [
    "vp",
    "exec",
    "napi",
    "build",
    "--platform",
    "--profile",
    "ci",
    "--manifest-path",
    "../../crates/vize_vitrine/Cargo.toml",
    "-p",
    "vize_vitrine",
    "--features",
    "napi,legacy",
    "--output-dir",
    ".",
    "--",
    "--message-format=json-render-diagnostics",
    "--locked",
  ];
  /** Build one token-distinct image from an initially empty Cargo target. */
  const buildSource = (label, mode) => {
    const target = path.join(temporary, label + "-" + sourceSha);
    assert.equal(fs.existsSync(target), false);
    fs.mkdirSync(target);
    const token = randomBytes(16).toString("hex");
    const log = path.join(output, label + "-build.log");
    const build = {
      label,
      mode,
      target,
      targetInitiallyEmpty: fs.readdirSync(target).length === 0,
      profileDebug: mode === "memcheck" ? 1 : false,
      token,
      process: run(
        command,
        native,
        {
          ...process.env,
          ...(mode === "memcheck" ? { CARGO_PROFILE_CI_DEBUG: "1" } : {}),
          CARGO_TARGET_DIR: target,
          VIZE_NATIVE_BUILD_TOKEN: token,
        },
        log,
      ),
    };
    receipt.builds.push(build);
    retain();
    success(build.process);
    const artifacts = fs
      .readFileSync(log, "utf8")
      .split("\n")
      .flatMap((line) => {
        try {
          const packet = JSON.parse(line);
          return packet.reason === "compiler-artifact" ? [packet] : [];
        } catch {
          return [];
        }
      });
    assert.ok(artifacts.length > 0);
    const owned = artifacts.filter((artifact) =>
      artifact.target.src_path.startsWith(source + path.sep),
    );
    assert.ok(owned.some((artifact) => artifact.target.name === "vize_croquis"));
    for (const artifact of owned) {
      assert.equal(artifact.fresh, false, artifact.target.name + " reused a compiled artifact");
      assert.ok(artifact.filenames.every((filename) => filename.startsWith(target + path.sep)));
    }
    write(path.join(output, label + "-cargo-artifacts.json"), artifacts);
    success(
      run(
        [process.execPath, path.join(native, "scripts/sync-entrypoint.mjs")],
        native,
        process.env,
        path.join(output, label + "-entrypoint.log"),
      ),
    );
    build.sourceChanges = git(source, "diff", "--name-only", "HEAD").split("\n").filter(Boolean);
    assert.ok(
      build.sourceChanges.every((file) =>
        ["npm/native/index.js", "npm/native/index.d.ts"].includes(file),
      ),
      "diagnostic build changed authored source",
    );
    retain();
    const binaries = fs.readdirSync(native).filter((file) => file.endsWith(".node"));
    assert.equal(binaries.length, 1);
    return identity(
      label,
      "source",
      mode,
      path.join(source, "npm/builder/vite-musea"),
      source,
      path.join(native, binaries[0]),
      token,
    );
  };
  const retainImage = (image) => {
    const directory = path.join(output, "loaded-images");
    fs.mkdirSync(directory, { recursive: true });
    image.retainedBinary = path.join(directory, image.label + ".node");
    fs.copyFileSync(image.binary, image.retainedBinary, fs.constants.COPYFILE_EXCL);
    assert.equal(fileHash(image.retainedBinary), image.binarySha256);
  };
  /** Run exactly one worker with the diagnostic runtime proven inside that same process. */
  const runImage = (image) => {
    retainImage(image);
    const identityFile = path.join(output, image.label + "-identity.json");
    const workerOutput = path.join(output, image.label);
    fs.mkdirSync(workerOutput);
    let environment = process.env;
    let workerCommand = ["vp", "exec", "tsx", self, "--worker", identityFile, workerOutput];
    if (image.mode === "allocator") {
      image.diagnosticRuntime = mallocDebug;
      environment = {
        ...process.env,
        LD_PRELOAD: mallocDebug.realpath,
        GLIBC_TUNABLES: "glibc.malloc.tcache_count=0:glibc.malloc.perturb=165",
        MALLOC_CHECK_: "3",
        MALLOC_PERTURB_: "165",
      };
    } else if (image.mode === "memcheck") {
      const require = createRequire(path.join(source, "npm/builder/vite-musea/package.json"));
      const tsxLoader = fs.realpathSync(require.resolve("tsx/esm"));
      image.loader = { path: tsxLoader, sha256: fileHash(tsxLoader) };
      workerCommand = [
        "/usr/bin/timeout",
        "600",
        "valgrind",
        "--tool=memcheck",
        "--error-exitcode=97",
        "--undef-value-errors=no",
        "--leak-check=no",
        "--num-callers=40",
        "--trace-children=no",
        `--log-file=${path.join(workerOutput, "memcheck.log")}`,
        process.execPath,
        "--import",
        pathToFileURL(tsxLoader).href,
        self,
        "--worker",
        identityFile,
        workerOutput,
      ];
    }
    write(identityFile, image);
    receipt.workers.push({
      label: image.label,
      side: image.side,
      mode: image.mode,
      attempt: image.attempt,
      environment: Object.fromEntries(
        ["GLIBC_TUNABLES", "MALLOC_CHECK_", "MALLOC_PERTURB_", "LD_PRELOAD"]
          .filter((key) => Object.hasOwn(environment, key))
          .map((key) => [key, environment[key]]),
      ),
      ...run(
        workerCommand,
        path.join(source, "npm/builder/vite-musea"),
        environment,
        path.join(output, image.label + "-worker.log"),
      ),
    });
    retain();
  };
  const publicImage = identity("public", "public", "baseline", publicProject);
  runImage(publicImage);
  const sourceImage = buildSource("source", "baseline");
  for (const name of ["./npm/cli", "./npm/builder/vite", "./npm/builder/vite-musea"]) {
    success(
      run(
        ["vp", "run", "--filter", name, "build"],
        source,
        process.env,
        path.join(output, name.replaceAll(/[^a-z0-9]/gi, "-") + "-build.log"),
      ),
    );
  }
  runImage(sourceImage);
  const allocatorImage = buildSource("source-allocator", "allocator");
  runImage(allocatorImage);
  const memcheckImage = buildSource("source-memcheck", "memcheck");
  runImage(memcheckImage);
  const sourceImages = [sourceImage, allocatorImage, memcheckImage];
  assert.equal(new Set(sourceImages.map(({ token }) => token)).size, sourceImages.length);
  assert.equal(
    new Set(sourceImages.map(({ binarySha256 }) => binarySha256)).size,
    sourceImages.length,
  );
  receipt.observationsComplete = receipt.workers.length === receipt.imageOrder.length;
  retain();
  for (const child of receipt.workers) success(child);
  receipt.qualified = true;
  retain();
}
if (process.argv[2] === "--worker") await runLifecycleWorker(process.argv[3], process.argv[4]);
else {
  assert.equal(process.argv.length, 4);
  await campaign(process.argv[2], process.argv[3]);
}

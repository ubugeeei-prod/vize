import assert from "node:assert/strict";
import { createHash, randomBytes } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { spawnSync, execFileSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";
const sourceSha = "1525ff2f7621e8f0b3053451568bec48ff5b8855";
const members =
  'spaced? : string;\ntabbed?\t: number;\ncommented? /* question */ : boolean;\ncompact?: string;\nrequired : number;\nreadonly read? : string;\n"scope.name"? : string;\ncafé? : number;\n"why?:"? /* trivia */ : boolean;';
const componentSource =
  '<script setup lang="ts">const props = defineProps<{ ' +
  members +
  " }>();</script><template><div>{{ props.required }}</div></template>";
const artSource =
  '<script setup lang="ts">defineArt("./SpacingProbe.vue", { title: "Optional Spacing" })</script><art><variant name="Default"><SpacingProbe :required="1" /></variant></art>';
const names = [
  "spaced",
  "tabbed",
  "commented",
  "compact",
  "required",
  "read",
  "scope.name",
  "café",
  "why?:",
];
const types = [
  "string",
  "number",
  "boolean",
  "string",
  "number",
  "string",
  "string",
  "number",
  "boolean",
];
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const fileHash = (file) => hash(fs.readFileSync(file));
const git = (root, ...args) =>
  execFileSync("git", ["-C", root, ...args], { encoding: "utf8" }).trim();
const write = (file, value) => fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
const self = fileURLToPath(import.meta.url);
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
async function worker(identityFile, output) {
  const identity = JSON.parse(fs.readFileSync(identityFile, "utf8"));
  const trace = (phase, extra = {}) => {
    fs.appendFileSync(
      path.join(output, "phases.jsonl"),
      JSON.stringify({ phase, ...extra }) + "\n",
    );
    console.log(JSON.stringify({ phase, ...extra }));
  };
  process.on("beforeExit", (code) => trace("process-before-exit", { code }));
  process.on("exit", (code) => trace("process-exit", { code }));
  const root = fs.mkdtempSync(path.join(identity.fixtureParent, "lifecycle-"));
  const component = path.join(root, "src/SpacingProbe.vue"),
    art = path.join(root, "src/SpacingProbe.art.vue");
  fs.mkdirSync(path.dirname(component), { recursive: true });
  fs.writeFileSync(component, componentSource);
  fs.writeFileSync(art, artSource);
  fs.writeFileSync(path.join(root, "index.html"), "<!doctype html><html><body></body></html>");
  const { default: ts } = await import(pathToFileURL(identity.typescriptEntry).href);
  assert.equal(ts.version, "6.0.3");
  const truth = ts.createSourceFile(
    "truth.ts",
    "type Props = { " + members + " }",
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  assert.deepEqual(truth.parseDiagnostics, []);
  const alias = truth.statements[0];
  assert.ok(ts.isTypeAliasDeclaration(alias) && ts.isTypeLiteralNode(alias.type));
  assert.deepEqual(
    alias.type.members.map((member) => member.questionToken !== undefined),
    names.map((name) => name !== "required"),
  );
  const require = createRequire(identity.nativeEntry);
  const binding = require(identity.nativeEntry);
  const images = () =>
    Object.keys(require.cache).filter(
      (file) => file.endsWith(".node") && path.basename(file).startsWith("vize-vitrine"),
    );
  assert.deepEqual(images(), [identity.binary]);
  assert.equal(fileHash(identity.binary), identity.binarySha256);
  const token = binding.getNativeBuildToken();
  assert.equal(typeof token, "string");
  if (identity.side === "source") assert.equal(token, identity.token);
  const packets = [];
  const retain = () =>
    write(path.join(output, "packets.json"), {
      identity,
      loadedToken: token,
      root,
      fixtureSha256: hash(componentSource),
      artSha256: hash(artSource),
      throughputMeasured: false,
      packets,
    });
  retain();
  trace("native-loaded", { binary: identity.binary, sha256: identity.binarySha256, token });
  const { createServer } = await import(pathToFileURL(identity.viteEntry).href);
  const { default: vize } = await import(pathToFileURL(identity.pluginEntry).href);
  const { musea } = await import(pathToFileURL(identity.museaEntry).href);
  trace("create-server-begin");
  const server = await createServer({
    root,
    configFile: false,
    resolve: { alias: { vue: identity.vueEntry } },
    plugins: [vize(), musea({ include: ["src/**/*.art.vue"] })],
    server: { host: "127.0.0.1", port: 0 },
  });
  let failure;
  try {
    trace("listen-begin");
    await server.listen();
    trace("listen-end");
    const address = server.httpServer.address();
    assert.ok(address && typeof address !== "string");
    const endpoint =
      "http://127.0.0.1:" + address.port + "/__musea__/api/arts/" + encodeURIComponent(art);
    for (const newline of ["\n", "\r\n"]) {
      const source = componentSource.replaceAll("\n", newline);
      trace("write-begin", { newline });
      await fs.promises.writeFile(component, source);
      trace("write-end");
      trace("native-begin");
      const native = binding.analyzeSfc(source, {
        filename: component,
        cache: false,
        profile: false,
      });
      const expected = {
        emits: [],
        props: names.map((name, i) => ({
          name,
          type: types[i],
          required: name === "required" || (identity.side === "public" && i < 3),
        })),
      };
      assert.deepEqual(
        native,
        expected,
        "published optional flags differ only for the three proven cases",
      );
      trace("native-end");
      const packet = { newline, source, native };
      packets.push(packet);
      retain();
      trace("http-analysis-begin");
      const analysis = await fetch(endpoint + "/analysis");
      assert.equal(analysis.status, 200);
      packet.analysis = await analysis.json();
      assert.deepEqual(packet.analysis, native);
      retain();
      trace("http-analysis-end");
      trace("http-palette-begin");
      const palette = await fetch(endpoint + "/palette");
      assert.equal(palette.status, 200);
      packet.palette = await palette.json();
      assert.deepEqual(
        packet.palette.controls.map(({ name, required }) => ({ name, required })),
        expected.props.map(({ name, required }) => ({ name, required })),
      );
      assert.deepEqual(JSON.parse(packet.palette.json).controls, packet.palette.controls);
      const exported = ts.createSourceFile(
        "palette.ts",
        packet.palette.typescript,
        ts.ScriptTarget.Latest,
        true,
        ts.ScriptKind.TS,
      );
      assert.deepEqual(exported.parseDiagnostics, []);
      const declaration = exported.statements[0];
      assert.ok(ts.isInterfaceDeclaration(declaration));
      assert.deepEqual(
        declaration.members.map((member) => ({
          name: ts.isStringLiteral(member.name) ? member.name.text : member.name.getText(exported),
          required: member.questionToken === undefined,
        })),
        expected.props.map(({ name, required }) => ({ name, required })),
      );
      retain();
      trace("http-palette-end");
    }
  } catch (error) {
    failure = error;
    trace("functional-error", { message: error.message, stack: error.stack });
  } finally {
    retain();
    trace("close-begin");
    await server.close();
    trace("close-end");
  }
  assert.deepEqual(images(), [identity.binary]);
  assert.equal(fileHash(identity.binary), identity.binarySha256);
  trace("worker-complete");
  if (failure) throw failure;
}
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
function identity(side, root, source, binary, token) {
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
    side,
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
async function campaign(source, output) {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  for (const key of [
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
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
  const target = path.join(temporary, "source-" + sourceSha);
  assert.equal(fs.existsSync(target), false);
  fs.mkdirSync(target);
  const publicProject = path.join(temporary, "public");
  fs.mkdirSync(publicProject);
  const receipt = {
    source: sourceSha,
    driver: process.env.GITHUB_SHA,
    driverSha256: fileHash(self),
    target,
    targetInitiallyEmpty: fs.readdirSync(target).length === 0,
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
    workers: [],
  };
  const retain = () => write(path.join(output, "receipt.json"), receipt);
  retain();
  await install(publicProject, output);
  const native = path.join(source, "npm/native"),
    token = randomBytes(16).toString("hex");
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
  receipt.build = run(
    command,
    native,
    { ...process.env, CARGO_TARGET_DIR: target, VIZE_NATIVE_BUILD_TOKEN: token },
    path.join(output, "source-build.log"),
  );
  retain();
  success(receipt.build);
  const artifacts = fs
    .readFileSync(path.join(output, "source-build.log"), "utf8")
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
  write(path.join(output, "source-cargo-artifacts.json"), artifacts);
  success(
    run(
      [process.execPath, path.join(native, "scripts/sync-entrypoint.mjs")],
      native,
      process.env,
      path.join(output, "source-entrypoint.log"),
    ),
  );
  const binaries = fs.readdirSync(native).filter((file) => file.endsWith(".node"));
  assert.equal(binaries.length, 1);
  const identities = [
    identity("public", publicProject),
    identity(
      "source",
      path.join(source, "npm/builder/vite-musea"),
      source,
      path.join(native, binaries[0]),
      token,
    ),
  ];
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
  for (const image of identities) {
    const identityFile = path.join(output, image.side + "-identity.json");
    write(identityFile, image);
    const workerOutput = path.join(output, image.side);
    fs.mkdirSync(workerOutput);
    receipt.workers.push({
      side: image.side,
      ...run(
        ["vp", "exec", "tsx", self, "--worker", identityFile, workerOutput],
        path.join(source, "npm/builder/vite-musea"),
        process.env,
        path.join(output, image.side + "-worker.log"),
      ),
    });
    retain();
  }
  for (const child of receipt.workers) success(child);
  receipt.complete = true;
  retain();
}
if (process.argv[2] === "--worker") await worker(process.argv[3], process.argv[4]);
else {
  assert.equal(process.argv.length, 4);
  await campaign(process.argv[2], process.argv[3]);
}

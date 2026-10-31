/** Current-source observations over the immutable original27; never a historical/public admission. */
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import {
  copyFileSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { asArray, asRecord, asString, parseJson, readJsonRecord } from "../../json-value.ts";
import { Recorder, sha256 } from "../../config/windows_consumer/common.ts";
import {
  assertCustody,
  assertDeclarationCustody,
  captureDeclarationGraph,
  fileImage,
  inside,
} from "../../release/computed_event_custody.ts";
import { capture, completed } from "../../release/constructor_original27_public/capture.ts";
import { sdkFiles } from "../../release/constructor_original27_public/authority.ts";
import {
  assertInputs,
  fixtureDriver,
  loadOriginal27,
  type Inputs,
} from "../../release/constructor_original27_public/fixtures.ts";
import {
  cliPacket,
  diagnostics,
  reference,
} from "../../release/constructor_original27_public/oracle.ts";
import {
  assertVueProviderLineage,
  captureVueNamespace,
} from "../../release/constructor_original27_public/host-versions.ts";
import {
  slotRequests,
  slotResponses,
} from "../../../benchmarks/scripts/canon-slot-constructor-protocol.ts";

const controllerRoot = fileURLToPath(new URL("../../../../", import.meta.url));
const git = (root: string, ...args: string[]) =>
  execFileSync("git", args, { cwd: root }).toString("utf8").trim();
const graphFiles = [...sdkFiles, "virtual-ts-slots.d.ts"].sort();
export function assertCurrentGraph(
  graph: ReturnType<typeof captureDeclarationGraph>,
  root: string,
) {
  const files = graph.files.map((file) => relative(root, file.path).replaceAll("\\", "/"));
  assert.equal(new Set(files).size, files.length);
  assert.deepEqual([...files].sort(), graphFiles);
  assert.equal(graph.edges.length, 13);
  for (const edge of graph.edges) {
    assert.ok(inside(root, edge.from) && inside(root, edge.target));
    assert.ok(graph.files.some((file) => file.path === edge.from));
    assert.ok(graph.files.some((file) => file.path === edge.target));
  }
  assert.equal(new Set(graph.edges.map((edge) => JSON.stringify(edge))).size, graph.edges.length);
}
export function assertSameGraph(
  source: ReturnType<typeof captureDeclarationGraph>,
  sourceRoot: string,
  staged: typeof source,
  stagedRoot: string,
) {
  const relativeGraph = (value: typeof source, root: string) => ({
    files: value.files.map((file) => ({ ...file, path: relative(root, file.path) })),
    edges: value.edges.map((edge) => ({
      ...edge,
      from: relative(root, edge.from),
      target: relative(root, edge.target),
    })),
  });
  assert.deepEqual(relativeGraph(staged, stagedRoot), relativeGraph(source, sourceRoot));
}
export function assertOriginalPopulation(input: Pick<Inputs, "rows" | "files" | "manifest">) {
  const ids = input.rows.map((row) => row.id),
    expected = asArray(reference().caseIds).map((id) => asString(id));
  assert.deepEqual(ids, expected);
  assert.equal(new Set(ids).size, 27);
  assert.deepEqual(
    input.rows.filter((row) => row.requiresAuthoredError).map((row) => row.id),
    reference().authoredCases,
  );
  const paths = ids.flatMap((id) => [id + "/component.ts", id + "/src/Tiny.vue"]);
  paths.push("authored_h.ts", "tsconfig.json", "tsconfig.mapper.json", "tsconfig.h.json");
  assert.deepEqual([...input.files.keys()], paths);
  assert.equal(input.files.size, 58);
  assert.deepEqual(
    input.manifest,
    [...input.files].map(([file, body]) => ({
      file,
      bytes: Buffer.byteLength(body),
      sha256: sha256(body),
    })),
  );
}
export function assertIndependentOracles(stock: unknown, official: unknown) {
  assert.deepEqual(stock, reference().stockH, "complete unchanged stock h() oracle");
  assert.deepEqual(
    official,
    reference().official,
    "complete unchanged fourteen-diagnostic Vue oracle",
  );
  assert.equal(asArray(asRecord(official).diagnostics).length, 14);
}
function writeProject(project: string, files: Map<string, string>) {
  mkdirSync(project);
  for (const [file, body] of files) {
    mkdirSync(dirname(join(project, file)), { recursive: true });
    writeFileSync(join(project, file), body, { flag: "wx" });
  }
}
function build(sourceRoot: string, source: string, record: Recorder) {
  const rows = readFileSync(join(record.root, "head-cargo.jsonl"), "utf8")
    .split("\n")
    .filter(Boolean)
    .map((line) => asRecord(parseJson(line)));
  assert.deepEqual(rows.at(-1), { reason: "build-finished", success: true });
  const images = rows.filter(
    (row) =>
      row.reason === "compiler-artifact" &&
      asRecord(row.target).name === "vize" &&
      asArray(asRecord(row.target).kind).includes("bin") &&
      row.executable,
  );
  assert.equal(images.length, 1);
  assert.equal(images[0].fresh, false);
  const cli = join(record.root, "source-vize");
  copyFileSync(asString(images[0].executable), cli);
  const result = {
    source,
    tree: git(sourceRoot, "rev-parse", "HEAD^{tree}"),
    sourceRoot,
    cli: fileImage(cli),
    cargoArtifact: images[0],
    profile: "ci-opt",
    argv: [
      "cargo",
      "build",
      "--locked",
      "--profile",
      "ci-opt",
      "-p",
      "vize",
      "--bin",
      "vize",
      "--message-format=json",
    ],
    controller: {
      source: git(controllerRoot, "rev-parse", "HEAD"),
      tree: git(controllerRoot, "rev-parse", "HEAD^{tree}"),
      script: fileImage(fileURLToPath(import.meta.url)),
    },
    tools: {
      node: process.version,
      rustc: execFileSync("rustc", ["-Vv"]).toString("utf8"),
      cargo: execFileSync("cargo", ["-V"]).toString("utf8"),
    },
    environment: Object.fromEntries(
      [
        "CARGO_TARGET_DIR",
        "CARGO_INCREMENTAL",
        "CARGO_BUILD_JOBS",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "GITHUB_SHA",
        "GITHUB_RUN_ID",
        "GITHUB_RUN_ATTEMPT",
      ].map((key) => [key, process.env[key] ?? null]),
    ),
    productionOverlay: false,
    timingUsable: false,
  };
  record.json("build.json", result);
}
async function provider(
  original: Awaited<ReturnType<typeof loadOriginal27>>,
  directory: string,
  record: Recorder,
) {
  const modules = realpathSync(join(directory, "node_modules")),
    images = [
      fileImage(join(directory, "package.json")),
      fileImage(join(directory, "package-lock.json")),
    ];
  assert.deepEqual(readJsonRecord(join(directory, "package.json")), original.provider);
  // Frozen whole manifest hashes below bind the actual provider names and versions.
  const versions = Object.fromEntries(
    ["vue", "typescript", "typescript-native", "vue-tsc"].map((name) => [
      name,
      readJsonRecord(join(modules, name, "package.json")).version,
    ]),
  );
  images.push(...captureVueNamespace(modules, record));
  assertVueProviderLineage(
    original.provider.overrides,
    readJsonRecord(join(modules, "vue-tsc/package.json")),
  );
  const resolver = join(modules, "typescript-native/lib/getExePath.js"),
    native = asRecord(await import(pathToFileURL(resolver).href));
  assert.equal(typeof native.default, "function");
  const checker = realpathSync(
    asString(Reflect.apply(native.default as () => unknown, undefined, [])),
  );
  assert.ok(inside(modules, checker));
  const stock = join(modules, "typescript/lib/tsc.js"),
    official = join(modules, "vue-tsc/bin/vue-tsc.js");
  assert.equal(
    realpathSync(createRequire(official).resolve("typescript/package.json")),
    realpathSync(join(modules, "typescript/package.json")),
  );
  for (const [file, expected] of [
    [checker, original.inputs.historical.checker],
    [stock, original.inputs.historical.stock],
    [official, original.inputs.historical.vueTsc],
  ] as const) {
    const image = fileImage(file);
    assert.equal(image.bytes, expected.bytes);
    assert.equal(image.sha256, expected.sha256);
    images.push(image);
  }
  for (const expected of original.inputs.historical.identities) {
    if (!/^node_modules\/(?:vue|typescript|typescript-native|vue-tsc)\//u.test(expected.file))
      continue;
    const image = fileImage(join(directory, expected.file));
    assert.equal(image.bytes, expected.bytes);
    assert.equal(image.sha256, expected.sha256);
    images.push(image);
  }
  images.push(fileImage(resolver), fileImage(process.execPath));
  assertCustody(images);
  record.json("providers.json", {
    versions,
    modules,
    images,
    lockRawBase64: readFileSync(join(directory, "package-lock.json")).toString("base64"),
    publishedVizeExecuted: false,
  });
  return { modules, checker, stock, official, images };
}
function stageSourcePackage(
  sourceRoot: string,
  source: string,
  cli: string,
  project: string,
  modules: string,
  record: Recorder,
) {
  const stagedModules = join(project, "node_modules");
  mkdirSync(stagedModules);
  for (const name of readdirSync(modules))
    if (name !== "vize" && name !== ".bin")
      symlinkSync(join(modules, name), join(stagedModules, name), "dir");
  const sourcePackage = realpathSync(join(sourceRoot, "npm/cli")),
    graph = captureDeclarationGraph(sourcePackage);
  assertCurrentGraph(graph, sourcePackage);
  const staged = join(stagedModules, "vize");
  mkdirSync(staged);
  const raw = readFileSync(join(sourcePackage, "package.json")),
    original = asRecord(parseJson(raw.toString("utf8")));
  const manifest = structuredClone(original);
  manifest.version = "0.0.0-source-" + source;
  asRecord(asRecord(manifest.typescript).contentMapper).exec = [cli, "content-mapper"];
  writeFileSync(join(staged, "package.json"), JSON.stringify(manifest, null, 2) + "\n", {
    flag: "wx",
  });
  writeFileSync(join(record.root, "source-package-original.json"), raw);
  record.json("source-package-staged.json", manifest);
  for (const image of graph.files) {
    const file = relative(sourcePackage, image.path),
      output = join(staged, file),
      retained = join(record.root, "source-package", file);
    mkdirSync(dirname(output), { recursive: true });
    copyFileSync(image.path, output);
    mkdirSync(dirname(retained), { recursive: true });
    copyFileSync(image.path, retained);
  }
  const stagedGraph = captureDeclarationGraph(staged);
  assertCurrentGraph(stagedGraph, staged);
  assertSameGraph(graph, sourcePackage, stagedGraph, staged);
  record.json("source-declaration-graphs.json", {
    original: graph,
    staged: stagedGraph,
    productionOverlay: false,
    sourceSdkStage: true,
    publicAcceptance: false,
  });
  return {
    sourcePackage,
    graph,
    staged,
    stagedGraph,
    sourceManifestImage: fileImage(join(sourcePackage, "package.json")),
    manifestImage: fileImage(join(staged, "package.json")),
  };
}
async function observe(
  sourceRoot: string,
  source: string,
  driverRoot: string,
  providerRoot: string,
  record: Recorder,
) {
  const original = await loadOriginal27(driverRoot);
  assert.equal(git(driverRoot, "rev-parse", "HEAD"), fixtureDriver);
  assertOriginalPopulation(original.inputs);
  const build = readJsonRecord(join(record.root, "build.json"));
  assert.equal(build.source, source);
  assert.equal(build.tree, git(sourceRoot, "rev-parse", "HEAD^{tree}"));
  const cli = asRecord(build.cli);
  assert.deepEqual(fileImage(asString(cli.path)), cli);
  const host = await provider(original, providerRoot, record),
    project = join(record.root, "project");
  writeProject(project, original.inputs.files);
  const staged = stageSourcePackage(
    sourceRoot,
    source,
    asString(cli.path),
    project,
    host.modules,
    record,
  );
  const retainedInputs = join(record.root, "inputs");
  writeProject(retainedInputs, original.inputs.files);
  record.json("original-manifest.json", original.inputs.manifest);
  record.json("fixture-custody.json", { source: fixtureDriver, custody: original.custody });
  const stock = capture(
    record,
    "stock-h",
    [process.execPath, host.stock, "--pretty", "false", "--project", "tsconfig.h.json"],
    project,
  );
  const official = capture(
    record,
    "official-vue-tsc",
    [process.execPath, host.official, "--pretty", "false", "--project", "tsconfig.json"],
    project,
  );
  const oracles = {
    stockH: { status: stock.status, diagnostics: diagnostics(stock.stdout, project) },
    official: { status: official.status, diagnostics: diagnostics(official.stdout, project) },
  };
  record.json("independent-oracles.json", oracles);
  completed(stock, 2);
  completed(official, 2);
  assertIndependentOracles(oracles.stockH, oracles.official);
  const mapper = capture(
    record,
    "current-mapper",
    [
      host.checker,
      "--runExternalCode",
      "--checkers",
      "1",
      "--noEmit",
      "--pretty",
      "false",
      "--project",
      "tsconfig.mapper.json",
    ],
    project,
  );
  const cliProcess = capture(
    record,
    "current-cli",
    [
      asString(cli.path),
      "check",
      "--no-config",
      "--format",
      "json",
      "--show-virtual-ts",
      "--tsconfig",
      "tsconfig.json",
      "--corsa-path",
      host.checker,
    ],
    project,
  );
  const mapped = diagnostics(mapper.stdout, project),
    displayed = cliPacket(
      cliProcess.stdout,
      cliProcess.status,
      project,
      original.inputs.rows.map((row) => row.id),
    );
  record.json("current-complete.json", {
    mapper: { status: mapper.status, diagnostics: mapped },
    cli: { status: cliProcess.status, packet: displayed },
  });
  completed(mapper, mapped.length ? 2 : 0);
  completed(cliProcess, displayed.result.errorCount ? 1 : 0, false);
  const protocols = [];
  for (const row of original.inputs.rows) {
    const label = "protocol-" + row.id,
      config = parseJson(original.inputs.files.get("tsconfig.mapper.json")!);
    const request = slotRequests(project, row.source, config, row.id + "/src/Tiny.vue");
    const child = capture(
      record,
      label,
      [asString(cli.path), "content-mapper"],
      project,
      undefined,
      request,
    );
    completed(child, 0);
    const packet = slotResponses(child.stdout);
    record.json(label + ".responses.json", packet.responses);
    protocols.push({
      id: row.id,
      raw: child.raw,
      status: child.status,
      responses: packet.responses,
    });
  }
  assert.deepEqual(assertInputs(project, original.inputs.files), original.inputs.manifest);
  assertCustody([
    ...host.images,
    staged.sourceManifestImage,
    staged.manifestImage,
    ...original.custody,
  ]);
  assert.deepEqual(fileImage(asString(cli.path)), cli);
  assertDeclarationCustody(staged.sourcePackage, staged.graph.files, staged.graph.edges);
  assertDeclarationCustody(staged.staged, staged.stagedGraph.files, staged.stagedGraph.edges);
  const history = reference();
  const byCase = original.inputs.rows.map(({ id, requiresAuthoredError }) => ({
    id,
    requiresAuthoredError,
    official: oracles.official.diagnostics.filter((row) => row.file.startsWith(id + "/")),
    current: {
      mapper: mapped.filter((row) => row.file.startsWith(id + "/")),
      cli: displayed.diagnostics.filter((row) => row.file.startsWith(id + "/")),
    },
    historicalB759: {
      mapper: asArray(history.mapper).filter((row) =>
        asString(asRecord(row).file).startsWith(id + "/"),
      ),
      cli: asArray(history.cli).filter((row) => asString(asRecord(row).file).startsWith(id + "/")),
    },
  }));
  record.json("summary.json", {
    schema: "vize.original27.currentSourceCapture.v1",
    source,
    tree: build.tree,
    build,
    cases: 27,
    inputFiles: 58,
    sourceDeclarations: staged.graph.files.length,
    sourceDeclarationEdges: staged.graph.edges.length,
    independent: oracles,
    current: { mapper: mapped, cli: displayed.diagnostics },
    protocols,
    directProtocolTransactions: protocols.length,
    byCase,
    captureComplete: true,
    repairQualified: false,
    publicAcceptance: false,
    timingUsable: false,
    scope:
      "One current source observation over frozen a8 original27. Generated coordinates and every full diagnostic are observations; no old nine-declaration/public packet equivalence is asserted.",
  });
}
async function main() {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(process.env.GITHUB_EVENT_NAME, "workflow_dispatch");
  assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
  assert.equal(process.version, "v24.14.0");
  const [mode, sourceRootArg, source, driverRoot, providerRoot, output] = process.argv.slice(2);
  assert.equal(process.argv.length, 8);
  assert.ok(mode === "build" || mode === "capture");
  assert.match(source, /^[a-f0-9]{40}$/u);
  const sourceRoot = realpathSync(sourceRootArg);
  assert.equal(git(sourceRoot, "rev-parse", "HEAD"), source);
  assert.equal(git(sourceRoot, "status", "--porcelain", "--untracked-files=no"), "");
  for (const name of [
    "VIZE_TEST_DISABLE_TSGO",
    "NODE_OPTIONS",
    "NODE_PATH",
    "CORSA_PATH",
    "CORSA_EXECUTABLE",
    "TSGO_PATH",
    "TSGO_EXECUTABLE",
  ])
    assert.ok(!process.env[name], "runtime override forbidden: " + name);
  const record = new Recorder(resolve(output));
  if (mode === "build") build(sourceRoot, source, record);
  else
    await observe(sourceRoot, source, realpathSync(driverRoot), realpathSync(providerRoot), record);
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) await main();

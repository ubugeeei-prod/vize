import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import {
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import { availableParallelism } from "node:os";
import { basename, dirname, join, relative, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import {
  assertCustody,
  assertDeclarationCustody,
  captureDeclarationGraph,
  fileImage,
} from "../../release/computed_event_custody.ts";
import {
  fixture,
  hostConfig,
  mapperExitReceipt,
  object,
  qualifyWire,
  semanticPacket,
} from "./admission-hosts-core.ts";
import { run, save, teeCommand } from "./admission-hosts-process.ts";
const json = (path: string) => object(JSON.parse(readFileSync(path, "utf8")));
const link = (target: string, path: string) => {
  mkdirSync(dirname(path), { recursive: true });
  symlinkSync(realpathSync(target), path, "dir");
};
function copyPackage(source: string, destination: string) {
  mkdirSync(destination, { recursive: true });
  const manifest = json(join(source, "package.json"));
  assert.ok(Array.isArray(manifest.files));
  cpSync(join(source, "package.json"), join(destination, "package.json"));
  for (const file of manifest.files) {
    assert.equal(typeof file, "string");
    assert.ok(!String(file).includes("..") && !String(file).includes("*"));
    cpSync(join(source, String(file)), join(destination, String(file)), { recursive: true });
  }
}
function graphShape(root: string) {
  const graph = captureDeclarationGraph(root);
  return {
    files: graph.files.map((row) => ({ ...row, path: relative(root, row.path) })),
    edges: graph.edges.map((row) => ({
      ...row,
      from: relative(root, row.from),
      target: relative(root, row.target),
    })),
  };
}
function host(provider: string, name: "typescript" | "tsrs", version: string) {
  const root = realpathSync(join(provider, "node_modules", name));
  assert.equal(json(join(root, "package.json")).version, version);
  const require = createRequire(join(root, "package.json"));
  const platform =
    name === "typescript" ? "@typescript/typescript-linux-x64" : "@tsrs/linux-x64-gnu";
  const nativeRoot = dirname(require.resolve(platform + "/package.json"));
  assert.equal(json(join(nativeRoot, "package.json")).version, version);
  const binaries = readdirSync(nativeRoot, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile() && ["tsc", "tsgo", "tsrs"].includes(entry.name))
    .map((entry) => join(entry.parentPath, entry.name))
    .filter((path) =>
      readFileSync(path)
        .subarray(0, 4)
        .equals(Buffer.from([127, 69, 76, 70])),
    );
  assert.equal(binaries.length, 1, "exactly one admitted native ELF required");
  const binary = fileImage(binaries[0]);
  const bytes = readFileSync(binary.path);
  assert.equal(bytes[4], 2);
  assert.equal(bytes[5], 1);
  assert.equal(bytes.readUInt16LE(18), 62);
  const command = join(root, "bin", name === "typescript" ? "tsc" : "tsrs");
  return { name, version, root, binary, launcher: fileImage(command), command };
}
export async function main() {
  assert.equal(process.platform, "linux");
  assert.equal(process.version, "v26.12.1");
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(availableParallelism(), 1, "taskset must expose exactly one CPU");
  const { values } = parseArgs({
    options: {
      source: { type: "string" },
      provider: { type: "string" },
      output: { type: "string" },
      "source-sha": { type: "string" },
      token: { type: "string" },
    },
  });
  for (const name of ["source", "provider", "output", "source-sha", "token"] as const)
    assert.ok(values[name]);
  const source = resolve(values.source!),
    provider = resolve(values.provider!),
    output = resolve(values.output!);
  assert.match(values["source-sha"]!, /^[a-f0-9]{40}$/u);
  assert.match(values.token!, /^[a-f0-9]{32}$/u);
  assert.equal(
    execFileSync("git", ["rev-parse", "HEAD"], { cwd: source, encoding: "utf8" }).trim(),
    values["source-sha"],
  );
  assert.equal(
    execFileSync("git", ["status", "--porcelain"], { cwd: source, encoding: "utf8" }),
    "",
  );
  mkdirSync(output);
  const pipeCapacity = Number(readFileSync("/proc/sys/fs/pipe-max-size", "utf8").trim());
  assert.ok(
    Number.isSafeInteger(pipeCapacity) && pipeCapacity > 0 && pipeCapacity <= 1_048_576,
    "unsupported pipe upper bound",
  );
  const sourcePackage = join(source, "npm/cli"),
    sourceNative = join(source, "npm/native");
  const nativeImage = fileImage(join(sourceNative, "vize-vitrine.linux-x64-gnu.node"));
  const declarations = captureDeclarationGraph(sourcePackage),
    originalGraph = graphShape(sourcePackage);
  for (const file of declarations.files) {
    const target = join(output, "source-sdk", relative(sourcePackage, file.path));
    mkdirSync(dirname(target), { recursive: true });
    cpSync(file.path, target);
  }
  cpSync(join(sourcePackage, "package.json"), join(output, "source-sdk/package.json"));
  cpSync(nativeImage.path, join(output, "source-native.node"));
  const fixed = [
    nativeImage,
    fileImage(join(provider, "package.json")),
    fileImage(join(provider, "package-lock.json")),
    fileImage(join(provider, "node_modules/.package-lock.json")),
    fileImage(join(provider, "provenance.json")),
    fileImage(process.execPath),
  ];
  const providerAuthority = json(join(provider, "provenance.json"));
  assert.equal(fileImage(join(provider, "package.json")).sha256, providerAuthority.packageSha256);
  assert.equal(fileImage(join(provider, "package-lock.json")).sha256, providerAuthority.lockSha256);
  const hosts = [
    host(provider, "typescript", "7.1.0-beta"),
    host(provider, "tsrs", "0.21.0-ts7.1.0-beta"),
  ];
  for (const entry of hosts) fixed.push(entry.binary, entry.launcher);
  const vue = realpathSync(join(provider, "node_modules/vue"));
  fixed.push(fileImage(join(vue, "package.json")));
  save(join(output, "provenance.json"), {
    source: values["source-sha"],
    tree: execFileSync("git", ["rev-parse", "HEAD^{tree}"], {
      cwd: source,
      encoding: "utf8",
    }).trim(),
    node: process.version,
    affinity: readFileSync("/proc/self/status", "utf8")
      .split("\n")
      .find((line) => line.startsWith("Cpus_allowed_list:")),
    availableParallelism: availableParallelism(),
    nativeImage,
    declarations,
    hosts,
    fixed,
    providerAuthority: json(join(provider, "provenance.json")),
    pipeCapacity,
    mode: "functional observer only; no performance measurements",
  });
  const packets = [];
  for (const entry of hosts) {
    const version = await run(
      join(output, entry.name + "-version"),
      [process.execPath, entry.command, "--version"],
      provider,
    );
    assert.equal(version.status, 0);
    assert.equal(version.errors, "");
    assert.match(version.text, /7\.1\.0-beta/u);
    for (const captureMode of [false, true]) {
      const label = entry.name + (captureMode ? "-capture" : "-normal"),
        directory = join(output, label);
      mkdirSync(directory);
      const project = join(directory, "project"),
        packageRoot = join(project, "node_modules/vize"),
        nativeRoot = join(project, "node_modules/@vizejs/native"),
        capture = join(directory, "protocol");
      mkdirSync(join(project, "src"), { recursive: true });
      mkdirSync(capture);
      copyPackage(sourcePackage, packageRoot);
      copyPackage(sourceNative, nativeRoot);
      cpSync(nativeImage.path, join(nativeRoot, basename(nativeImage.path)));
      const wrapperImages = [packageRoot, nativeRoot].flatMap((root) =>
        readdirSync(root, { recursive: true, withFileTypes: true })
          .filter((entry) => entry.isFile())
          .map((entry) => join(entry.parentPath, entry.name))
          .filter((path) => path !== join(packageRoot, "package.json"))
          .map(fileImage),
      );
      link(nativeRoot, join(packageRoot, "node_modules/@vizejs/native"));
      link(vue, join(project, "node_modules/vue"));
      link(vue, join(packageRoot, "node_modules/vue"));
      const sourceRequire = createRequire(join(sourcePackage, "package.json"));
      link(
        dirname(sourceRequire.resolve("oxc-transform/package.json")),
        join(packageRoot, "node_modules/oxc-transform"),
      );
      link(entry.root, join(project, "node_modules", entry.name));
      assert.equal(
        realpathSync(join(project, "node_modules/vue/package.json")),
        realpathSync(join(packageRoot, "node_modules/vue/package.json")),
      );
      assert.deepEqual(
        graphShape(packageRoot),
        originalGraph,
        "staged SDK closure differs from source",
      );
      const sdk = captureDeclarationGraph(packageRoot),
        manifest = json(join(packageRoot, "package.json"));
      const typescript = object(manifest.typescript),
        mapper = object(typescript.contentMapper);
      assert.deepEqual(mapper.exec, ["node", "./bin/vize", "content-mapper"]);
      if (captureMode) {
        mapper.exec = teeCommand(packageRoot, capture, values.token!);
        writeFileSync(join(packageRoot, "package.json"), JSON.stringify(manifest, null, 2) + "\n");
      }
      save(join(directory, "mapper-observer.json"), {
        normalEndpoint: [process.execPath, join(packageRoot, "bin/vize"), "content-mapper"],
        observer: mapper.exec,
        sdk,
        vue: {
          project: fileImage(join(project, "node_modules/vue/package.json")),
          helper: fileImage(join(packageRoot, "node_modules/vue/package.json")),
        },
        wrapperImages,
      });
      const expected = new Map<string, string>();
      for (let index = 0; index < 8; index++) {
        const name = join(project, `src/File${index}.vue`),
          text = fixture(index % 2 === 1, pipeCapacity);
        writeFileSync(name, text);
        expected.set(name, text);
      }
      const config = hostConfig({
        strict: true,
        target: "ESNext",
        module: "ESNext",
        moduleResolution: "Bundler",
        skipLibCheck: false,
        types: [],
      });
      save(join(project, "tsconfig.json"), config);
      save(join(project, "package.json"), { private: true, type: "module" });
      const inputImages = [...expected.keys()].map(fileImage);
      inputImages.push(
        fileImage(join(project, "tsconfig.json")),
        fileImage(join(project, "package.json")),
        fileImage(join(packageRoot, "package.json")),
      );
      const result = await run(
        join(directory, "host"),
        [
          process.execPath,
          entry.command,
          "--runExternalCode",
          "--noEmit",
          "--pretty",
          "false",
          "-p",
          "tsconfig.json",
        ],
        project,
        entry.binary,
        fileImage(join(nativeRoot, basename(nativeImage.path))),
      );
      assertCustody(wrapperImages);
      assertCustody(inputImages);
      assertCustody(fixed);
      assertDeclarationCustody(packageRoot, sdk.files, sdk.edges);
      assertDeclarationCustody(sourcePackage, declarations.files, declarations.edges);
      save(join(directory, "post-custody.json"), {
        source: values["source-sha"],
        wrappers: wrapperImages,
        inputs: inputImages,
        fixed,
        stagedSdk: sdk,
        sourceSdk: declarations,
        allMatched: true,
      });
      const diagnostics = semanticPacket(
        result.status,
        result.text,
        result.errors,
        project,
        [1, 3, 5, 7],
      );
      for (const value of result.observedHosts) {
        const argv = object(value).argv;
        assert.ok(Array.isArray(argv));
        assert.ok(
          !argv.some((value) =>
            /^(?:--fast|--checkers|--extendedDiagnostics|--pprofDir)(?:=|$)/u.test(String(value)),
          ),
          "actual Go host acquired a nondefault or profiling flag",
        );
      }
      if (!captureMode)
        for (const value of result.observedNatives) {
          const argv = object(value).argv;
          assert.ok(Array.isArray(argv) && argv.at(-1) === "content-mapper");
          assert.ok(
            !argv.some(
              (value) =>
                String(value).includes("admission-hosts") || String(value).startsWith("--import"),
            ),
            "normal mapper acquired a tee or preload observer",
          );
        }
      if (captureMode) {
        const sessions = readdirSync(capture).sort();
        assert.equal(sessions.length, 1, "exactly one mapper session expected");
        const session = join(capture, sessions[0]);
        const identity = json(join(session, "native-identity.json"));
        assert.equal(identity.token, values.token);
        assert.ok(Array.isArray(identity.images) && identity.images.length === 1);
        assert.equal(object(identity.images[0]).sha256, nativeImage.sha256);
        assert.deepEqual(json(join(session, "workers.json")), {
          requestedWorkers: 1,
          effectiveWorkers: 1,
          spawnFailure: null,
        });
        assert.equal(readFileSync(join(session, "stderr.raw"), "utf8"), "");
        const wire = qualifyWire(
          readFileSync(join(session, "stdin.raw")),
          readFileSync(join(session, "stdout.raw")),
          expected,
          pipeCapacity,
        );
        save(join(directory, "whole-protocol.json"), wire);
        const resultPath = join(session, "result.json");
        const sidecarObserved = existsSync(resultPath);
        save(join(directory, "capture-observation.json"), {
          sidecarObserved,
          childExit: mapperExitReceipt(sidecarObserved ? json(resultPath) : undefined),
          host: {
            status: result.status,
            signal: result.signal,
            closeObserved: result.closeObserved,
            error: result.error,
            timedOut: result.timedOut,
            survivors: result.survivors,
            remaining: result.remaining,
          },
          knownProcesses: result.observedProcesses,
          terminalProofs: result.terminatedProcesses,
          raw: ["stdin.raw", "stdout.raw", "stderr.raw"].map((name) =>
            fileImage(join(session, name)),
          ),
          completeProtocol: true,
          postCustody: fileImage(join(directory, "post-custody.json")),
        });
        save(join(directory, "diagnostics.json"), {
          diagnostics,
          endAndRelated: "unavailable from text CLI; no full authored-range qualification",
        });
      } else {
        assert.deepEqual(readdirSync(capture), [], "normal invocation acquired an observer");
        save(join(directory, "diagnostics.json"), {
          diagnostics,
          endAndRelated: "unavailable from text CLI; no full authored-range qualification",
        });
      }
      packets.push({
        label,
        captureMode,
        completed: true,
        authoredSources: expected.size,
        diagnostics,
      });
    }
  }
  assert.equal(
    packets.length,
    4,
    "both hosts require normal completion and a separate complete capture",
  );
  for (const packet of packets)
    assert.deepEqual(
      packet.diagnostics,
      packets[0].diagnostics,
      "normal/capture pinned host whole diagnostic identity changed",
    );
  assert.equal(
    execFileSync("git", ["status", "--porcelain"], { cwd: source, encoding: "utf8" }),
    "",
    "source changed during functional qualification",
  );
  save(join(output, "acceptance.json"), {
    source: values["source-sha"],
    packets,
    qualified: true,
    scope:
      "two hosts: normal pipe liveness first, then separate instrumented whole-protocol replay; mixed valid4/negative4 source controls",
    performanceClaim: false,
  });
}
if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url)
  main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });

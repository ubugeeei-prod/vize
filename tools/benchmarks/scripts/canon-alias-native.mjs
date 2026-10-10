#!/usr/bin/env node
/** Literal base/head CLI inputs, stock TS/native oracles and whole raw packets. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { basename, dirname, isAbsolute, join, resolve } from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";
import { performance } from "node:perf_hooks";
import { assertBinariesUnchanged, fileSha256, hashInPlace } from "./benchmark-binary.mjs";
import { corpusManifest } from "./type-snapshot-cli-corpus.mjs";
import { prepareRun } from "./type-snapshot-cli-runner.mjs";
import { normalizeTypecheckResult } from "./typecheck-command.mjs";
import {
  cases,
  fixtureRoot,
  prepareAliasCase,
  expectedCliReport,
  expectedOriginalDiagnostics,
  expectedServerPacket,
} from "./canon-alias-corpus.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const writeJson = (path, value) => writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);

export function captureAliasCorpus({
  directory,
  commands,
  runtimePath,
  typescriptPath,
  vuePackageDir,
  baseMode = "observed-regression",
}) {
  let sequence = 0;
  const samples = [];
  const ts = createRequire(import.meta.url)(join(dirname(typescriptPath), "../lib/typescript.js"));
  mkdirSync(join(directory, "raw"), { recursive: true });
  const fixture = corpusManifest(fixtureRoot);
  function execute(command, args, cwd, label, input) {
    const id = `${String(sequence++).padStart(3, "0")}-${label}`;
    const start = performance.now();
    const result = spawnSync(command[0], [...command.slice(1), ...args], {
      cwd,
      input,
      encoding: "utf8",
      timeout: 120_000,
      maxBuffer: 16 * 1024 * 1024,
      env: { ...process.env, NO_COLOR: "1" },
    });
    const packet = {
      id,
      command: [...command, ...args],
      cwd,
      stdin: input ?? null,
      status: result.status,
      signal: result.signal,
      stdout: result.stdout ?? "",
      stderr: result.stderr ?? "",
      ms: performance.now() - start,
      error: result.error?.message ?? null,
    };
    writeJson(join(directory, "raw", `${id}.json`), packet);
    samples.push(packet);
    assert.equal(packet.error, null, `${id}: process failed`);
    assert.equal(packet.signal, null, `${id}: signal death`);
    assert([0, 1, 2].includes(packet.status), `${id}: unexpected exit`);
    return packet;
  }
  const qualified = [];
  const expectedCode = (item, side) =>
    side === "base" && baseMode === "observed-regression" ? item.before : item.after;
  for (const item of cases) {
    const project = prepareAliasCase(join(directory, "inputs"), item, "tsx", vuePackageDir);
    writeJson(join(directory, "inputs", item.id, "tsx.manifest.json"), project.manifest);
    for (const { name, command } of [
      { name: "ts6", command: [process.execPath, typescriptPath] },
      { name: "native7", command: [runtimePath] },
    ]) {
      const original = execute(
        command,
        ["--project", "tsconfig.json", "--pretty", "false"],
        project.root,
        `${item.id}-${name}`,
      );
      const report = normalizeTypecheckResult(original, project.root, "plain");
      assert.equal(original.stderr, "", `${item.id}/${name}: unexpected oracle stderr`);
      assert.equal(original.status, item.after == null ? 0 : name === "ts6" ? 2 : 1);
      assert.deepEqual(
        report.diagnostics,
        expectedOriginalDiagnostics(item.after),
        `${item.id}/${name}: independent oracle`,
      );
    }
    const reports = {};
    for (const [side, command] of Object.entries(commands)) {
      const expected = expectedCode(item, side);
      const result = execute(
        command,
        ["check", "App.tsx", "--quiet", "--format", "json", "--corsa-path", runtimePath],
        project.root,
        `${item.id}-${side}-tsx`,
      );
      assert.equal(result.stderr, "", `${item.id}/${side}: unexpected check stderr`);
      const report = JSON.parse(result.stdout);
      assert.deepEqual(
        report,
        expectedCliReport(item, project.root, expected),
        `${item.id}/${side}: complete authored CLI report`,
      );
      assert.equal(result.status, expected == null ? 0 : 1);
      reports[side] = report;
    }
    if (expectedCode(item, "base") === item.after && reports.base && reports.head)
      assert.deepEqual(reports.head, reports.base, `${item.id}: complete unaffected CLI report`);
    assert.deepEqual(
      corpusManifest(project.root),
      project.manifest,
      `${item.id}: CLI input mutation`,
    );

    const server = prepareAliasCase(join(directory, "inputs"), item, "server", vuePackageDir);
    writeJson(join(directory, "inputs", item.id, "server.manifest.json"), server.manifest);
    const content = readFileSync(join(server.root, "App.vue"), "utf8");
    const input = [
      {
        jsonrpc: "2.0",
        id: 1,
        method: "check",
        params: { uri: join(server.root, "App.vue"), content, flags: "" },
      },
      { jsonrpc: "2.0", id: 2, method: "shutdown" },
    ]
      .map((request) => `${JSON.stringify(request)}\n`)
      .join("");
    for (const [side, command] of Object.entries(commands)) {
      const result = execute(
        command,
        ["check-server", "--working-dir", server.root, "--corsa-path", runtimePath],
        server.root,
        `${item.id}-${side}-server`,
        input,
      );
      assert.equal(result.status, 0, `${item.id}/${side}: server termination`);
      const replies = result.stdout
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line));
      assert.equal(replies.length, 2, `${item.id}/${side}: complete reply set`);
      assert.deepEqual(replies[1], { jsonrpc: "2.0", id: 2, result: { status: "shutdown" } });
      assert.equal(replies[0].jsonrpc, "2.0");
      assert.equal(replies[0].id, 1);
      assert.equal(replies[0].error, undefined);
      const { virtualTs, ...packet } = replies[0].result;
      assert.equal(typeof virtualTs, "string");
      assert(virtualTs.length > 0, "native project producer must execute");
      const parsed = ts.createSourceFile("App.vue.ts", virtualTs, ts.ScriptTarget.Latest);
      const imports = parsed.statements.filter(
        (statement) =>
          ts.isImportDeclaration(statement) &&
          statement.importClause?.namedBindings != null &&
          ts.isNamedImports(statement.importClause?.namedBindings) &&
          statement.importClause.namedBindings.elements.some(
            (element) => element.name.text === "value",
          ),
      );
      assert.equal(imports.length, 1, "one authored value import must survive the producer");
      assert(ts.isStringLiteral(imports[0].moduleSpecifier));
      const module = imports[0].moduleSpecifier.text;
      const mirror =
        side === "base" && baseMode === "observed-regression"
          ? item.beforeMirror
          : item.afterMirror;
      if (mirror == null) assert.equal(module, "@x", "missing selected route must remain authored");
      else {
        assert(isAbsolute(module), "a resolved alias must name its physical native mirror");
        assert.equal(basename(module), mirror, "literal authored alias target");
        assert.match(
          module.replaceAll("\\", "/"),
          /\/vize-canon\/editor\/sessions\/session-[^/]+\/projects\/[a-f0-9]{64}\//u,
        );
      }
      assert.deepEqual(
        packet,
        expectedServerPacket(expectedCode(item, side)),
        `${item.id}/${side}: full server diagnostic packet`,
      );
    }
    assert.deepEqual(
      corpusManifest(server.root),
      server.manifest,
      `${item.id}: server input mutation`,
    );
    qualified.push({
      id: item.id,
      before: item.before,
      after: item.after,
      tsxInput: project.manifest,
      serverInput: server.manifest,
    });
  }
  assert.deepEqual(corpusManifest(fixtureRoot), fixture, "authored fixture changed");
  return {
    fixture,
    baseMode,
    qualified,
    rawSamples: samples.map(({ id, ms, status }) => ({ id, ms, status })),
    timingClaim: "none; paired 500-SFC throughput is reported separately",
  };
}

export function main(argv = process.argv.slice(2)) {
  assert.equal(argv.length, 3, "usage: canon-alias-native.mjs BASE_BIN HEAD_BIN OUTPUT");
  const output = resolve(argv[2]);
  const directory = `${output}.samples`;
  assert(!existsSync(output) && !existsSync(directory), "fresh output required");
  mkdirSync(join(directory, "work"), { recursive: true });
  let metadata;
  try {
    const prepared = prepareRun(argv[0], argv[1], directory, ROOT);
    const typescriptPath = realpathSync(
      createRequire(realpathSync(join(ROOT, "node_modules/vite-plus/package.json"))).resolve(
        "typescript/bin/tsc",
      ),
    );
    const typescriptPackage = JSON.parse(
      readFileSync(join(dirname(typescriptPath), "../package.json"), "utf8"),
    );
    assert.equal(typescriptPackage.version, "6.0.3");
    assert.equal(prepared.metadata.dependencies.runtimePackageVersion, "7.0.2");
    const typescriptRoot = resolve(dirname(typescriptPath), "..");
    const typescriptPayload = corpusManifest(typescriptRoot);
    const originals = {
      node: hashInPlace(realpathSync(process.execPath)),
      typescript: hashInPlace(typescriptPath),
    };
    const baseCheckout = resolve(dirname(prepared.binarySources.base), "../..");
    const baseResolverSha256 = fileSha256(
      join(
        baseCheckout,
        "crates/vize_canon/src/batch/virtual_project/dependency_scan/resolution.rs",
      ),
    );
    // Only the byte-exact observed old source receives the frozen wrong packets.
    // Later baselines must satisfy the original typed contract themselves.
    const baseMode =
      baseResolverSha256 === "5a43aa38a8beb0f4005afb4e7ef94e2fcf96dfef53c9b19166471d10e7add833"
        ? "observed-regression"
        : "original-contract";
    metadata = {
      ...prepared.metadata,
      kind: "canon-path-alias-precedence-native",
      baseResolverSha256,
      baseMode,
      originals,
      stockTypeScript: { root: typescriptRoot, version: "6.0.3", payload: typescriptPayload },
      driverSha256: fileSha256(fileURLToPath(import.meta.url)),
      corpusDriverSha256: fileSha256(join(ROOT, "tools/benchmarks/scripts/canon-alias-corpus.mjs")),
    };
    writeJson(join(directory, "provenance.json"), metadata);
    const results = captureAliasCorpus({
      directory,
      commands: Object.fromEntries(
        Object.entries(prepared.binaries)
          .filter(([side]) => ["base", "head"].includes(side))
          .map(([side, binary]) => [side, [binary.measuredPath]]),
      ),
      runtimePath: prepared.runtimePath,
      typescriptPath,
      vuePackageDir: prepared.vuePackageDir,
      baseMode,
    });
    assertBinariesUnchanged(prepared.binaries);
    assertBinariesUnchanged(originals);
    assert.deepEqual(
      corpusManifest(typescriptRoot),
      typescriptPayload,
      "stock TypeScript payload changed",
    );
    writeJson(output, { ...metadata, ...results });
  } catch (error) {
    writeJson(join(directory, "failure.json"), {
      metadata,
      message: error.message,
      stack: error.stack,
    });
    throw error;
  }
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) main();

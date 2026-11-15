/** Fresh-Bun public acceptance for the no-unused-script-vars cases reported in #12999. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join, resolve, sep } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const rule = "vue/no-unused-script-vars";
const docs = "docs/content/rules/reference/vue-no-unused-script-vars.md";

type DiagnosticSpec = { name: string; marker: string; line: number };
type GlobalCase = { id: string; code: string; diagnostics: DiagnosticSpec[] };
type Corpus = {
  schema: string;
  version: number;
  typescriptEslint: string;
  options: Record<string, unknown>;
  globalCases: GlobalCase[];
  quietCase: { id: string; code: string };
  oppositeCase: { id: string; code: string };
};
type Position = { line: number; column: number };
type Message = {
  ruleId: string;
  ruleDocsPath: string;
  severity: number;
  message: string;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
};
type Report = { file: string; messages: Message[]; errorCount: number; warningCount: number };

export const filename = "UnusedScriptVarsCase.vue";

export function source(code: string): string {
  return `<script setup lang="ts">\n${code}\n</script>\n\n<template><div /></template>\n`;
}

export function authoredPosition(text: string, offset: number): Position {
  const prefix = text.slice(0, offset);
  const lines = prefix.split("\n");
  return { line: lines.length, column: Array.from(lines.at(-1)!).length + 1 };
}

function message(text: string, name: string, marker: string, body: string): Message {
  const markerStart = text.indexOf(marker);
  assert.notEqual(markerStart, -1, marker);
  const nameInMarker = marker.lastIndexOf(name);
  assert.notEqual(nameInMarker, -1, `${name} in ${marker}`);
  const start = markerStart + nameInMarker;
  const from = authoredPosition(text, start);
  const to = authoredPosition(text, start + name.length);
  return {
    ruleId: rule,
    ruleDocsPath: docs,
    severity: 1,
    message: `[vize:${rule}] ${body}`,
    line: from.line,
    column: from.column,
    endLine: to.line,
    endColumn: to.column,
  };
}

function packet(messages: Message[]): Report[] {
  return [{ file: filename, messages, errorCount: 0, warningCount: messages.length }];
}

export function expectedGlobalPacket(row: GlobalCase): Report[] {
  const text = source(row.code);
  return packet(
    row.diagnostics.map(({ name, marker }) =>
      message(
        text,
        name,
        marker,
        `'${name}' is defined but never used. Allowed unused vars must match /^_/u`,
      ),
    ),
  );
}

export function expectedQuietPacket(): Report[] {
  return packet([]);
}

export function expectedOppositePacket(code: string): Report[] {
  const text = source(code);
  return packet([
    message(
      text,
      "a",
      "foo(a)",
      "'a' is defined but never used. Allowed unused args must match /^_/u",
    ),
    message(
      text,
      "cb",
      "  cb = (function",
      "'cb' is assigned a value but never used. Allowed unused args must match /^_/u",
    ),
  ]);
}

export function configSource(options: Record<string, unknown>): string {
  return `export default ${JSON.stringify({
    vue: {
      linter: {
        preset: "incremental",
        rules: { [rule]: "warn" },
        ruleOptions: { [rule]: options },
      },
    },
  })};\n`;
}

export function validateRequest(
  version: string,
  candidate: string,
  releaseSource: string,
  producerRun: string,
  environment: NodeJS.ProcessEnv,
): void {
  assert.match(version, /^\d+\.\d+\.\d+$/u);
  assert.match(candidate, /^[0-9a-f]{40}$/u);
  assert.match(releaseSource, /^[0-9a-f]{40}$/u);
  assert.match(producerRun, /^[1-9][0-9]*$/u);
  assert.equal(environment.GITHUB_ACTIONS, "true");
  assert.equal(environment.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
  assert.equal(environment.GITHUB_REF, "refs/heads/main");
  assert.equal(environment.GITHUB_EVENT_NAME, "workflow_dispatch");
  assert.match(
    environment.GITHUB_WORKFLOW_REF ?? "",
    /ubugeeei-prod\/vize\/\.github\/workflows\/no-unused-script-vars-public\.yml@refs\/heads\/main$/u,
  );
  for (const name of [
    "NODE_PATH",
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
    "VIZE_NATIVE_PATH",
    "VIZE_PREFER_WORKSPACE_BINDING",
  ])
    assert.equal(environment[name], undefined, `${name} must not override the public install`);
}

type ProcessPacket = {
  argv: string[];
  cwd: string;
  status: number | null;
  signal: NodeJS.Signals | null;
  error: string | null;
  stdout: string;
  stderr: string;
};

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

function execute(proof: string, name: string, argv: string[], cwd: string): ProcessPacket {
  const result = spawnSync(argv[0], argv.slice(1), {
    cwd,
    encoding: "utf8",
    shell: false,
    env: {
      ...process.env,
      NPM_CONFIG_REGISTRY: "https://registry.npmjs.org/",
    },
  });
  const output: ProcessPacket = {
    argv,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
  writeFileSync(join(proof, `${name}.json`), `${JSON.stringify(output, null, 2)}\n`);
  writeFileSync(join(proof, `${name}.stdout`), output.stdout);
  writeFileSync(join(proof, `${name}.stderr`), output.stderr);
  return output;
}

function success(output: ProcessPacket): void {
  assert.equal(output.error, null);
  assert.equal(output.signal, null);
  assert.equal(output.status, 0, `${output.stdout}\n${output.stderr}`);
}

function assertLint(
  proof: string,
  project: string,
  name: string,
  code: string,
  expected: Report[],
): void {
  writeFileSync(join(project, filename), source(code));
  const output = execute(
    proof,
    name,
    [
      process.execPath,
      join(project, "node_modules/vize/bin/vize"),
      "lint",
      "--config",
      "vite.config.mjs",
      "--format",
      "json",
      filename,
    ],
    project,
  );
  success(output);
  assert.deepEqual(JSON.parse(output.stdout), expected, name);
}

export async function run(args: string[]): Promise<void> {
  assert.equal(args.length, 5);
  const [version, candidate, releaseSource, producerRun, outputArgument] = args;
  assert(version && candidate && releaseSource && producerRun && outputArgument);
  validateRequest(version, candidate, releaseSource, producerRun, process.env);
  const bunVersion = String(Reflect.get(process.versions, "bun"));
  assert.equal(bunVersion, "1.3.14");

  const runnerTemp = realpathSync(process.env.RUNNER_TEMP!);
  const output = resolve(outputArgument);
  assert.ok(output.startsWith(runnerTemp + sep));
  assert.equal(existsSync(output), false);
  const proof = join(output, "proof");
  const project = join(output, "consumer");
  mkdirSync(proof, { recursive: true });
  mkdirSync(project, { recursive: true });

  const corpusPath = resolve(
    dirname(fileURLToPath(import.meta.url)),
    "../../../../tests/_fixtures/differential/linter/no-unused-script-vars-12999/corpus.json",
  );
  const corpusBytes = readFileSync(corpusPath);
  const corpus = JSON.parse(corpusBytes.toString("utf8")) as Corpus;
  assert.equal(corpus.schema, "vize.no-unused-script-vars.corpus");
  assert.equal(corpus.version, 1);
  assert.equal(corpus.typescriptEslint, "8.72.2");
  assert.equal(corpus.globalCases.length, 11);

  writeFileSync(
    join(project, "package.json"),
    `${JSON.stringify({ private: true, dependencies: { vize: version } }, null, 2)}\n`,
  );
  writeFileSync(join(project, "vite.config.mjs"), configSource(corpus.options));
  const install = execute(
    proof,
    "bun-install",
    [process.execPath, "install", "--no-progress"],
    project,
  );
  success(install);

  const installedManifestPath = join(project, "node_modules/vize/package.json");
  assert.ok(realpathSync(installedManifestPath).startsWith(project + sep));
  const installedManifest = JSON.parse(readFileSync(installedManifestPath, "utf8"));
  assert.equal(installedManifest.version, version);
  const metadataResponse = await fetch(
    `https://registry.npmjs.org/${encodeURIComponent("vize")}/${version}`,
  );
  assert.equal(metadataResponse.status, 200);
  const metadata = (await metadataResponse.json()) as {
    version?: string;
    dist?: { integrity?: string; tarball?: string };
  };
  assert.equal(metadata.version, version);
  const integrity = metadata.dist?.integrity;
  const tarball = metadata.dist?.tarball;
  assert.ok(integrity);
  assert.ok(tarball);

  for (const row of corpus.globalCases)
    assertLint(proof, project, row.id, row.code, expectedGlobalPacket(row));
  assertLint(proof, project, corpus.quietCase.id, corpus.quietCase.code, expectedQuietPacket());
  assertLint(
    proof,
    project,
    corpus.oppositeCase.id,
    corpus.oppositeCase.code,
    expectedOppositePacket(corpus.oppositeCase.code),
  );

  writeFileSync(
    join(proof, "complete.json"),
    `${JSON.stringify(
      {
        schema: "vize.no-unused-script-vars.public.v1",
        complete: true,
        version,
        candidate,
        releaseSource,
        producerRun: Number(producerRun),
        bun: bunVersion,
        package: {
          version: installedManifest.version,
          integrity,
          tarball,
        },
        corpus: { sha256: sha256(corpusBytes), globals: 11, quiet: 1, opposite: 1 },
        packets: 13,
      },
      null,
      2,
    )}\n`,
  );
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await run(process.argv.slice(2));

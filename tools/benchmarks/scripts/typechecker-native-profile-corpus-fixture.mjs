/** Deterministic real-filesystem builders used only by native profile corpus tests. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readlinkSync,
  realpathSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { corpusManifest } from "./type-snapshot-cli-corpus.mjs";
import { diagnosticFingerprint } from "./typecheck-command.mjs";

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const write = (path, bytes) => {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, bytes);
};
function revision(path) {
  const stat = lstatSync(path, { bigint: true });
  return Object.fromEntries(
    ["dev", "ino", "mode", "size", "mtimeNs", "ctimeNs"].map((key) => [key, String(stat[key])]),
  );
}
function identity(path, links = []) {
  const realPath = realpathSync(path);
  const bytes = readFileSync(path);
  return {
    path,
    realPath,
    links,
    revision: revision(realPath),
    bytes: bytes.length,
    sha256: sha256(bytes),
  };
}
export function editArchive(shard, change) {
  const graph = JSON.parse(readFileSync(shard.nativeGraphArchive.record));
  change(graph);
  const bytes = JSON.stringify(graph);
  writeFileSync(shard.nativeGraphArchive.record, bytes);
  shard.nativeGraphArchive.sha256 = sha256(bytes);
}

export function fixture(mutate = () => {}) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "native-profile-corpus-")));
  const original = join(root, "original");
  const directory = join(root, "receipts");
  const dependencies = join(root, "dependency-tree");
  for (const path of [original, directory, dependencies]) mkdirSync(path);
  mkdirSync(join(original, "nested"));
  const authored = {
    "A.vue": Buffer.from(
      '\uFEFF<script setup lang="ts">const a = "雪";</script>\r\n<template>{{ a }}</template>\r\n',
    ),
    "B.vue": Buffer.from("<template><div>{{ absent }}</div></template>\n"),
    "nested/model.ts": Buffer.from("export interface Model { value: number }\n"),
    "globals.d.ts": Buffer.from("declare const globalValue: number;\n"),
    "tsconfig.json": Buffer.from('{"compilerOptions":{"strict":true},"include":["**/*"]}\n'),
    "package.json": Buffer.from('{"private":true,"type":"module"}\n'),
  };
  for (const [name, bytes] of Object.entries(authored)) writeFileSync(join(original, name), bytes);
  writeFileSync(join(dependencies, "package.json"), '{"name":"controlled-dependency"}\n');
  writeFileSync(join(dependencies, "index.d.ts"), "export interface External { value: number }\n");
  const runtime = join(root, "controlled-runtime");
  writeFileSync(runtime, "controlled pinned runtime bytes\n");
  symlinkSync(dependencies, join(original, "node_modules"), "dir");
  const corpus = { id: "controlled", dir: original, expectedVueFiles: 2, args: ["."] };
  corpus.manifest = corpusManifest(original);
  const mode = { id: "max", args: [] };
  const label = "controlled-max";
  const duplicateRoot = join(directory, "work", `${label}-profile-corpus`);
  const report = {
    success: false,
    errorCount: 2,
    warningCount: 0,
    programs: [{ files: ["A.vue", "B.vue", "nested/model.ts", "globals.d.ts"] }],
    files: ["A.vue", "B.vue"].map((file, index) => ({
      file,
      diagnostics: [
        {
          code: 2339 + index,
          severity: 1,
          message: `ordered diagnostic ${index}`,
          range: { start: { line: index, character: 2 }, end: { line: index, character: 7 } },
          relatedInformation: [
            { file: "nested/model.ts", message: "full related message", line: 1 },
          ],
        },
      ],
    })),
  };
  const projection = {
    files: ["A.vue", "B.vue"].map((file, index) => ({
      file,
      code: `const generated${index} = ${index};\n`,
      maps: [{ original: [2, 8], generated: [6, 12] }],
      links: [{ source: "template", target: "script", span: [0, 6] }],
    })),
  };
  const virtualFiles = projection.files.map(({ file, code }) => ({
    file: `${file}.virtual.ts`,
    bytes: Buffer.byteLength(code),
    sha256: sha256(code),
  }));
  const context = { root, original, directory, duplicateRoot, authored, mode, label, calls: [] };
  function shards(authoredRoot) {
    const key = sha256(`vize-canon-project-key:v1\0${realpathSync(authoredRoot)}`);
    const cwd = join(
      root,
      context.alternateCache && authoredRoot !== original ? "other-cache" : "cache",
      "projects",
      key,
    );
    mkdirSync(join(cwd, "node_modules"), { recursive: true });
    const link = join(cwd, "node_modules", "dep");
    symlinkSync(dependencies, link, "dir");
    write(join(cwd, "globals.d.ts"), authored["globals.d.ts"]);
    write(join(cwd, "nested/model.ts"), authored["nested/model.ts"]);
    return ["A.vue.ts", "B.vue.ts"].map((name, index) => {
      write(join(cwd, name), projection.files[index].code);
      const compilerOptions = { strict: true, moduleResolution: "bundler", types: [] };
      const files = [name, "globals.d.ts", "nested/model.ts", "node_modules/dep/index.d.ts"];
      const config = join(cwd, `tsconfig.shard${index}.json`);
      write(config, JSON.stringify({ compilerOptions, files }));
      const args = ["--checkers", "1", "--pretty", "false", "--project", config];
      const rows = files.map((file, memberIndex) => ({
        index: memberIndex,
        reportedPath: file,
        ...identity(
          join(cwd, file),
          file.startsWith("node_modules")
            ? [{ path: link, target: readlinkSync(link), revision: revision(link) }]
            : [],
        ),
      }));
      const graph = {
        schemaVersion: 1,
        cwd,
        args,
        runtime: identity(runtime),
        config: { ...identity(config), declaredRootSelection: { files } },
        membersBefore: rows,
        membersAfter: structuredClone(rows),
      };
      const record = join(cwd, `shard${index}.graph.json`);
      write(record, JSON.stringify(graph));
      return {
        cwd,
        config,
        configSha256: graph.config.sha256,
        runtimeSha256: graph.runtime.sha256,
        args,
        compilerOptions,
        members: rows.map(({ path, bytes, sha256 }) => ({ path, bytes, sha256 })),
        nativeGraphArchive: { record, sha256: sha256(readFileSync(record)) },
        nativeProfile: {
          validation: "passed",
          profileFiles: [{ kind: "cpu" }, { kind: "allocations" }],
        },
      };
    });
  }
  const reference = {
    direct: { fingerprint: diagnosticFingerprint(report), virtualFiles },
    wrapped: { nativeShards: shards(original) },
  };
  const options = {
    corpus,
    mode,
    label,
    directory,
    reference,
    project(input, actualLabel) {
      assert.equal(input.dir, duplicateRoot, "project must use duplicate cwd");
      assert.equal(input.id, corpus.id);
      context.calls.push(actualLabel);
      const output = structuredClone(projection);
      mutate({
        context,
        projection: output,
        stage: actualLabel.endsWith("-after") ? "after" : "before",
      });
      const text = JSON.stringify(output);
      return { text, sha256: sha256(text) };
    },
    pair(input, actualMode, actualLabel, virtual, profile) {
      assert.equal(input.dir, duplicateRoot, "pair must use duplicate cwd");
      assert.equal(actualMode, mode);
      assert.equal(actualLabel, `${label}-profile`);
      assert.equal(virtual, true);
      assert.equal(profile, true);
      for (const [name, bytes] of Object.entries(authored)) {
        assert(readFileSync(join(input.dir, name)).equals(bytes), `authored bytes differ: ${name}`);
      }
      const modules = join(input.dir, "node_modules");
      assert(lstatSync(modules).isSymbolicLink(), "dependency tree was dereferenced");
      assert.equal(readlinkSync(modules), readlinkSync(join(original, "node_modules")));
      assert.equal(realpathSync(modules), realpathSync(join(original, "node_modules")));
      context.calls.push(actualLabel);
      const changedReport = structuredClone(report);
      const checked = {
        direct: { id: "controlled-direct", virtualFiles: structuredClone(virtualFiles) },
        wrapped: { id: "controlled-wrapped", nativeShards: shards(input.dir) },
      };
      mutate({ context, checked, report: changedReport, reference, stage: "pair" });
      checked.direct.fingerprint = diagnosticFingerprint(changedReport);
      return checked;
    },
  };
  return { ...context, options };
}

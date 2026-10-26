import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import ts from "typescript";
import { createVizeLanguageService } from "../src/index.js";
import { toUtf16LanguageProjection } from "@vizejs/native/language-projection.js";

const require = createRequire(import.meta.url);

export const directory = path.dirname(fileURLToPath(import.meta.url));
export const workspace = path.join(directory, "fixture");
export const evidence = process.env.VIZE_VOLAR_EVIDENCE_DIR ?? path.join(directory, ".artifacts");
export const packets = [];
export const inputs = [];
export const timings = [];
export const hash = (text) => createHash("sha256").update(text).digest("hex");

export function normalize(value) {
  return JSON.parse(
    JSON.stringify(value, (key, item) => {
      if (key === "file" && item?.fileName && typeof item.text === "string")
        return { fileName: item.fileName.replaceAll(workspace, "<workspace>"), text: item.text };
      if (typeof item === "string") return item.replaceAll(workspace, "<workspace>");
      return item;
    }),
  );
}

export function query(name, params, invoke) {
  const started = performance.now();
  const result = invoke();
  timings.push({ name, elapsedMs: performance.now() - started });
  packets.push(normalize({ name, params, result }));
  return result;
}

export function fixture(name, files, extraOptions = {}, languageOptions = {}, overrides = {}) {
  const documents = new Map(
    Object.entries(files).map(([file, source]) => [
      path.join(workspace, file),
      { source, version: 1, snapshot: ts.ScriptSnapshot.fromString(source) },
    ]),
  );
  const deleted = new Set();
  let revision = 1;
  const settings = {
    target: ts.ScriptTarget.ESNext,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    strict: true,
    jsx: ts.JsxEmit.Preserve,
    skipLibCheck: true,
    ...extraOptions,
  };
  const host = {
    getCurrentDirectory: () => workspace,
    getCompilationSettings: () => settings,
    getScriptFileNames: () => [...documents.keys()],
    getScriptVersion: (file) => String(documents.get(file)?.version ?? "disk"),
    getProjectVersion: () => String(revision),
    getScriptSnapshot: (file) => {
      if (deleted.has(file)) return;
      const document = documents.get(file);
      if (document) return document.snapshot;
      const text = ts.sys.readFile(file);
      return text === undefined ? undefined : ts.ScriptSnapshot.fromString(text);
    },
    getDefaultLibFileName: ts.getDefaultLibFilePath,
    fileExists: (file) => !deleted.has(file) && (documents.has(file) || ts.sys.fileExists(file)),
    readFile: (file) =>
      deleted.has(file) ? undefined : (documents.get(file)?.source ?? ts.sys.readFile(file)),
    readDirectory: (...args) => ts.sys.readDirectory(...args),
    directoryExists: (dir) => dir.startsWith(workspace) || ts.sys.directoryExists(dir),
    realpath: (file) => ts.sys.realpath?.(file) ?? file,
  };
  Object.assign(host, overrides.host);
  const service = createVizeLanguageService(overrides.typescript ?? ts, host, languageOptions);
  inputs.push({ name, files, compilerOptions: settings, languageOptions });
  return {
    ...service,
    path: (file) => path.join(workspace, file),
    text: (file) => documents.get(path.join(workspace, file)).source,
    update(file, source) {
      const id = path.join(workspace, file);
      documents.set(id, {
        source,
        version: ++revision,
        snapshot: ts.ScriptSnapshot.fromString(source),
      });
      deleted.delete(id);
      inputs.push({ name, update: file, source, revision });
    },
    delete(file) {
      const id = path.join(workspace, file);
      documents.delete(id);
      deleted.add(id);
      revision++;
      inputs.push({ name, delete: file, revision });
    },
    capture(stage) {
      for (const [file, document] of documents) {
        if (!/\.(vue|art)$/.test(file)) continue;
        const root = service.language.scripts.get(file)?.generated?.root;
        if (!root) continue;
        const { includeArt: _includeArt, ...generationOptions } = languageOptions;
        const rawTransformJSON = require("@vizejs/native").generateVueLanguageCode(
          document.source,
          { ...generationOptions, filename: file },
        );
        assert.deepEqual(toUtf16LanguageProjection(document.source, rawTransformJSON), {
          ...root.projection,
          diagnostics: root.projection.diagnostics.filter(
            (diagnostic) => diagnostic.code !== 100003,
          ),
        });
        inputs.push({
          name,
          stage,
          file,
          sourceSha256: hash(document.source),
          rawTransformJSON,
          projection: root.projection,
        });
      }
    },
  };
}

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { admissionFiles } from "./fixtures/type-admission-project.ts";
import { globalRefAdmission } from "./fixtures/type-admission-global-fixtures.ts";
import { typeProofs, mutatedAdmission } from "./fixtures/type-admission-fixtures.ts";
import {
  array,
  hash,
  integer,
  parse,
  record,
  string,
  write,
  type Expected,
  type Runtime,
} from "./model.ts";
export { admissionFiles, globalRefAdmission, mutatedAdmission };
export type Project = { directory: string; files: Record<string, string>; expected: Expected[] };
export function verifyOriginals(capsule: string): unknown {
  const manifest = record(parse(fs.readFileSync(path.join(capsule, "original-inputs.json.txt"))));
  assert.equal(manifest.schema, "vize.template-ref.public-capsule.v1");
  const originalNames = [
    "type-admission-fixtures.ts",
    "type-admission-global-fixtures.ts",
    "type-admission-mutation.ts",
    "type-admission-project.ts",
    "type-admission-ref-edges.ts",
  ];
  assert.deepEqual(
    array(manifest.files)
      .map((value) => string(record(value).name))
      .sort(),
    originalNames.sort(),
  );
  for (const value of array(manifest.files)) {
    const row = record(value),
      name = string(row.name);
    if (!/^type-admission-[a-z-]+\.ts$/u.test(name)) throw new Error("unowned fixture name");
    const bytes = fs.readFileSync(path.join(capsule, "fixtures", name));
    if (hash(bytes) !== string(row.sha256) || bytes.length !== integer(row.bytes))
      throw new Error("original fixture changed: " + name);
  }
  const drivers = array(manifest.driverFiles);
  assert.deepEqual(drivers.map((value) => string(record(value).name)).sort(), [
    "capability.ts",
    "fixtures.ts",
    "identity.ts",
    "installed-runtime.ts",
    "lsp-probes.ts",
    "lsp.ts",
    "mapper-host.ts",
    "model.ts",
    "observer.ts",
    "process.ts",
    "run.ts",
    "source-admission.ts",
  ]);
  for (const value of drivers) {
    const row = record(value),
      name = string(row.name),
      bytes = fs.readFileSync(path.join(capsule, name));
    assert.equal(bytes.length, integer(row.bytes));
    assert.equal(hash(bytes), string(row.sha256), name);
  }
  const shared = array(manifest.sharedFiles);
  assert.deepEqual(shared.map((value) => string(record(value).relative)).sort(), [
    "../../json-value.ts",
    "../public-provider.ts",
  ]);
  for (const value of shared) {
    const row = record(value),
      relative = string(row.relative),
      bytes = fs.readFileSync(path.join(capsule, relative));
    assert.equal(bytes.length, integer(row.bytes));
    assert.equal(hash(bytes), string(row.sha256), relative);
  }
  return manifest;
}
function config(runtime: Runtime, directory: string, composables: string[]) {
  const document = {
    compilerOptions: {
      strict: true,
      noEmit: true,
      skipLibCheck: true,
      module: "ESNext",
      moduleResolution: "Bundler",
      target: "ES2024",
      lib: ["ES2024", "DOM"],
      types: [],
    },
    vueCompilerOptions: { composables: { useTemplateRef: composables } },
    include: ["*.vue", "*.ts"],
  };
  fs.writeFileSync(path.join(directory, "tsconfig.json"), JSON.stringify(document, null, 2));
  fs.writeFileSync(
    path.join(directory, "tsconfig.mapper.json"),
    JSON.stringify(
      { ...document, contentMappers: [{ package: "vize", extensions: [".vue"] }] },
      null,
      2,
    ),
  );
  fs.writeFileSync(
    path.join(directory, "vite.config.mjs"),
    "export default {vue:" +
      JSON.stringify({
        languageServer: { editor: true, hover: true, lint: false, typecheck: true },
        typeChecker: { corsaPath: runtime.backend, jsxTypecheck: false },
      }) +
      "};\n",
  );
}
export function project(
  runtime: Runtime,
  name: string,
  files: Record<string, string>,
  expected: Expected[],
  composables: string[] = [],
): Project {
  const directory = path.join(runtime.workspace, "type-admission-" + name);
  fs.mkdirSync(directory);
  config(runtime, directory, composables);
  for (const [file, text] of Object.entries(files))
    fs.writeFileSync(path.join(directory, file), text);
  write(path.join(runtime.output, name, "authored-inputs.txt"), { files, expected, directory });
  return { directory, files, expected };
}
export function baseProject(runtime: Runtime): Project {
  const valid = admissionFiles("valid");
  return project(runtime, "refs", valid.files, valid.expected, [
    "useTemplateRef",
    "templateRef",
    "r",
    "useTemplateRe\\u0066",
  ]);
}
export function globalProject(runtime: Runtime, index: number): Project {
  const row = globalRefAdmission[index];
  if (!row) throw new Error("missing original global case");
  const mutation = mutatedAdmission(row.source, row.mutation);
  const files = {
    "proofs.ts": typeProofs.replace(
      'declare global { const useTemplateRef: typeof import("vue").useTemplateRef; }',
      row.ambient ? "declare global { " + row.ambient + " }" : "",
    ),
    "Probe.vue": row.source,
  };
  return project(
    runtime,
    "global-" + row.name,
    files,
    [{ file: "Probe.vue", line: mutation.line, column: mutation.column, code: mutation.code }],
    row.vueComposable ? ["useTemplateRef"] : [],
  );
}

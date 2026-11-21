// Each target supplies the actual virtual script extension used by its checker.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const fromUi = createRequire(path.resolve("npm/ui/package.json"));
const ts = process.env.VIZE_TYPESCRIPT_PACKAGE
  ? createRequire(process.env.VIZE_TYPESCRIPT_PACKAGE)("./lib/typescript.js")
  : fromUi("typescript");
assert.equal(ts.version, "6.0.3");
const cases = JSON.parse(fs.readFileSync(0, "utf8"));
const documents = new Map(
  cases.map(({ id, extension, text }) => [
    path.resolve(`npm/ui/styled-checker-${id}.${extension}`),
    { id, extension, text },
  ]),
);
const options = {
  target: ts.ScriptTarget.ESNext,
  module: ts.ModuleKind.ESNext,
  moduleResolution: ts.ModuleResolutionKind.Bundler,
  moduleDetection: ts.ModuleDetectionKind.Force,
  strict: true,
  noEmit: true,
  types: [],
  allowJs: true,
  checkJs: true,
  skipLibCheck: true,
};
const host = ts.createCompilerHost(options);
const read = host.readFile.bind(host);
const exists = host.fileExists.bind(host);
host.readFile = (name) => documents.get(name)?.text ?? read(name);
host.fileExists = (name) => documents.has(name) || exists(name);
const globals = path.resolve("npm/cli/virtual-ts-globals.d.ts");
const program = ts.createProgram([...documents.keys(), globals], options, host);
const output = new Map([...documents.keys()].map((name) => [name, []]));
for (const [name, input] of documents) {
  const file = program.getSourceFile(name);
  assert.equal(file.scriptKind, input.extension === "mjs" ? ts.ScriptKind.JS : ts.ScriptKind.TS);
}
for (const d of ts.getPreEmitDiagnostics(program)) {
  const packet = output.get(d.file?.fileName);
  assert.ok(packet, "unexpected project or ambient diagnostic");
  assert.equal(typeof d.start, "number");
  assert.equal(typeof d.length, "number");
  packet.push({
    code: d.code,
    category: d.category,
    message: ts.flattenDiagnosticMessageText(d.messageText, "\n"),
    start: d.start,
    length: d.length,
  });
}
process.stdout.write(JSON.stringify([...output.values()]));

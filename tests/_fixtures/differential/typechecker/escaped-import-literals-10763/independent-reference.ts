import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { createHash } from "node:crypto";
import { pathToFileURL } from "node:url";
const typescriptModule = process.env.VIZE_IMPORT_GEOMETRY_REFERENCE_TYPESCRIPT;
assert(typescriptModule);
const { default: ts } = await import(pathToFileURL(typescriptModule).href);
assert.equal(ts.version, "5.9.3");
const fixture = path.resolve(
  "tests/_fixtures/differential/typechecker/escaped-import-literals-10763",
);
const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-import-geometry-reference-"));
const outputs = [];
for (const name of ["App.vue", "App.invalid.vue"]) {
  const code = fs.readFileSync(path.join(fixture, name + ".txt"), "utf8");
  const match = /<script setup lang="ts">([\s\S]*?)<\/script>/.exec(code);
  assert(match);
  const start = match.index + match[0].indexOf(match[1]);
  const script = " ".repeat(start) + match[1];
  const file = path.join(root, "src/App.ts");
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, script);
  const module = path.join(root, `src/日本語🍣 quoted'"directory/lib/greet/index.ts`);
  fs.mkdirSync(path.dirname(module), { recursive: true });
  fs.copyFileSync(path.join(fixture, "greet.ts.txt"), module);
  const config = JSON.parse(fs.readFileSync(path.join(fixture, "tsconfig.json.txt"), "utf8"));
  const parsed = ts.parseJsonConfigFileContent(
    config,
    ts.sys,
    root,
    undefined,
    path.join(root, "tsconfig.json"),
  );
  assert.equal(parsed.errors.length, 0);
  const program = ts.createProgram([file], { ...parsed.options, types: [] });
  const actual = ts.getPreEmitDiagnostics(program).map((d) => ({
    code: d.code,
    category: d.category,
    file: d.file?.fileName,
    start: d.start,
    length: d.length,
    message: ts.flattenDiagnosticMessageText(d.messageText, "\n"),
    position:
      d.file && d.start !== undefined ? d.file.getLineAndCharacterOfPosition(d.start) : null,
  }));
  if (name === "App.vue") assert.deepEqual(actual, []);
  else {
    assert.equal(actual.length, 1);
    assert.equal(actual[0].code, 2322);
    assert.equal(code.slice(actual[0].start, actual[0].start + actual[0].length), "invalid");
  }
  outputs.push({
    name,
    authoredSha256: createHash("sha256").update(code).digest("hex"),
    referenceScript: script,
    diagnostics: actual,
  });
}
const out = {
  schema: "vize.import-literal-geometry-independent-typescript-reference",
  version: 1,
  compiler: {
    version: ts.version,
    moduleSha256: createHash("sha256").update(fs.readFileSync(typescriptModule)).digest("hex"),
  },
  note: "Independent handwritten script extraction. This proves authored type semantics, not generated literal/navigation geometry. Raw Vize map source witness is mandatory separately.",
  root,
  outputs,
};
fs.writeFileSync(
  path.join(fixture, "independent-reference.json"),
  JSON.stringify(out, null, 2) + "\n",
);
console.log(
  JSON.stringify(
    outputs.map(({ referenceScript, ...x }) => x),
    null,
    2,
  ),
);

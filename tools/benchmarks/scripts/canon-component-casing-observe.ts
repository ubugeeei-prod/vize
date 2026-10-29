/** Complete native diagnostics and explicit public-SDK source-executable selection. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { isRecord, jsonValue } from "./canon-instance-directives-model.ts";
import { save, identify, run } from "./canon-instance-directives-io.ts";
export type NativeDiagnostic = {
  file: string;
  line: number;
  column: number;
  code: number;
  message: string;
};
export function nativeDiagnostics(
  stdout: Buffer,
  status: number | null,
  project: string,
): NativeDiagnostic[] {
  const text = new TextDecoder("utf8", { fatal: true }).decode(stdout);
  const result: NativeDiagnostic[] = [];
  for (const match of text.matchAll(/^(.+?)\((\d+),(\d+)\): error TS(\d+): (.*)$/gmu)) {
    const [, file, line, column, code, message] = match;
    assert.ok(file && line && column && code && message !== undefined);
    result.push({
      file: path.relative(project, path.resolve(project, file)).replaceAll("\\", "/"),
      line: +line,
      column: +column,
      code: +code,
      message,
    });
  }
  assert.equal(result.length, (text.match(/error TS\d+:/gu) ?? []).length);
  assert.equal(status, result.length ? 2 : 0);
  return result;
}
export function sourceSdk(project: string, provider: string, binary: string): void {
  const modules = path.join(project, "node_modules");
  fs.mkdirSync(modules);
  for (const name of fs.readdirSync(path.join(provider, "node_modules")))
    if (name !== "vize")
      fs.symlinkSync(path.join(provider, "node_modules", name), path.join(modules, name), "dir");
  const original = path.join(provider, "node_modules/vize"),
    target = path.join(modules, "vize");
  fs.mkdirSync(target);
  for (const name of fs.readdirSync(original))
    if (name !== "package.json") fs.symlinkSync(path.join(original, name), path.join(target, name));
  const manifest = jsonValue(fs.readFileSync(path.join(original, "package.json"), "utf8"));
  assert.ok(
    isRecord(manifest) &&
      isRecord(manifest.typescript) &&
      isRecord(manifest.typescript.contentMapper),
  );
  manifest.typescript.contentMapper.exec = [binary, "content-mapper"];
  save(path.join(target, "package.json"), manifest);
}
export function position(source: string, token: string, skip = 0) {
  const at = source.lastIndexOf(token);
  assert.ok(at >= 0);
  const lines = source.slice(0, at + skip).split(/\r\n|[\r\n\u2028\u2029]/u),
    tail = lines.at(-1);
  assert.ok(tail !== undefined);
  return { line: lines.length, column: tail.length + 1 };
}

export function stockFrameworkProof(provider: string, project: string, out: string) {
  const inputs = ["valid", "invalid", "repaired"].map((stage) => ({
    file: "stock/" + stage + ".ts",
    source:
      "import C from '../camel-kebab-valid/component';\nimport { h, type Component } from 'vue';\nconst component: Component=C;\nconst props:InstanceType<typeof C>['$props']={value:" +
      (stage === "invalid" ? "'bad'" : "1") +
      "};\nh(C,props);void component;\n",
  }));
  for (const input of inputs) {
    const file = path.join(project, input.file);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, input.source);
  }
  save(path.join(project, "tsconfig.stock.json"), {
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
    include: ["stock/*.ts", "**/component.ts"],
  });
  const compiler = path.join(provider, "node_modules/typescript/lib/tsc.js");
  const result = run(
    out,
    "stock-ts6",
    process.execPath,
    [compiler, "--pretty", "false", "--project", "tsconfig.stock.json"],
    project,
  );
  const diagnostics = nativeDiagnostics(result.stdout, result.status, project);
  const proof = { compiler: identify(compiler), inputs, diagnostics };
  save(path.join(out, "stock-summary.json"), proof);
  const invalid = inputs.find((input) => input.file === "stock/invalid.ts");
  assert.ok(invalid);
  assert.deepEqual(
    diagnostics.map((d) => ({ file: d.file, code: d.code, line: d.line, column: d.column })),
    [{ file: invalid.file, code: 2322, ...position(invalid.source, "value:") }],
  );
  return proof;
}

/** Public original small graphs; every imported source remains type checked. */
import assert from "node:assert/strict";
import path from "node:path";

export type Diagnostic = {
  file: string;
  line: number;
  column: number;
  code: number;
  message: string;
};

export const longModule =
  "dependency-with-a-deliberately-long-module-specifier-for-allocation-census";
const fromKeyword = ["fr", "om"].join("");

export function censusFixture(sfcs: number, invalid: boolean) {
  assert.ok([1, 10, 100].includes(sfcs));
  const files: Record<string, string> = {
    "package.json": JSON.stringify({ name: "vts-source-census", private: true, type: "module" }),
    "tsconfig.json": JSON.stringify({
      compilerOptions: {
        strict: true,
        noEmit: true,
        incremental: false,
        target: "ES2022",
        module: "ESNext",
        moduleResolution: "Bundler",
        skipLibCheck: true,
        jsx: "preserve",
        lib: ["ES2022", "DOM"],
        types: [],
      },
      files: [
        "env.d.ts",
        ...Array.from({ length: sfcs }, (_, i) => "src/App" + String(i).padStart(3, "0") + ".vue"),
      ],
    }),
    "env.d.ts":
      'declare function require(path: "./graph/required"): typeof import("./src/graph/required");\ndeclare namespace JSX { interface Element { readonly nodeType: "census" } interface IntrinsicElements { span: { title?: string } } }\n',
    "src/graph/types.ts": "export interface Model { id: string; count: number }\n",
    "src/graph/barrel.ts":
      "export { value } " +
      fromKeyword +
      ' "./' +
      longModule +
      '";\nexport type { Model } ' +
      fromKeyword +
      ' "./types";\n',
    ["src/graph/" + longModule + ".ts"]: "export const value: number = 1;\n",
    "src/graph/required.ts": "export const value: number = 1;\n",
    "src/graph/jsx-widget.tsx": 'export const node = <span title="census" />;\n',
  };
  const expected: Diagnostic[] = [];
  const diagnostic = (file: string, token: string, message: string) => {
    const source = files[file],
      offset = source.indexOf(token);
    assert.ok(offset >= 0);
    assert.equal(source.indexOf(token, offset + 1), -1);
    const prefix = source.slice(0, offset),
      line = prefix.split("\n").length,
      column = prefix.length - (prefix.lastIndexOf("\n") + 1) + 1;
    expected.push({ file, line, column, code: 2322, message });
  };
  for (let i = 0; i < sfcs; i++) {
    const file = "src/App" + String(i).padStart(3, "0") + ".vue";
    const lines = [
      '<script setup lang="ts">',
      'import { value } from "./graph/barrel";',
      'import type { Model } from "./graph/types";',
      'import { node } from "./graph/jsx-widget";',
      'const required = require("./graph/required");',
      'const model: Model = { id: "entry' +
        String(i).padStart(3, "0") +
        '", count: value + required.value };',
      'const prefix = "日本😀"; const text: string = model.' + (invalid ? "count" : "id") + ";",
      "void node;",
      "</script>",
      "<template><span>{{ prefix }} {{ text }}</span></template>",
      "",
    ];
    files[file] = lines.join(i % 2 === 0 ? "\r\n" : "\n");
    if (invalid)
      diagnostic(file, "text: string", "Type 'number' is not assignable to type 'string'.");
  }
  if (invalid)
    for (const [i, file] of [
      "src/graph/types.ts",
      "src/graph/barrel.ts",
      "src/graph/" + longModule + ".ts",
      "src/graph/required.ts",
      "src/graph/jsx-widget.tsx",
    ].entries()) {
      const token = "dependencyCoverage" + i;
      files[file] += "export const " + token + ': number = "invalid";\n';
      diagnostic(file, token, "Type 'string' is not assignable to type 'number'.");
    }
  return { files, expected };
}

export function packetDiagnostics(packet: unknown, root: string): Diagnostic[] {
  assert.ok(packet && typeof packet === "object");
  const value = packet as {
    errorCount: number;
    warningCount: number;
    fileCount: number;
    files: { file: string; diagnostics: string[] }[];
  };
  assert.ok(Array.isArray(value.files));
  assert.equal(value.warningCount, 0);
  assert.equal(value.fileCount, value.files.length);
  const names = value.files.map((row) => path.resolve(root, row.file));
  assert.equal(new Set(names).size, names.length, "file inventory cannot contain duplicates");
  const actual = value.files.flatMap((row) =>
    row.diagnostics.map((text) => {
      const match = /^error:(\d+):(\d+) \[TS(\d+)\] (.*)$/u.exec(text);
      assert.ok(match, "retain and reject any unclassified diagnostic");
      const file = (
        path.isAbsolute(row.file) ? path.relative(root, row.file) : row.file
      ).replaceAll("\\", "/");
      return {
        file,
        line: +match[1],
        column: +match[2],
        code: +match[3],
        message: match[4].replace(/ \(source: .*\)$/u, ""),
      };
    }),
  );
  assert.equal(value.errorCount, actual.length);
  return actual;
}

export function assertPacket(actual: Diagnostic[], expected: Diagnostic[]) {
  assert.equal(actual.length, expected.length, "diagnostic multiplicity is mandatory");
  const sort = (rows: Diagnostic[]) => rows.map((row) => JSON.stringify(row)).toSorted();
  assert.deepEqual(sort(actual), sort(expected));
}

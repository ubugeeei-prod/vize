import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

interface Message {
  ruleId: string | null;
  severity: number;
  message: string;
  line: number;
  column: number;
  endLine?: number;
  endColumn?: number;
  nodeType?: string;
  messageId?: string;
}
interface Packet {
  filename: string;
  error_count: number;
  warning_count: number;
  diagnostics: unknown[];
}
interface Case {
  id: string;
  filename: string;
  source: string;
  reference: Message[];
  expectedNative: Packet;
}
interface Original extends Case {
  path: string;
  sourceSha256: string;
}
interface Successor extends Case {
  historicalNative: Packet;
}
interface MatrixFinding {
  file: string;
  ruleId: string;
  severity: string;
  message: string;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
}
interface Corpus {
  schema: string;
  version: number;
  versions: Record<string, string>;
  historical: { path: string; sha256: string };
  originals: Original[];
  successors: Successor[];
  controls: Case[];
  originalMatrix: { falsePositives: MatrixFinding[]; falseNegatives: MatrixFinding[] };
}
interface EslintApi {
  Linter: new () => {
    verify(source: string, config: unknown[], options: { filename: string }): Message[];
  };
}
const root = fileURLToPath(new URL("../../", import.meta.url));
const fixtureRoot = path.join(
  root,
  "tests/_fixtures/differential/linter/template-script-shadow-10822",
);
const corpus = JSON.parse(readFileSync(path.join(fixtureRoot, "cases.json"), "utf8")) as Corpus;
const oracle = createRequire(
  path.join(
    process.env.VIZE_ESLINT_ORACLE_ROOT ?? path.join(root, "tools/benchmarks/scripts"),
    "package.json",
  ),
);
const { Linter } = oracle("eslint") as EslintApi;
const plugin: unknown = oracle("eslint-plugin-vue");
const parser: unknown = oracle("vue-eslint-parser");
const tsParser: unknown = oracle("@typescript-eslint/parser");
const digest = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");

test("original n8n sources and zero-budget findings retain their exact immutable inputs", () => {
  assert.equal(corpus.schema, "vize.template-script-shadow-correctness");
  assert.equal(corpus.version, 1);
  assert.deepEqual(
    corpus.originals.map((entry) => entry.reference.length),
    [4, 2],
  );
  assert.deepEqual(corpus.originalMatrix.falsePositives, []);
  const current = corpus.originals.flatMap((entry) => {
    assert.equal(readFileSync(path.join(fixtureRoot, entry.path), "utf8"), entry.source);
    assert.equal(digest(entry.source), entry.sourceSha256);
    return entry.reference.map((message) => ({
      file: entry.filename,
      ruleId: message.ruleId,
      severity: "warning",
      message: message.message,
      line: message.line,
      column: message.column,
      endLine: message.endLine,
      endColumn: message.endColumn,
    }));
  });
  assert.deepEqual(
    current,
    corpus.originalMatrix.falseNegatives.map((entry) => ({
      file: entry.file,
      ruleId: entry.ruleId,
      severity: entry.severity,
      message: entry.message,
      line: entry.line,
      column: entry.column,
      endLine: entry.endLine,
      endColumn: entry.endColumn,
    })),
  );
});

test("the three script-exposure successors authenticate unchanged historical zero packets", () => {
  const bytes = readFileSync(path.join(root, corpus.historical.path));
  assert.equal(digest(bytes), corpus.historical.sha256);
  const historical = JSON.parse(bytes.toString("utf8")) as {
    cases: Array<Pick<Case, "id" | "source" | "expectedNative">>;
  };
  assert.deepEqual(
    corpus.successors.map((entry) => entry.id),
    ["script-setup-for", "script-setup-slot", "props-slot"],
  );
  for (const current of corpus.successors) {
    const before = historical.cases.find((entry) => entry.id === current.id);
    assert(before);
    assert.equal(before.source, current.source);
    assert.deepEqual(before.expectedNative, current.historicalNative);
    assert.equal(current.historicalNative.warning_count, 0);
    assert.deepEqual(current.historicalNative.diagnostics, []);
    assert.equal(current.expectedNative.warning_count, 1);
    assert.equal(current.expectedNative.diagnostics.length, 1);
  }
});

test("the pinned real Vue/TypeScript oracle reproduces every full source and control answer", () => {
  const versions: Record<string, string> = Object.fromEntries(
    ["eslint", "eslint-plugin-vue", "vue-eslint-parser", "@typescript-eslint/parser"].map(
      (name): [string, string] => [
        name,
        (oracle(`${name}/package.json`) as { version: string }).version,
      ],
    ),
  );
  assert.deepEqual(versions, corpus.versions);
  assert.deepEqual(versions, {
    eslint: "10.4.1",
    "eslint-plugin-vue": "10.9.2",
    "vue-eslint-parser": "10.4.1",
    "@typescript-eslint/parser": "8.65.0",
  });
  const config: unknown[] = [
    {
      files: ["**/*.vue"],
      languageOptions: {
        parser,
        parserOptions: { parser: tsParser, ecmaVersion: "latest", sourceType: "module" },
      },
      plugins: { vue: plugin },
      rules: { "vue/no-template-shadow": "warn" },
    },
  ];
  for (const entry of [...corpus.originals, ...corpus.successors, ...corpus.controls]) {
    const actual = new Linter().verify(entry.source, config, { filename: entry.filename });
    assert.deepEqual(actual, entry.reference, entry.id);
  }
});

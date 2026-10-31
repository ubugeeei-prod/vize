/** One reviewed factory migration; source identities and raw packets stay separate. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import type ts from "typescript";
import { coldScalePacketView } from "./cold-typechecker-scale-packet.ts";
import { readRefFactoryAuthority } from "./ref-factory-authority.ts";
import { factoryCodeEdits } from "./ref-factory-parity.ts";

type CliFile = Record<string, unknown> & { file: string; virtualTs?: string | null };
type CliPacket = Record<string, unknown> & { files: CliFile[] };
export type RefFactoryCorpus = {
  root: string;
  app: string;
  entries: { path: string; bytes: number; sha256: string }[];
};
type FactoryEdit = (
  before: string,
  after: string,
  oldFactory: string,
  importDelta: number,
) => { post: unknown[] };
const asset = "crates/vize_canon/src/virtual_ts/helpers/declarations/ref-instance-factory.d.ts.txt";
const identities = [
  [
    "tools/benchmarks/scripts/ref-factory-authority.ts",
    "d85affc15b7d7b5c8298bb5a4b2da9494ec72164417d9c9824be93b12305703e",
  ],
  [
    "tools/benchmarks/scripts/ref-factory-parity.ts",
    "7c8cf76a9b51201fd4936672c6a83359cb6c35cd99254d454a68b200ae4b85d8",
  ],
] as const;
const publicFiles = [
  "virtual-ts.d.ts",
  "virtual-ts.d.mts",
  "virtual-ts.d.cts",
  "virtual-ts-globals.d.ts",
  "virtual-ts-patterns.d.ts",
  "content-mapper-helpers.d.ts",
  "content-mapper-helpers.d.mts",
  "content-mapper-helpers.d.cts",
  "generated/content-mapper-types.d.ts",
  "virtual-ts-slots.d.ts",
];
const hash = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
function packet(value: unknown): CliPacket {
  assert(value && typeof value === "object" && !Array.isArray(value));
  const result = value as CliPacket;
  assert(Array.isArray(result.files));
  for (const file of result.files) {
    assert(file && typeof file === "object" && typeof file.file === "string");
    assert(file.virtualTs == null || typeof file.virtualTs === "string");
  }
  return result;
}

function moduleSpecifiers(typescript: typeof ts, text: string, file: string) {
  const source = typescript.createSourceFile(
    file,
    text,
    typescript.ScriptTarget.Latest,
    true,
    typescript.ScriptKind.TS,
  );
  assert.equal(
    (source as ts.SourceFile & { parseDiagnostics: readonly ts.Diagnostic[] }).parseDiagnostics
      .length,
    0,
  );
  const rows: { kind: string; specifier: string; start: number; end: number }[] = [];
  const add = (node: ts.Node, literal: ts.Node | undefined) => {
    assert(literal && typescript.isStringLiteralLike(literal));
    rows.push({
      kind: typescript.SyntaxKind[node.kind],
      specifier: literal.text,
      start: node.getStart(source),
      end: node.end,
    });
  };
  const visit = (node: ts.Node) => {
    if (typescript.isImportDeclaration(node)) add(node, node.moduleSpecifier);
    else if (typescript.isExportDeclaration(node) && node.moduleSpecifier)
      add(node, node.moduleSpecifier);
    else if (typescript.isImportTypeNode(node)) {
      assert(typescript.isLiteralTypeNode(node.argument));
      add(node, node.argument.literal);
    } else if (
      typescript.isImportEqualsDeclaration(node) &&
      typescript.isExternalModuleReference(node.moduleReference)
    )
      add(node, node.moduleReference.expression);
    else if (
      typescript.isCallExpression(node) &&
      (node.expression.kind === typescript.SyntaxKind.ImportKeyword ||
        (typescript.isIdentifier(node.expression) && node.expression.text === "require"))
    ) {
      assert.equal(node.arguments.length, 1);
      add(node, node.arguments[0]);
    }
    typescript.forEachChild(node, visit);
  };
  visit(source);
  assert(
    rows.every(
      (row) =>
        row.specifier !== "vize" &&
        !row.specifier.startsWith("vize/") &&
        !row.specifier.startsWith("@vizejs/") &&
        !row.specifier.includes("/node_modules/vize/"),
    ),
    "a provider SDK import is outside this canonical-helper comparison",
  );
  return rows;
}

export function compareRefFactoryResults(
  beforeValue: unknown,
  afterValue: unknown,
  options: {
    corpus: RefFactoryCorpus;
    oldFactory: string;
    edit: FactoryEdit;
    typescript: typeof ts;
  },
) {
  const before = packet(beforeValue),
    after = packet(afterValue),
    expected = structuredClone(before);
  coldScalePacketView(before, options.corpus);
  coldScalePacketView(after, options.corpus);
  const authoredInputs = new Map(
    options.corpus.entries.map((entry) => [path.join(options.corpus.root, entry.path), entry]),
  );
  assert.equal(after.files.length, before.files.length);
  const seen = new Set<string>(),
    changed: { file: string; sourceSha256: string; importDelta: number; edits: number }[] = [];
  before.files.forEach((file, index) => {
    assert(!seen.has(file.file));
    seen.add(file.file);
    assert.equal(after.files[index].file, file.file);
    if (typeof file.virtualTs !== "string") return;
    assert(typeof after.files[index].virtualTs === "string");
    const references = moduleSpecifiers(options.typescript, file.virtualTs, file.file);
    moduleSpecifiers(options.typescript, after.files[index].virtualTs, file.file);
    const factoryAt = file.virtualTs.indexOf(options.oldFactory);
    let importDelta = 0;
    if (factoryAt >= 0) {
      const original = path.resolve(options.corpus.app, file.file);
      assert(original.endsWith(".vue"));
      const source = fs.readFileSync(original, "utf8");
      const identity = authoredInputs.get(original);
      assert(identity);
      assert.equal(Buffer.byteLength(source), identity.bytes);
      assert.equal(hash(source), identity.sha256);
      const scripts = [...source.matchAll(/<script setup lang="ts">([\s\S]*?)<\/script>/gu)];
      assert.equal(
        scripts.length,
        1,
        "only the existing generated cold-corpus script shape is admitted",
      );
      const authoredAll = moduleSpecifiers(options.typescript, scripts[0][1], original);
      const authored = authoredAll.filter((row) => row.specifier.endsWith(".vue"));
      const rewritten = references.filter(
        (row) => row.start < factoryAt && row.specifier.endsWith(".vue.ts"),
      );
      assert.equal(authored.length, 1);
      assert.equal(rewritten.length, 1);
      assert.equal(rewritten[0].specifier, authored[0].specifier + ".ts");
      assert.deepEqual(
        references
          .filter((row) => row.start < factoryAt)
          .map(({ kind, specifier }) => ({ kind, specifier })),
        authoredAll.map(({ kind, specifier }) => ({
          kind,
          specifier: specifier.endsWith(".vue") ? specifier + ".ts" : specifier,
        })),
        "every original module kind/literal/order must survive exactly; only the one Vue suffix changes",
      );
      assert(
        !references.some(
          (row) =>
            row.start < factoryAt &&
            row.specifier.endsWith(".ts") &&
            !row.specifier.endsWith(".vue.ts"),
        ),
        "unknown pre-factory extension rewrite",
      );
      importDelta = 3;
      changed.push({ file: file.file, sourceSha256: hash(source), importDelta, edits: 0 });
    }
    const edits = options.edit(
      file.virtualTs,
      after.files[index].virtualTs,
      options.oldFactory,
      importDelta,
    );
    if (factoryAt >= 0) changed.at(-1)!.edits = edits.post.length;
    expected.files[index].virtualTs = after.files[index].virtualTs;
  });
  assert.deepEqual(
    after,
    expected,
    "whole result admits only the checked factory/code-comment edits",
  );
  return { completeFiles: before.files.length, activeFiles: changed.length, changed };
}

export async function refFactoryComparison(baseRoot: string, headRoot: string, provider: string) {
  if (!fs.existsSync(path.join(headRoot, asset)) || fs.existsSync(path.join(baseRoot, asset)))
    return null;
  for (const [file, digest] of identities) {
    assert.equal(hash(fs.readFileSync(path.join(headRoot, file))), digest);
    assert.equal(
      hash(fs.readFileSync(new URL("./" + path.basename(file), import.meta.url))),
      digest,
    );
  }
  const io = "tools/benchmarks/scripts/canon-instance-directives-io.ts";
  assert.equal(
    hash(fs.readFileSync(path.join(headRoot, io))),
    hash(fs.readFileSync(new URL("./canon-instance-directives-io.ts", import.meta.url))),
  );
  const authority = readRefFactoryAuthority(headRoot);
  const nativeLaunchSources = [
    "crates/vize_carton/src/corsa_api_mode.rs",
    "crates/vize_carton/src/corsa_resolver.rs",
    "crates/vize_canon/src/batch/executor/cli.rs",
    "crates/vize_canon/src/batch/executor/option_probe.rs",
  ].map((file) => {
    const base = fs.readFileSync(path.join(baseRoot, file)),
      head = fs.readFileSync(path.join(headRoot, file));
    assert.deepEqual(
      head,
      base,
      "native launch/classification/option-probe policy must be unchanged between roles",
    );
    return { file, bytes: head.length, sha256: hash(head) };
  });
  assert.equal(
    fs.readFileSync(path.join(baseRoot, authority.source.path), "utf8"),
    authority.source.beforeWholeUtf8,
  );
  const publicClosure = publicFiles.map((file) => {
    const base = fs.readFileSync(path.join(baseRoot, "npm/cli", file)),
      head = fs.readFileSync(path.join(headRoot, "npm/cli", file));
    assert.deepEqual(
      head,
      base,
      "both sources must carry the same current ten-file public package",
    );
    return { file, bytes: head.length, sha256: hash(head) };
  });
  assert.deepEqual(
    fs.readFileSync(path.join(headRoot, "npm/cli/package.json")),
    fs.readFileSync(path.join(baseRoot, "npm/cli/package.json")),
  );
  const require = createRequire(path.join(provider, "package.json")),
    typescript = require("typescript") as typeof ts;
  assert.equal(typescript.version, "6.0.3");
  const compare = (before: unknown, after: unknown, corpus: RefFactoryCorpus) => {
    const result = compareRefFactoryResults(before, after, {
      corpus,
      oldFactory: "  " + authority.declaration.wholeUtf8,
      edit: factoryCodeEdits,
      typescript,
    });
    assert.equal(
      result.activeFiles,
      622,
      "the existing sealed cold-corpus demand population must be retained",
    );
    return result;
  };
  return {
    compare,
    publicClosure,
    nativeLaunchSources,
    helperIdentities: identities,
    canonicalPreamble: authority.preamble,
    sourceScope:
      "explicit base/head canonical-helper images; source public10 identity only, fresh consumer acceptance is external; native loaded files unobserved",
  };
}

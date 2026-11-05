import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import ts from "typescript";
import {
  assertCompilerPatchSuccessor,
  compilerPatchSuccessor,
  type CompilerPatchCase,
} from "./compiler-patch-successors.ts";
import type { CompilerOutput } from "./realworld-build-errors.ts";
import {
  assertRouterImportedPropsSuccessor,
  importedRouterProps,
  routerImportedPropsSuccessor,
} from "./vue-router-imported-props-successor.ts";

export type AuthoredPieceCompilerCase = CompilerPatchCase | "create-vue-formatted";
type OriginalPins = Parameters<typeof assertCompilerPatchSuccessor>[2];
type Review = {
  id: AuthoredPieceCompilerCase;
  sourceSha256: string;
  previousPacketSha256: string;
  previousCodeSha256: string;
  currentPacketSha256: string;
  currentCodeSha256: string;
};
const directory = new URL(
  "../_fixtures/compiler-authored-piece-successors-11698/",
  import.meta.url,
);
const capture = JSON.parse(fs.readFileSync(new URL("capture.json", directory), "utf8")) as {
  rows: Review[];
};
const hash = (value: string) => createHash("sha256").update(value).digest("hex");
const serialize = (value: CompilerOutput) => JSON.stringify(value, null, 2);

// #11698 erases authored TypeScript pieces before assembly. The emitter's
// quotes, parentheses, patch comments and layout now survive whole-module
// output. The exact packet checks below remain separate from this proof that
// the reviewed transition preserves every JavaScript syntax node and value.
function syntax(node: ts.Node): unknown {
  if (ts.isParenthesizedExpression(node)) return syntax(node.expression);
  const children: unknown[] = [];
  ts.forEachChild(node, (child) => {
    children.push(syntax(child));
  });
  return [
    node.kind,
    ts.isIdentifier(node) || ts.isPrivateIdentifier(node) || ts.isLiteralExpression(node)
      ? node.text
      : null,
    ts.isVariableDeclarationList(node) ? node.flags : null,
    ts.isPrefixUnaryExpression(node) || ts.isPostfixUnaryExpression(node) ? node.operator : null,
    children,
  ];
}

function previousOutput(id: AuthoredPieceCompilerCase): CompilerOutput {
  if (id === "vue-router-clean" || id === "vue-router-broken") {
    return routerImportedPropsSuccessor(id).output;
  }
  const previous = compilerPatchSuccessor(
    id === "create-vue-formatted" ? "create-vue-clean" : id,
  ).current;
  if (id !== "create-vue-formatted") return previous;
  // Formatting sorts exactly these two attributes; the original formatted
  // code pin (17e394dd...) is checked unchanged by its caller and capture row.
  const before = '              target: "_blank",\n              rel: "noopener"';
  const after = '              rel: "noopener",\n              target: "_blank"';
  assert.equal(previous.code.split(before).length, 2);
  const code = previous.code.replace(before, after);
  assert.equal(code.split(after).length, 2);
  assert.equal(code.replace(after, before), previous.code);
  return { ...previous, code };
}

export function verifyAuthoredPieceCompilerPair(
  id: AuthoredPieceCompilerCase,
  previous: CompilerOutput,
  current: CompilerOutput,
): void {
  const review = capture.rows.find((row) => row.id === id);
  assert.ok(review, `unregistered authored-piece compiler successor ${id}`);
  assert.equal(hash(serialize(previous)), review.previousPacketSha256);
  assert.equal(hash(previous.code), review.previousCodeSha256);
  assert.equal(hash(serialize(current)), review.currentPacketSha256);
  assert.equal(hash(current.code), review.currentCodeSha256);
  assert.deepEqual(current, { ...previous, code: current.code }, "exact full forward packet");
  assert.deepEqual(previous, { ...current, code: previous.code }, "exact full inverse packet");
  assert.deepEqual(
    syntax(ts.createSourceFile("previous.js", previous.code, ts.ScriptTarget.Latest, true)),
    syntax(ts.createSourceFile("current.js", current.code, ts.ScriptTarget.Latest, true)),
    `${id}: only reviewed printing changes; every syntax node and value stays equal`,
  );
}

export function authoredPieceCompilerSuccessor(id: AuthoredPieceCompilerCase) {
  const previous = previousOutput(id);
  const raw = fs.readFileSync(new URL(`${id}.json`, directory), "utf8");
  const output = JSON.parse(raw) as CompilerOutput;
  assert.equal(raw, serialize(output), "retain every original serialization byte");
  verifyAuthoredPieceCompilerPair(id, previous, output);
  return { previous, output, raw };
}

export function assertAuthoredPieceCompilerSuccessor(
  id: AuthoredPieceCompilerCase,
  actualRaw: string,
  original: OriginalPins,
): void {
  const expected = authoredPieceCompilerSuccessor(id);
  if (id === "vue-router-clean" || id === "vue-router-broken") {
    assert.ok(original.packetSha256);
    assert.ok(original.codeSha256);
    assertRouterImportedPropsSuccessor(id, serialize(expected.previous), {
      packetSha256: original.packetSha256,
      codeSha256: original.codeSha256,
    });
  } else if (id === "create-vue-formatted") {
    assert.equal(hash(expected.previous.code), original.codeSha256);
  } else {
    assertCompilerPatchSuccessor(id, compilerPatchSuccessor(id).currentRaw, original);
  }
  assert.equal(actualRaw, expected.raw, `${id}: exact whole authored-piece compiler packet`);
}

export const authoredPieceRouterProps = importedRouterProps.replaceAll(
  /    (\w+): \{\n      type: ([^,\n]+),\n      required: (true|false)\n    }/g,
  "    $1: { type: $2, required: $3 }",
);

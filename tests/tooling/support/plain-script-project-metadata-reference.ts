import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import path from "node:path";

type NativePacket = { code: string; [key: string]: unknown };
export type PlainScriptPacket = {
  case: string;
  filename: string;
  source: string;
  sourceSha256: string;
  options: {
    filename: string;
    inlineTemplate: boolean;
    sourceMap: boolean;
    isProd: boolean;
  };
  native: NativePacket;
  official: unknown;
};

const constantControlsSource = "56ae19e666b9b8bf4ad88971d29d4c17c660506072a6f86f92a6b6d6464a41d0";

type ComparableNode = { type?: unknown; [key: string]: unknown };

function isReadyLiteralDeclaration(value: unknown): boolean {
  if (!value || typeof value !== "object") return false;
  const node = value as ComparableNode;
  if (node.type !== "VariableDeclaration" || node.kind !== "const") return false;
  const declarations = node.declarations;
  if (!Array.isArray(declarations) || declarations.length !== 1) return false;
  const declaration = declarations[0] as ComparableNode;
  return (
    declaration.type === "VariableDeclarator" &&
    (declaration.id as ComparableNode | undefined)?.type === "Identifier" &&
    (declaration.id as ComparableNode).name === "ready" &&
    (declaration.init as ComparableNode | undefined)?.type === "BooleanLiteral" &&
    (declaration.init as ComparableNode).value === true
  );
}

function setupStatements(exportDefault: ComparableNode): unknown[] {
  const declaration = exportDefault.declaration as ComparableNode | undefined;
  assert.equal(declaration?.type, "ObjectExpression");
  const properties = declaration.properties;
  assert.ok(Array.isArray(properties));
  const setups = properties.filter(
    (property) =>
      (property as ComparableNode).type === "ObjectMethod" &&
      ((property as ComparableNode).key as ComparableNode | undefined)?.type === "Identifier" &&
      ((property as ComparableNode).key as ComparableNode).name === "setup",
  ) as ComparableNode[];
  assert.equal(setups.length, 1);
  const body = setups[0].body as ComparableNode | undefined;
  assert.equal(body?.type, "BlockStatement");
  assert.ok(Array.isArray(body.body));
  return body.body;
}

/**
 * Normal script presence disables setup-literal hoisting. Admit exactly the
 * one `ready = true` declaration moving into setup; every other program node
 * remains owned by the preserved packet comparison.
 */
export function assertSetupLiteralMovesIntoSetup(actual: unknown, preserved: unknown): void {
  const normalized = structuredClone(actual) as ComparableNode;
  assert.equal(normalized.type, "Program");
  assert.ok(Array.isArray(normalized.body));
  assert.equal(normalized.body.filter(isReadyLiteralDeclaration).length, 0);
  const exports = normalized.body.filter(
    (statement) => (statement as ComparableNode).type === "ExportDefaultDeclaration",
  ) as ComparableNode[];
  assert.equal(exports.length, 1);
  const statements = setupStatements(exports[0]);
  const ready = statements.filter(isReadyLiteralDeclaration);
  assert.equal(ready.length, 1);
  statements.splice(statements.indexOf(ready[0]), 1);
  normalized.body.splice(normalized.body.indexOf(exports[0]), 0, ready[0]);
  assert.deepEqual(normalized, preserved, "only the setup literal placement advances");
}

/**
 * #11865 adds project-watch metadata to the four preserved constant-controls
 * App packets. Specify that addition independently of the candidate packet;
 * code, maps, diagnostics and every other field remain owned by the existing
 * bracket/erasure reference and the final whole-packet comparison.
 */
export function plainScriptProjectMetadataReference<T extends NativePacket>(
  row: PlainScriptPacket,
  preserved: T,
  workspaceRoot: string,
): T {
  const out = structuredClone(preserved);
  if (row.case !== "constant-controls" || row.filename !== "src/App.vue") return out;

  assert.equal(row.sourceSha256, constantControlsSource);
  assert.equal(createHash("sha256").update(row.source).digest("hex"), constantControlsSource);
  assert.equal(typeof row.options.inlineTemplate, "boolean");
  assert.equal(typeof row.options.sourceMap, "boolean");
  assert.deepEqual(row.options, {
    filename: "src/App.vue",
    inlineTemplate: row.options.inlineTemplate,
    sourceMap: row.options.sourceMap,
    isProd: row.options.inlineTemplate,
  });
  assert.equal(Object.hasOwn(preserved, "dependencies"), false);
  assert.equal(Object.hasOwn(preserved, "typeProject"), false);
  assert.equal(path.posix.resolve(workspaceRoot), workspaceRoot);

  return {
    ...out,
    dependencies: [
      path.posix.join(workspaceRoot, "src", "tsconfig.json"),
      path.posix.join(workspaceRoot, "tsconfig.json"),
    ],
    typeProject: { root: path.posix.join(workspaceRoot, "tsconfig.json"), revision: "1" },
  };
}

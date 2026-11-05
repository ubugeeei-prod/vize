import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import type * as TypeScript from "typescript";

// The CLI owns this declared test parser; keep it out of ambient project discovery.
const ts = createRequire(new URL("../../npm/cli/package.json", import.meta.url))(
  "typescript",
) as typeof TypeScript;

// These three archived references use a formatted module representation. Keep
// those bytes and bind their raw native-emitter successors independently.
// Candidate output never passes through this reader or the AST comparison.
const successors = new Map([
  [
    "compiler/project/imported-props-path-alias-11794",
    {
      inputs: "f3faa55206b067cc26598f1729586856f0daef3a470f98f6ed2af494c6d7914a",
      before: "899e4a902414b55468b9717c3df5ce3a13a0e4ffd8b5b3443dc5e800d4a42c4d",
      after: "36cdf10668f70c3e7119001f7791836979a62adbb8b4ab2bbd65da80ea54336f",
      file: "imported-props-path-alias-11794/module.raw.expected.txt",
    },
  ],
  [
    "compiler/sfc/nested-dynamic-vfor-11728",
    {
      inputs: "98dfe1e7089c06012fcfcfe53a376ff3bdeb8dc8d1a963570deef103ff1f99eb",
      before: "ba96f9acc62d4201bb1079d6216e6330446ebc7ce345fb236487151e61d81378",
      after: "ba5145fcce6fc6ceb25128cd24c17fadb7f34023c5eb67e4af2e1a0c67629862",
      file: "nested-dynamic-vfor-11728/module.raw.expected.txt",
    },
  ],
  [
    "compiler/sfc/recovered-attribute-boundary-11731",
    {
      inputs: "962ab0295e669feac1946ea35e55391a5ef1b10204f22c50080ddc065e29775e",
      before: "670e14cdf69b3d75b3a21e770878c1c3d939f7b59f4d2971237498ffdfebe150",
      after: "4abcd3025b7d8e4064a0475f77630bba8308969c8d0c4e8e272b5693996e0787",
      file: "recovered-attribute-boundary-11731/module.raw.expected.txt",
    },
  ],
]);

type Shape = {
  kind: number;
  flags: number;
  fields: Record<string, string | boolean | number>;
  children: Shape[];
};

const presentation = new Set([
  "pos",
  "end",
  "id",
  "kind",
  "flags",
  "transformFlags",
  "modifierFlagsCache",
  "singleQuote",
  "multiLine",
]);

function shape(node: TypeScript.Node): Shape {
  if (ts.isSourceFile(node)) {
    return { kind: node.kind, flags: node.flags, fields: {}, children: node.statements.map(shape) };
  }
  if (
    ts.isShorthandPropertyAssignment(node) &&
    !node.objectAssignmentInitializer &&
    node.name.text !== "__proto__"
  ) {
    // `{ value }` and `{ value: value }` retain the same binding and property.
    // `__proto__` is deliberately excluded: its long form changes prototypes.
    return {
      kind: ts.SyntaxKind.PropertyAssignment,
      flags: node.flags,
      fields: {},
      children: [shape(node.name), shape(node.name)],
    };
  }
  const fields: Shape["fields"] = {};
  for (const [key, value] of Object.entries(node)) {
    if (
      !presentation.has(key) &&
      (typeof value === "string" || typeof value === "boolean" || typeof value === "number")
    ) {
      fields[key] = value;
    }
  }
  const children: Shape[] = [];
  ts.forEachChild(node, (child) => {
    children.push(shape(child));
  });
  return { kind: node.kind, flags: node.flags, fields, children };
}

function moduleShape(source: string): Shape {
  const syntax = ts.transpileModule(source, {
    fileName: "module.ts",
    reportDiagnostics: true,
    compilerOptions: { target: ts.ScriptTarget.Latest, module: ts.ModuleKind.ESNext },
  });
  assert.ok(Array.isArray(syntax.diagnostics), "independent parser must return its diagnostics");
  assert.deepEqual(syntax.diagnostics, [], "the complete expected module must parse");
  const parsed = ts.createSourceFile(
    "module.ts",
    source,
    ts.ScriptTarget.Latest,
    false,
    ts.ScriptKind.TS,
  );
  return shape(parsed);
}

export function assertEquivalentCompilerModule(archived: string, current: string): void {
  assert.deepEqual(
    moduleShape(current),
    moduleShape(archived),
    "complete independent module AST equivalence",
  );
}

const digest = (value: string | Uint8Array): string =>
  createHash("sha256").update(value).digest("hex");

export function currentCompilerModuleReference(
  root: string,
  fixture: {
    readonly id: string;
    readonly inputs: unknown;
    readonly reference: { readonly sha256: string };
  },
  archived: Buffer,
): Buffer {
  const next = successors.get(fixture.id);
  if (!next) return archived;
  // The generic project loader also accepts independent manifests whose case
  // IDs match a registered example. A transition belongs to the exact archived
  // reference, not to every fixture that happens to reuse its human-readable ID.
  assert.equal(digest(archived), fixture.reference.sha256, "whole declared archived reference");
  if (fixture.reference.sha256 !== next.before) return archived;
  assert.equal(
    digest(JSON.stringify(fixture.inputs)),
    next.inputs,
    "all original compiler input descriptors",
  );
  assert.equal(fixture.reference.sha256, next.before, "original manifest reference identity");
  assert.equal(digest(archived), next.before, "whole immutable formatted reference");
  const current = fs.readFileSync(path.join(root, next.file));
  assert.equal(digest(current), next.after, "whole reviewed raw emitter reference");
  assertEquivalentCompilerModule(archived.toString("utf8"), current.toString("utf8"));
  return current;
}

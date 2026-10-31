import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync } from "node:fs";
import type tsApi from "../../../../npm/cli/node_modules/typescript/lib/typescript.js";
import { assertCustody, fileImage, type FileImage } from "../computed_event_custody.ts";
import type { Stage } from "./fixtures.ts";

type IntrinsicType = { flags: number; intrinsicName?: string; debugIntrinsicName?: string };

/** Inspect the actual bound slot value, not an IsAny-only surrogate. */
export function slotProviderTypes(
  ts: typeof tsApi,
  file: string,
  text: string,
  control: {
    roots?: string[];
    providerNames?: string[];
    noUnusedLocals?: boolean;
    noUnusedParameters?: boolean;
  } = {},
) {
  assert.equal(ts.version, "6.0.3", "intrinsic error identities require the pinned compiler");
  const options = {
    strict: true,
    noEmit: true,
    skipLibCheck: true,
    target: ts.ScriptTarget.ES2024,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    types: [],
    noUnusedLocals: control.noUnusedLocals,
    noUnusedParameters: control.noUnusedParameters,
  };
  const host = ts.createCompilerHost(options),
    original = host.getSourceFile.bind(host);
  const physicalInputs = new Map<string, FileImage>();
  host.getSourceFile = (name, language, error, fresh) => {
    if (name === file) return ts.createSourceFile(name, text, language, true);
    const before = existsSync(name) ? fileImage(name) : undefined;
    const source = original(name, language, error, fresh);
    if (source) {
      assert.ok(before, "each physical compiler input must have a file image");
      assert.deepEqual(fileImage(name), before, "compiler input changed while read");
      physicalInputs.set(before.path, before);
    }
    return source;
  };
  const roots = [file, ...(control.roots ?? [])];
  const program = ts.createProgram(roots, options, host);
  const source = program.getSourceFile(file);
  assert.ok(source);
  const checker = program.getTypeChecker();
  const rows: Array<{
    role: string;
    start: number;
    end: number;
    text: string;
    display: string;
    flags: number;
    any: boolean;
    intrinsicName: string | null;
    debugIntrinsicName: string | null;
    symbol: string | null;
    ownerFunction: string | null;
  }> = [];
  const record = (role: string, node: tsApi.Node) => {
    const type = checker.getTypeAtLocation(node) as tsApi.Type & IntrinsicType;
    let ownerFunction: string | null = null;
    for (let parent = node.parent; parent; parent = parent.parent) {
      if (ts.isFunctionExpression(parent) || ts.isFunctionDeclaration(parent)) {
        ownerFunction = parent.name?.getText(source) ?? null;
        break;
      }
    }
    rows.push({
      role,
      start: node.getStart(source),
      end: node.end,
      text: node.getText(source),
      display: checker.typeToString(type),
      flags: type.flags,
      any: (type.flags & ts.TypeFlags.Any) !== 0,
      intrinsicName: type.intrinsicName ?? null,
      debugIntrinsicName: type.debugIntrinsicName ?? null,
      symbol: checker.getSymbolAtLocation(node)?.getName() ?? null,
      ownerFunction,
    });
  };
  const visit = (node: tsApi.Node) => {
    if (
      ts.isVariableDeclaration(node) &&
      ts.isIdentifier(node.name) &&
      control.providerNames?.includes(node.name.text)
    )
      record("actual-provider", node.name);
    if (ts.isIdentifier(node) && node.text === "WitnessChild") record("provider", node);
    if (ts.isBindingElement(node) && ts.isIdentifier(node.name) && node.name.text === "item")
      record("slot-binding", node.name);
    if (
      ts.isCallExpression(node) &&
      ts.isIdentifier(node.expression) &&
      node.expression.text === "slotReport"
    )
      record("slot-report", node);
    ts.forEachChild(node, visit);
  };
  visit(source);
  assertCustody([...physicalInputs.values()]);
  return {
    compiler: ts.version,
    roots: program.getRootFileNames(),
    sourceSha256: createHash("sha256").update(text).digest("hex"),
    file,
    physicalInputs: [...physicalInputs.values()],
    rows,
    diagnostics: ts.getPreEmitDiagnostics(program).map((diagnostic) => ({
      file: diagnostic.file?.fileName ?? null,
      start: diagnostic.start ?? null,
      length: diagnostic.length ?? null,
      code: diagnostic.code,
      message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
    })),
  };
}

/** Literal C8 emits all template scopes before its component-props pass. */
function collisionDeclarations(stage: Stage, text: string) {
  const tags = stage.collisionTags;
  assert.ok(tags && tags.length === 2);
  const matches = [
    ...text.matchAll(/^  void function (_slot_(props_)?default_([0-9]+))\(\{ item \}:/gmu),
  ];
  assert.equal(matches.length, 4, "all four literal slot declarations required");
  const declarations = matches.map((match, index) => {
    const ownerFunction = match[1]!,
      scope = match[3]!,
      props = Boolean(match[2]);
    const tag = tags[index % 2],
      key = tag === "A-b" ? "AB" : "A_b";
    const end = text.indexOf("\n", match.index);
    assert.ok(end > match.index);
    assert.equal(
      text.slice(match.index, end),
      `  void function ${ownerFunction}({ item }: import("vize/virtual-ts").$SlotPayload<typeof __vize_slot_${props ? "props_" : ""}host_${scope}, "default", import("vize/virtual-ts").$RegisteredSlotProvider<import("vue").GlobalComponents, "${key}", "${key}", "${tag}">>) {`,
      "complete declaration must retain its literal host ID and frozen provider owner",
    );
    const start = match.index + match[0].lastIndexOf("item");
    return {
      ownerFunction,
      scope,
      role: props ? "props" : "template",
      start,
      end: start + 4,
      text: "item",
      symbol: "item",
    };
  });
  assert.deepEqual(
    declarations.map((row) => row.role),
    ["template", "template", "props", "props"],
  );
  const ids = declarations.map((row) => row.scope);
  assert.notEqual(ids[0], ids[1], "two distinct authored slot scopes required");
  assert.deepEqual(
    ids,
    [ids[0], ids[1], ids[0], ids[1]],
    "props must retain each template's exact scope ID",
  );
  return declarations;
}

export function assertTypeIdentity(
  stage: Stage,
  observed: ReturnType<typeof slotProviderTypes>,
  text?: string,
) {
  if (text !== undefined) {
    assert.equal(observed.sourceSha256, createHash("sha256").update(text).digest("hex"));
    for (const row of observed.rows) {
      assert.ok(
        Number.isSafeInteger(row.start) &&
          Number.isSafeInteger(row.end) &&
          row.start >= 0 &&
          row.end > row.start &&
          row.end <= text.length,
      );
      assert.equal(
        text.slice(row.start, row.end),
        row.text,
        "exact UTF16 position and declaration text",
      );
    }
  }
  const bindings = observed.rows.filter((row) => row.role === "slot-binding");
  const reports = observed.rows.filter((row) => row.role === "slot-report");
  const providers = observed.rows.filter((row) => row.role === "actual-provider");
  assert.equal(bindings.length, stage.slotRoles.length, "all actual emitted slot scopes required");
  const scopes = bindings.map((row) => {
    const match = /^_slot_(props_)?default_([0-9]+)$/u.exec(row.ownerFunction ?? "");
    assert.ok(match, "actual slot function ownership required");
    return { role: match[1] ? "props" : "template", scope: match[2] };
  });
  assert.deepEqual(
    scopes.map((row) => row.role),
    stage.slotRoles,
  );
  if (stage.population !== "collision16") {
    assert.equal(new Set(scopes.map((row) => row.scope)).size, 1);
    const kind = stage.kind!;
    // Actual TypeScript 6.0.3 enum: String=32 and Never=262144.
    const flags = { number: 64, any: 1, error: 1, unknown: 2, never: 262144, string: 32 };
    assert.deepEqual(
      bindings.map(({ flags, any, intrinsicName }) => ({ flags, any, intrinsicName })),
      stage.slotRoles.map(() => ({
        flags: flags[kind],
        any: kind === "any" || kind === "error",
        intrinsicName: kind,
      })),
    );
    assert.deepEqual(
      reports.map(({ flags, any, display, intrinsicName }) => ({
        flags,
        any,
        display,
        intrinsicName,
      })),
      [
        kind === "error"
          ? { flags: 1, any: true, display: "any", intrinsicName: "error" }
          : {
              flags: 1024,
              any: false,
              display: JSON.stringify(kind === "string" ? "other" : kind),
              intrinsicName: null,
            },
      ],
    );
    if (stage.population === "registered15") {
      assert.equal(providers.length, 1);
      assert.equal(providers[0]!.any, kind !== "number");
      if (kind === "any") assert.notEqual(providers[0]!.intrinsicName, "error");
    }
  } else {
    assert.ok(text !== undefined, "collision identities require their complete C8 VTS source");
    const declarations = collisionDeclarations(stage, text);
    assert.deepEqual(
      bindings.map(({ start, end, text, symbol, ownerFunction }) => ({
        start,
        end,
        text,
        symbol,
        ownerFunction,
      })),
      declarations.map(({ start, end, text, symbol, ownerFunction }) => ({
        start,
        end,
        text,
        symbol,
        ownerFunction,
      })),
    );
    const tags = stage.collisionTags!,
      anyKey = stage.collisionAnyKey;
    assert.ok(anyKey === "AB" || anyKey === "A_b");
    const key = (tag: string) => (tag === "A-b" ? "AB" : "A_b");
    const kind = (key: string) => (stage.collisionInvalid && key === anyKey ? "any" : "number");
    const kinds = tags.map((tag) => kind(key(tag)));
    assert.equal(providers.length, 2);
    assert.equal(new Set(providers.map((row) => row.symbol)).size, 2);
    const providerKeys: Record<string, string> = {
      $Component_A_x2d_b: "AB",
      A_b: "A_b",
      $Component_A_x5f_b: "A_b",
    };
    assert.equal(new Set(providers.map((row) => providerKeys[row.symbol ?? ""])).size, 2);
    for (const row of providers) {
      const owner = providerKeys[row.symbol ?? ""];
      assert.ok(owner === "AB" || owner === "A_b", "literal registered provider symbol required");
      assert.equal(row.any, kind(owner) === "any");
      assert.equal(row.intrinsicName, kind(owner) === "any" ? "any" : null);
    }
    assert.deepEqual(
      bindings.map(({ flags, any, intrinsicName, debugIntrinsicName, display }) => ({
        flags,
        any,
        intrinsicName,
        debugIntrinsicName,
        display,
      })),
      [...kinds, ...kinds].map((kind) => ({
        flags: kind === "any" ? 1 : 64,
        any: kind === "any",
        intrinsicName: kind,
        debugIntrinsicName: null,
        display: kind,
      })),
    );
    assert.equal(reports.length, 2);
    assert.deepEqual(
      reports.map(({ flags, any, intrinsicName, display, ownerFunction }) => ({
        flags,
        any,
        intrinsicName,
        display,
        ownerFunction,
      })),
      kinds.map((kind, index) => ({
        flags: 1024,
        any: false,
        intrinsicName: null,
        display: JSON.stringify(kind),
        ownerFunction: declarations[index].ownerFunction,
      })),
    );
  }
  assert.ok(
    observed.diagnostics.every((row) => row.file === observed.file),
    "no hidden external compiler diagnostics",
  );
  assert.deepEqual(
    observed.diagnostics.map(({ code, message }) => ({ code, message })),
    stage.expected.map(({ code, message }) => ({ code, message })),
    "whole VTS diagnostic messages",
  );
}

import assert from "node:assert/strict";
import type {
  BabelAst,
  BabelCore,
  BabelResult,
  MappingModule,
  TraceModule,
  WholeError,
} from "../host-types.ts";
import type { HostFixture } from "./fixtures.ts";

export function nodes(ast: unknown): BabelAst[] {
  const found: BabelAst[] = [],
    pending: unknown[] = [ast],
    seen = new Set<object>();
  while (pending.length) {
    const value = pending.pop();
    if (!value || typeof value !== "object" || seen.has(value)) continue;
    seen.add(value);
    if (typeof (value as { type?: unknown }).type === "string") found.push(value as BabelAst);
    for (const child of Object.values(value)) {
      if (Array.isArray(child)) pending.push(...child);
      else if (child && typeof child === "object") pending.push(child);
    }
  }
  return found;
}
export function hasJsx(ast: unknown) {
  return nodes(ast).some((node) => node.type === "JSXElement" || node.type === "JSXFragment");
}
export function editJsx(ast: unknown, value: string) {
  const attributes = nodes(ast).filter(
    (node) =>
      node.type === "JSXAttribute" &&
      node.name.type === "JSXIdentifier" &&
      node.name.name === "data-probe",
  );
  assert.equal(attributes.length, 1, "the authored attribute edit must be unambiguous");
  assert(attributes[0].type === "JSXAttribute" && attributes[0].value?.type === "StringLiteral");
  attributes[0].value.value = value;
}
export function removeJsx(core: BabelCore, ast: unknown) {
  const arrow = nodes(ast).find((node) => node.type === "ArrowFunctionExpression");
  assert(arrow && hasJsx(arrow.body));
  arrow.body = core.types.numericLiteral(42);
}
export function wholeError(error: unknown): WholeError {
  const value = error as { name: unknown; constructor: { name: string }; [key: string]: unknown };
  return {
    name: value.name,
    constructor: value.constructor.name,
    ownProperties: Object.fromEntries(
      Object.getOwnPropertyNames(error).map((key) => [
        key,
        value[key] === undefined ? { type: "undefined" } : value[key],
      ]),
    ),
  };
}
export function assertCompiled(
  result: BabelResult | null,
  fixture: HostFixture,
): asserts result is BabelResult & { code: string; ast: Extract<BabelAst, { type: "File" }> } {
  assert(result && typeof result.code === "string" && result.ast);
  assert.equal(hasJsx(result.ast), false, "the complete returned AST has no remaining JSX");
  assert.deepEqual(result.metadata.vize!.jsx, {
    defaultMode: "vdom",
    ssr: false,
    nativePasses: 1,
    warnings: [],
    scopedStyles: [],
  });
  if (fixture.pre === "edit") assert.equal(result.metadata.vize!.predecessor, "preserved");
  if (fixture.typescript)
    assert.equal(
      nodes(result.ast).filter((node) => node.type.startsWith("TS")).length,
      0,
      "the actual host TypeScript transform must run",
    );
  if (fixture.decorators)
    assert.equal(
      nodes(result.ast).filter((node) => node.type === "Decorator").length,
      0,
      "the actual host decorator transform must run",
    );
  if (fixture.classProperties)
    assert.equal(
      nodes(result.ast).filter((node) => node.type === "ClassProperty").length,
      0,
      "the actual host class transform must run",
    );
  if (fixture.prologue) {
    assert.equal(result.ast.program.interpreter!.value, "/usr/bin/env node");
    assert.equal(
      result.ast.program.directives.filter((d) => d.value.value === "use client").length,
      1,
    );
    for (const marker of ["authored-leading 🧭", "authored-trailing"]) {
      assert.equal(
        result.ast.comments!.filter((c) => c.value.trim() === marker).length,
        1,
        "the complete comment survives exactly once",
      );
    }
  }
  if (fixture.comments === false) {
    assert(!result.code.includes("authored-leading"));
    assert(!result.code.includes("authored-trailing"));
  }
}
export function position(text: string, token: string) {
  assert.equal(text.split(token).length, 2, "unique authored source-map anchor");
  const at = text.indexOf(token),
    prefix = text.slice(0, at);
  return {
    line: prefix.split("\n").length,
    column: prefix.slice(prefix.lastIndexOf("\n") + 1).length,
  };
}
export function upstreamMap(mapping: MappingModule, source: string, sourceName: string) {
  const map = new mapping.GenMapping({ file: sourceName });
  const lines = source.split(/\r\n|\n|\r/);
  const authored =
    "// upstream header\n// second authored line\n" + lines.map((line) => "   " + line).join("\n");
  mapping.setSourceContent(map, "authored/origin.jsx", authored);
  for (let line = 0; line < lines.length; line++) {
    for (let column = 0; column <= lines[line].length; column++) {
      mapping.addMapping(map, {
        generated: { line: line + 1, column },
        source: "authored/origin.jsx",
        original: { line: line + 3, column: column + 3 },
      });
    }
  }
  return { map: mapping.toEncodedMap(map), authored };
}
export function assertUpstream(trace: TraceModule, result: BabelResult, source: string) {
  assert(result.code && result.map);
  const generated = position(result.code, "authoredToken =");
  const original = position(source, "authoredToken =");
  const map = new trace.TraceMap(result.map);
  const actual = trace.originalPositionFor(map, generated);
  assert.deepEqual(
    { source: actual.source, line: actual.line, column: actual.column },
    { source: "authored/origin.jsx", line: original.line + 2, column: original.column + 3 },
  );
  if (actual.name !== null) assert.equal(actual.name, "authoredToken");
  assert.equal(
    result.map.sourcesContent![result.map.sources.indexOf(actual.source!)],
    "// upstream header\n// second authored line\n" +
      source
        .split(/\r\n|\n|\r/)
        .map((line) => "   " + line)
        .join("\n"),
  );
}

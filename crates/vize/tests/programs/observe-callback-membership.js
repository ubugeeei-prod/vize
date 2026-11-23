import fs from "node:fs";
import { createRequire } from "node:module";
const req = createRequire(process.argv[2]);
const ts = req(process.argv[2]);
const inputs = JSON.parse(fs.readFileSync(process.argv[3], "utf8"));
const rows = inputs.map((input) => {
  const sf = ts.createSourceFile(
    "Parent.vue.ts",
    input.virtualTs,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  const imports = [],
    negative = [];
  const point = (node) => {
    const p = sf.getLineAndCharacterOfPosition(node.getStart(sf));
    return { line: p.line + 1, column: p.character + 1 };
  };
  const stringMarkers = new Set(["direct_bad", "callback_bad"]);
  const missingMarkers = new Set(["directMissing", "callbackMissing"]);
  const ancestor = (node, predicate) => {
    for (let current = node; current; current = current.parent)
      if (predicate(current)) return current;
    return null;
  };
  const owner = (node) => {
    const declaration = ancestor(node, ts.isVariableDeclaration);
    if (!declaration) {
      const statement = ancestor(node, ts.isExpressionStatement);
      if (statement && ts.isVoidExpression(statement.expression)) return "interpolation";
      throw new Error("unknown interpolation owner");
    }
    if (!ts.isIdentifier(declaration.name)) throw new Error("non-identifier negative owner");
    if (declaration.name.text === "__vize_prop_check_0_textConverter") return "checked-callback";
    if (declaration.name.text !== "__vize_resolved_Formatter_props_0")
      throw new Error("unknown callback owner");
    const property = ancestor(node, ts.isPropertyAssignment);
    if (!property || !ts.isStringLiteral(property.name) || property.name.text !== "textConverter")
      throw new Error("unknown inference property");
    const comments = (ts.getLeadingCommentRanges(input.virtualTs, property.pos) ?? []).map(
      (range) => input.virtualTs.slice(range.pos, range.end),
    );
    if (
      JSON.stringify(comments) !==
      JSON.stringify([
        "// @ts-ignore Inference-only callback prop; mapped prop owner checks diagnostics.",
      ])
    )
      throw new Error("unguarded inference copy");
    if (point(property).line !== point(node).line)
      throw new Error("inference diagnostic escaped the single ignored line");
    return "inference-only";
  };
  function visit(node) {
    if (ts.isImportDeclaration(node) && node.moduleSpecifier.text === "./Formatter.vue.ts")
      imports.push({ ...point(node.moduleSpecifier), text: node.moduleSpecifier.text });
    if (
      ts.isStringLiteral(node) &&
      stringMarkers.has(node.text) &&
      ts.isCallExpression(node.parent) &&
      ts.isPropertyAccessExpression(node.parent.expression) &&
      node.parent.expression.name.text === "repeat"
    )
      negative.push({
        ...point(node),
        owner: owner(node),
        marker: "'" + node.text + "'",
        code: 2345,
        message: "Argument of type 'string' is not assignable to parameter of type 'number'.",
      });
    if (ts.isPropertyAccessExpression(node) && missingMarkers.has(node.name.text))
      negative.push({
        ...point(node.name),
        owner: owner(node),
        marker: node.name.text,
        code: 2339,
        message: "Property '" + node.name.text + "' does not exist on type 'Priority'.",
      });
    ts.forEachChild(node, visit);
  }
  visit(sf);
  negative.sort((a, b) => a.line - b.line || a.column - b.column);
  return {
    phase: input.phase,
    imports,
    negative,
    parseDiagnostics: sf.parseDiagnostics.map((d) => ({
      code: d.code,
      start: d.start,
      length: d.length,
      messageText: d.messageText,
    })),
  };
});
console.log(JSON.stringify({ typescript: ts.version, rows }));
